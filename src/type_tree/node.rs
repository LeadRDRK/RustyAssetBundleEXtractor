#![allow(clippy::redundant_closure_call)]
use std::collections::HashMap;

use crate::{common_strings::COMMON_STRINGS, TypeTreeValue};
use crate::read_ext::ReadUrexExt;
use crate::Error;
use bitflags::bitflags;
use byteorder::{ByteOrder, ReadBytesExt};

bitflags! {
    struct TransferMetaFlags: i32 {
        const NO_TRANSFER_FLAGS = 0;
        /// Putting this mask in a transfer will make the variable be hidden in the property editor
        const HIDE_IN_EDITOR_MASK = 1 << 0;

        /// Makes a variable not editable in the property editor
        const NOT_EDITABLE_MASK = 1 << 4;

        /// There are 3 types of PPtrs: kStrongPPtrMask, default (weak pointer)
        /// a Strong PPtr forces the referenced object to be cloned.
        /// A Weak PPtr doesnt clone the referenced object, but if the referenced object is being cloned anyway (eg. If another (strong) pptr references this object)
        /// this PPtr will be remapped to the cloned object
        /// If an  object  referenced by a WeakPPtr is not cloned, it will stay the same when duplicating and cloning, but be NULLed when templating
        const STRONG_PPTR_MASK = 1 << 6;
        // unused  = 1 << 7,

        /// kEditorDisplaysCheckBoxMask makes an integer variable appear as a checkbox in the editor
        const EDITOR_DISPLAYS_CHECK_BOX_MASK = 1 << 8;

        // unused = 1 << 9,
        // unused = 1 << 10,

        /// Show in simplified editor
        const SIMPLE_EDITOR_MASK = 1 << 11;

        /// When the options of a serializer tells you to serialize debug properties kSerializeDebugProperties
        /// All debug properties have to be marked kDebugPropertyMask
        /// Debug properties are shown in expert mode in the inspector but are not serialized normally
        const DEBUG_PROPERTY_MASK = 1 << 12;

        const ALIGN_BYTES_FLAG = 1 << 14;
        const ANY_CHILD_USES_ALIGN_BYTES_FLAG = 1 << 15;
        const IGNORE_WITH_INSPECTOR_UNDO_MASK = 1 << 16;

        // unused = 1 << 18,

        // Ignore this property when reading or writing .meta files
        const IGNORE_IN_META_FILES = 1 << 19;

        // When reading meta files and this property is not present, read array entry name instead (for backwards compatibility).
        const TRANSFER_AS_ARRAY_ENTRY_NAME_IN_META_FILES = 1 << 20;

        // When writing YAML Files, uses the flow mapping style (all properties in one line, with "{}").
        const TRANSFER_USING_FLOW_MAPPING_STYLE = 1 << 21;

        // Tells SerializedProperty to generate bitwise difference information for this field.
        const GENERATE_BITWISE_DIFFERENCES = 1 << 22;

        const DONT_ANIMATE = 1 << 23;
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    m_Version: i32,
    m_Level: u8,
    m_TypeFlags: i32,
    m_ByteSize: i32,
    m_Index: Option<i32>,
    m_MetaFlag: Option<i32>,
    m_Type: String,
    m_Name: String,
    //unsigned short children_count,
    //struct TypeTreeNodeObject **children,
    // UnityFS
    // unsigned int m_TypeStrOffset,
    // unsigned int m_NameStrOffset,
    // UnityFS - version >= 19
    m_RefTypeHash: Option<u64>,
    // UnityRaw - versin = 2
    m_VariableCount: Option<i32>,
    // helper fields
    //typehash: u32,
    children: Vec<Node>,
}
impl Node {
    pub fn from_reader<R: std::io::Read + std::io::Seek, B: ByteOrder>(
        reader: &mut R,
        version: u32,
    ) -> Result<Node, Error> {
        fn read_node_base<R: std::io::Read + std::io::Seek, B: ByteOrder>(
            reader: &mut R,
            version: u32,
            level: u8,
        ) -> Result<Node, Error> {
            let mut node = Node {
                m_Level: level,
                m_Type: reader.read_cstr()?,
                m_Name: reader.read_cstr()?,
                m_ByteSize: reader.read_i32::<B>()?,
                m_VariableCount: if version == 2 {
                    Some(reader.read_i32::<B>()?)
                } else {
                    None
                },
                m_Index: if version != 3 {
                    Some(reader.read_i32::<B>()?)
                } else {
                    None
                },
                // in version 4, m_TypeFlags are m_IsArray
                m_TypeFlags: reader.read_i32::<B>()?,
                m_Version: reader.read_i32::<B>()?,
                m_MetaFlag: if version != 3 {
                    Some(reader.read_i32::<B>()?)
                } else {
                    None
                },
                m_RefTypeHash: None,
                children: Vec::new(),
            };
            let children_count = reader.read_i32::<B>()?;
            node.children = (0..children_count)
                .map(|_| read_node_base::<R, B>(reader, version, node.m_Level + 1))
                .collect::<Result<Vec<Node>, Error>>()?;
            Ok(node)
        }
        Ok(read_node_base::<R, B>(reader, version, 0)?)
    }

    pub fn blob_from_reader<R: std::io::Read + std::io::Seek, B: ByteOrder>(
        reader: &mut R,
        version: u32,
    ) -> Result<Node, Error> {
        // originally a list with level slicing
        // reordered here to fit the newer tree structure
        let node_size = if version >= 19 { 32 } else { 24 };
        let node_count = reader.read_i32::<B>()?;
        let string_buffer_size = reader.read_i32::<B>()?;

        let mut node_reader = std::io::Cursor::new(
            reader.read_bytes_sized(node_size as usize * node_count as usize)?,
        );
        let mut string_buffer_reader =
            std::io::Cursor::new(reader.read_bytes_sized(string_buffer_size as usize)?);

        fn read_string<R: std::io::Read + std::io::Seek, B: ByteOrder>(
            string_buffer_reader: &mut R,
            value: u32,
        ) -> Result<String, Error> {
            // TODO - cache strings
            let isOffset = (value & 0x80000000) == 0;
            if isOffset {
                string_buffer_reader
                    .seek(std::io::SeekFrom::Start(value as u64))?;
                return string_buffer_reader.read_cstr();
            }
            let offset = value & 0x7FFFFFFF;

            let ret = COMMON_STRINGS.get(&offset);

            if let Some(ret) = ret {
                Ok(ret.to_string())
            } else {
                Ok(offset.to_string())
            }
        }

        let nodes = (0..node_count)
            .map(|_| Ok(Node {
                m_Version: node_reader.read_u16::<B>()? as i32,
                m_Level: node_reader.read_u8()?,
                m_TypeFlags: node_reader.read_u8()? as i32,
                m_Type: read_string::<std::io::Cursor<Vec<u8>>, B>(
                    &mut string_buffer_reader,
                    node_reader.read_u32::<B>()?,
                )
                ?,
                m_Name: read_string::<std::io::Cursor<Vec<u8>>, B>(
                    &mut string_buffer_reader,
                    node_reader.read_u32::<B>()?,
                )
                ?,
                m_ByteSize: node_reader.read_i32::<B>()?,
                m_Index: Some(node_reader.read_i32::<B>()?),
                m_MetaFlag: Some(node_reader.read_i32::<B>()?),
                m_RefTypeHash: if version >= 19 {
                    Some(node_reader.read_u64::<B>()?)
                } else {
                    None
                },
                children: Vec::new(),
                m_VariableCount: None,
            }))
            .collect::<Result<Vec<Node>, Error>>()?;

        fn add_children(parent: &mut Node, nodes: &[Node], offset: usize) -> i32 {
            let mut added: i32 = 0;
            for i in (offset + 1)..nodes.len() {
                let mut node = nodes[i].clone();
                if node.m_Level == parent.m_Level + 1 {
                    added += add_children(&mut node, nodes, i) + 1;
                    parent.children.push(node.clone());
                } else if node.m_Level <= parent.m_Level {
                    break;
                }
            }
            added
        }

        let mut root_node = nodes[0].clone();
        let added = add_children(&mut root_node, &nodes, 0);

        #[cfg(debug_assertions)]
        if added != node_count - 1 {
            println!("Warning: not all nodes were added to the tree");
        }

        Ok(root_node)
    }

    fn requires_align(&self) -> bool {
        (self.m_MetaFlag.unwrap_or(0) & TransferMetaFlags::ALIGN_BYTES_FLAG.bits()) != 0
    }

    pub fn read<R: std::io::Read + std::io::Seek, B: ByteOrder>(&self, reader: &mut R) -> Result<TypeTreeValue, Error> {
        use crate::read_ext::ReadSeekUrexExt;

        let mut align = self.requires_align();
        let value = match self.m_Type.as_str() {
            "SInt8" => {
                TypeTreeValue::SInt8(reader.read_i8()?)
            }
            "UInt8" => {
                TypeTreeValue::UInt8(reader.read_u8()?)
            }
            "char" => {
                TypeTreeValue::Char(reader.read_u8()? as char)
            }
            "SInt16" | "short" => {
                TypeTreeValue::SInt16(reader.read_i16::<B>()?)
            }
            "UInt16" | "unsigned short" => {
                TypeTreeValue::UInt16(reader.read_u16::<B>()?)
            }
            "SInt32" | "int" => {
                TypeTreeValue::SInt32(reader.read_i32::<B>()?)
            }
            "UInt32" | "unsigned int" => {
                TypeTreeValue::UInt32(reader.read_u32::<B>()?)
            }
            "Type*" => {
                TypeTreeValue::Type(reader.read_u32::<B>()?)
            }
            "SInt64" | "long long" => {
                TypeTreeValue::SInt64(reader.read_i64::<B>()?)
            }
            "UInt64" | "unsigned long long" => {
                TypeTreeValue::UInt64(reader.read_u64::<B>()?)
            }
            "FileSize" => {
                TypeTreeValue::FileSize(reader.read_u64::<B>()?)
            }
            "float" => {
                TypeTreeValue::Float(reader.read_f32::<B>()?)
            }
            "double" => {
                TypeTreeValue::Double(reader.read_f64::<B>()?)
            }
            "bool" => {
                TypeTreeValue::Bool(reader.read_bool()?)
            }
            "string" => {
                align |= &self.children[0].requires_align();
                TypeTreeValue::String(reader.read_string::<B>()?)
            }
            "TypelessData" => {
                TypeTreeValue::TypelessData(reader.read_bytes::<B>()?)
            }
            "map" => {
                // map m_Container
                //  Array Array
                //      int size
                //      pair data
                //          TYPE first
                //          TYPE second
                if self.children.len() != 1 {
                    return Err(Error::InvalidValue("Malformed map node".to_owned()));
                }

                let array = &self.children[0];
                if array.children.len() != 2 {
                    return Err(Error::InvalidValue("Malformed map node".to_owned()));
                }

                let size = reader.read_array_len::<B>()?;
                let pair = &self.children[0].children[1];
                align |= pair.requires_align();

                if pair.children.len() != 2 {
                    return Err(Error::InvalidValue("Malformed map node".to_owned()));
                }

                let first = &pair.children[0];
                let second = &pair.children[1];

                TypeTreeValue::Map(
                    (0..size)
                        .map(|_| Ok(
                            (
                                first.read::<R, B>(reader)?,
                                second.read::<R, B>(reader)?
                            )
                        ))
                        .collect::<Result<Vec<(TypeTreeValue, TypeTreeValue)>, Error>>()?
                )
            }
            default => {
                // array
                //vector m_Component // ByteSize{ffffffff}, Index{1}, Version{1}, IsArray{0}, MetaFlag{8041}
                //  Array Array // ByteSize{ffffffff}, Index{2}, Version{1}, IsArray{1}, MetaFlag{4041}
                //      int size // ByteSize{4}, Index{3}, Version{1}, IsArray{0}, MetaFlag{41}
                //      ComponentPair data // ByteSize{c}, Index{4}, Version{1}, IsArray{0}, MetaFlag{41}
                if self.children.len() == 1 && self.children[0].m_Type == "Array" {
                    let array = &self.children[0];
                    if array.children.len() != 2 {
                        return Err(Error::InvalidValue("Malformed array node".to_owned()));
                    }

                    align |= array.requires_align();

                    let size = reader.read_array_len::<B>()?;
                    let data = &array.children[1];

                    TypeTreeValue::Array(
                        (0..size)
                            .map(|_| data.read::<R, B>(reader))
                            .collect::<Result<Vec<TypeTreeValue>, Error>>()?,
                    )
                } else {
                    // class
                    let mut map = HashMap::new();
                    for child in self.children.iter() {
                        map.insert(
                            child.m_Name.clone(),
                            child.read::<R, B>(reader)?
                        );
                    }
                    TypeTreeValue::Class(map)
                }
            }
        };
        if align {
            reader.align4()?;
        }
        Ok(value)
    }
}
