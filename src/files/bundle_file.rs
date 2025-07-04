use crate::{
    unitycn::ArchiveStorageDecryptor,
    config::ExtractionConfig,
    files::{SerializedFile, unity_file::{FileEntry, UnityFile}},
    read_ext::{ReadSeekUrexExt, ReadUrexExt}, Error,
};
use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::io::{Cursor, Read, Seek, SeekFrom};

bitflags! {
    struct ArchiveFlags: u32 {
        const COMPRESSION_TYPE_MASK = 0x3f;
        const BLOCKS_AND_DIRECTORY_INFO_COMBINED = 0x40;
        const BLOCKS_INFO_AT_THE_END = 0x80;
        const OLD_WEB_PLUGIN_COMPATIBILITY = 0x100;
        const BLOCK_INFO_NEED_PADDING_AT_START = 0x200;
        const USES_ASSET_BUNDLE_ENCRYPTION = 0x400;
    }

    struct ArchiveFlagsOld: u32 {
        const COMPRESSION_TYPE_MASK = 0x3f;
        const BLOCKS_AND_DIRECTORY_INFO_COMBINED = 0x40;
        const BLOCKS_INFO_AT_THE_END = 0x80;
        const OLD_WEB_PLUGIN_COMPATIBILITY = 0x100;
        const USES_ASSET_BUNDLE_ENCRYPTION = 0x200;
    }
}

// bitflags! {
//     struct StorageBlockFlags: u32 {
//         const CompressionTypeMask = 0x3f;
//         const Streamed = 0x40;
//         const Encrypted = 0x100;
//     }
// }

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum CompressionType {
    None = 0,
    Lzma = 1,
    Lz4 = 2,
    Lz4hc = 3,
    Lzham = 4,
}

#[derive(Debug)]
pub struct BundleFileHeader {
    signature: String,
    version: u32,
    unity_version: String,
    unity_revision: String,
    size: u32,
}

impl BundleFileHeader {
    fn from_reader<T: Read + Seek>(reader: &mut T) -> Result<Self, Error> {
        Ok(BundleFileHeader {
            signature: reader.read_cstr()?,
            version: reader.read_u32::<BigEndian>()?,
            unity_version: reader.read_cstr()?,
            unity_revision: reader.read_cstr()?,
            size: 0,
        })
    }

    fn get_revision_tuple(&self, config: &ExtractionConfig) -> Result<(u32, u32, u32), Error> {
        // could be done way better, but this works for now
        let mut revision = self.unity_revision.clone();
        if revision.is_empty() | (revision == "0.0.0") {
            revision = config.fallback_unity_version.clone();
        }
        let mut revision_split = revision.split('.');
        Ok((
            revision_split.next().map(|v| v.parse())
                .transpose()
                .ok()
                .flatten()
                .ok_or_else(|| Error::InvalidRevision(self.unity_revision.clone()))?,
            revision_split.next().map(|v| v.parse())
                .transpose()
                .ok()
                .flatten()
                .ok_or_else(|| Error::InvalidRevision(self.unity_revision.clone()))?,
            {
                let mut val = 0;
                let last_split = revision_split.next()
                    .ok_or_else(|| Error::InvalidRevision(self.unity_revision.clone()))?;

                for (i, c) in last_split.chars().enumerate() {
                    if !c.is_numeric() {
                        val = last_split[..i].parse::<u32>()?;
                        break;
                    }
                }
                val
            },
        ))
    }
}

#[derive(Debug)]
pub struct StorageBlock {
    compressed_size: u32,
    uncompressed_size: u32,
    flags: u32,
}

pub struct BundleFile {
    pub m_Header: BundleFileHeader,
    pub m_BlocksInfo: Vec<StorageBlock>,
    pub m_DirectoryInfo: Vec<FileEntry>,
    pub m_BlockReader: Cursor<Vec<u8>>,
    _decryptor: Option<ArchiveStorageDecryptor>,
}

impl BundleFile {
    pub fn from_reader<T: Read + Seek>(
        reader: &mut T,
        config: &ExtractionConfig,
    ) -> Result<Self, Error> {
        let mut bundle = Self {
            m_Header: BundleFileHeader::from_reader(reader)?,
            m_BlocksInfo: Vec::new(),
            m_DirectoryInfo: Vec::new(),
            m_BlockReader: Cursor::new(Vec::new()),
            _decryptor: None,
        };

        (bundle.m_DirectoryInfo, bundle.m_BlockReader) = match bundle.m_Header.signature.as_str() {
            "UnityArchive" => {
                return Err(Error::Unimplemented("UnityArchive is not supported"));
            }
            "UnityWeb" | "UnityRaw" => {
                if bundle.m_Header.version == 6 {
                    bundle.read_unityfs(reader, config)?
                } else {
                    bundle.read_unity_raw(reader, config)?
                }
            }
            "UnityFS" => bundle.read_unityfs(reader, config)?,
            _ => {
                return Err(Error::UnknownSignature);
            }
        };
        Ok(bundle)
    }

    fn read_unity_raw<T: Read + Seek>(
        &mut self,
        reader: &mut T,
        config: &ExtractionConfig,
    ) -> Result<(Vec<FileEntry>, Cursor<Vec<u8>>), Error> {
        if self.m_Header.version >= 4 {
            let hash = reader.read_u128::<BigEndian>()?;
            let crc = reader.read_u32::<BigEndian>()?;
        }
        let minimum_streamed_bytes = reader.read_u32::<BigEndian>()?;

        self.m_Header.size = reader.read_u32::<BigEndian>()?;

        let number_of_levels_to_download_before_streaming = reader.read_u32::<BigEndian>()?;
        let level_count = reader.read_u32::<BigEndian>()?;

        // jump to last level
        // TODO - keep the levels for use in low-memory block decompressor strategy
        reader
            .seek(std::io::SeekFrom::Current(((level_count - 1) * 8) as i64))?;

        let mut m_BlocksInfo = StorageBlock {
            compressed_size: reader.read_u32::<BigEndian>()?,
            uncompressed_size: reader.read_u32::<BigEndian>()?,
            flags: 0,
        };

        if self.m_Header.version >= 2 {
            let complete_file_size = reader.read_u32::<BigEndian>()?;
        }
        if self.m_Header.version >= 3 {
            let file_info_header_size = reader.read_u128::<BigEndian>()?;
        }
        reader
            .seek(std::io::SeekFrom::Start(self.m_Header.size as u64))?;

        //ReadBlocksAndDirectory
        // is compressed -> lzma compression -> can be passed to decompress_block
        if self.m_Header.signature == "UnityWeb" {
            m_BlocksInfo.flags += CompressionType::Lzma as u32;
        }

        let blocks_info_bytes = self.decompress_block(reader, &m_BlocksInfo, 0)?;
        let mut block_info_reader = Cursor::new(blocks_info_bytes);

        let FileEntrys_count = block_info_reader.read_i32::<BigEndian>()?;
        let m_DirectoryInfo = (0..FileEntrys_count)
            .map(|_| Ok(FileEntry {
                path: block_info_reader.read_cstr()?,
                offset: block_info_reader.read_u32::<BigEndian>()? as i64,
                size: block_info_reader.read_u32::<BigEndian>()? as i64,
                flags: 0,
            }))
            .collect::<Result<Vec<FileEntry>, Error>>()?;

        Ok((m_DirectoryInfo, block_info_reader))
    }

    fn read_unityfs<T: Read + Seek>(
        &mut self,
        reader: &mut T,
        config: &ExtractionConfig,
    ) -> Result<(Vec<FileEntry>, Cursor<Vec<u8>>), Error> {
        //ReadHeader
        let unity_ver = self.m_Header.get_revision_tuple(config)?;
        let use_new_archive_flags = !(unity_ver < (2020, 0, 0))
            | ((unity_ver.0 == 2020) & (unity_ver < (2020, 3, 34)))
            | ((unity_ver.0 == 2021) & (unity_ver < (2021, 3, 2)))
            | ((unity_ver.0 == 2022) & (unity_ver < (2022, 1, 1)));
        self.m_Header.size = reader.read_i64::<BigEndian>()? as u32;

        let block_info = StorageBlock {
            compressed_size: reader.read_u32::<BigEndian>()?,
            uncompressed_size: reader.read_u32::<BigEndian>()?,
            flags: reader.read_u32::<BigEndian>()?,
        };

        if self.m_Header.signature != "UnityFS" {
            reader.read_bool()?;
        }

        //ReadBlocksInfoAndDirectory
        if self.m_Header.version >= 7 {
            reader.align(16)?;
        }
        else if unity_ver.0 >= 2019 && unity_ver.1 >= 4 {
            //check if we need to align the reader
            //- align to 16 bytes and check if all are 0
            //- if not, reset the reader to the previous position
            let pre_align = reader.stream_position()?;
            let align_data = reader.read_bytes_sized((16 - (pre_align as usize % 16)) % 16)?;
            if align_data.iter().any(|x| *x != 0) {
                reader.seek(SeekFrom::Start(pre_align))?;
            }
        }

        let blocks_info_bytes: Vec<u8>;
        if block_info.flags & ArchiveFlags::BLOCKS_INFO_AT_THE_END.bits() != 0 {
            //0x80 BlocksInfoAtTheEnd
            let position = reader.stream_position()?;
            // originally reader.length
            reader
                .seek(std::io::SeekFrom::End(block_info.compressed_size as i64))?;
            blocks_info_bytes = self.decompress_block(reader, &block_info, 0)?;
            reader.seek(std::io::SeekFrom::Start(position))?;
        } else {
            //0x40 BlocksAndDirectoryInfoCombined
            if (use_new_archive_flags
                & (block_info.flags & ArchiveFlags::USES_ASSET_BUNDLE_ENCRYPTION.bits() > 0))
                | (!use_new_archive_flags
                    & (block_info.flags & ArchiveFlagsOld::USES_ASSET_BUNDLE_ENCRYPTION.bits() > 0))
            {
                #[cfg(feature = "unitycn_encryption")]
                {
                    self._decryptor = Some(ArchiveStorageDecryptor::from_reader(
                        reader,
                        config.unitycn_key.ok_or_else(|| Error::NoUnityCNKey)?,
                    )?);
                }

                #[cfg(not(feature = "unitycn_encryption"))]
                return Err(Error::FeatureDisabled("unitycn_encryption"));
            }
            blocks_info_bytes = self.decompress_block(reader, &block_info, 0)?;
        }

        let mut block_info_reader = Cursor::new(&blocks_info_bytes);

        let uncompressed_data_hash = block_info_reader.read_u128::<BigEndian>()?;

        let block_info_count = block_info_reader.read_i32::<BigEndian>()?;
        let m_BlocksInfo = (0..block_info_count)
            .map(|_| Ok(StorageBlock {
                uncompressed_size: block_info_reader.read_u32::<BigEndian>()?,
                compressed_size: block_info_reader.read_u32::<BigEndian>()?,
                flags: block_info_reader.read_u16::<BigEndian>()? as u32,
            }))
            .collect::<Result<Vec<StorageBlock>, Error>>()?;

        let FileEntrys_count = block_info_reader.read_i32::<BigEndian>()?;
        let m_DirectoryInfo: Vec<FileEntry> = (0..FileEntrys_count)
            .map(|_| Ok(FileEntry {
                offset: block_info_reader.read_i64::<BigEndian>()?,
                size: block_info_reader.read_i64::<BigEndian>()?,
                flags: block_info_reader.read_u32::<BigEndian>()?,
                path: block_info_reader.read_cstr()?,
            }))
            .collect::<Result<Vec<FileEntry>, Error>>()?;

        if use_new_archive_flags
            & (block_info.flags & ArchiveFlags::BLOCK_INFO_NEED_PADDING_AT_START.bits() > 0)
        {
            reader.align(16)?;
        }

        let block_data_size: u32 = m_BlocksInfo
            .iter()
            .map(|block| block.uncompressed_size)
            .sum();
        let mut block_data = vec![0u8; block_data_size as usize];

        let mut block_offset = 0usize;
        for (i, block) in m_BlocksInfo.iter().enumerate() {
            let end = block_offset + block.uncompressed_size as usize;
            self.decompress_block_into(reader, block, i, &mut block_data[block_offset..end])?;
            block_offset = end;
        }

        let block_reader = Cursor::new(block_data);
        Ok((m_DirectoryInfo, block_reader))
    }

    fn read_files<T: Read + Seek>(
        &mut self,
        file_entries: &[FileEntry],
        reader: &mut T,
        config: &ExtractionConfig,
    ) -> Result<Vec<SerializedFile>, Error> {
        file_entries
            .iter()
            .map(|entry| {
                reader.seek(std::io::SeekFrom::Start(entry.offset as u64))?;
                SerializedFile::from_reader(reader, config)
            })
            .collect()
    }

    fn decompress_block_into<T: Read + Seek>(
        &mut self,
        reader: &mut T,
        block: &StorageBlock,
        index: usize,
        output: &mut [u8]
    ) -> Result<(), Error> {
        #[allow(unused_mut)]
        let mut compressed = reader
            .read_bytes_sized(block.compressed_size as usize)?;

        match CompressionType::try_from(block.flags & 0x3F)? {
            CompressionType::Lzma => {
                #[cfg(feature = "lzma")]
                {
                    let mut compressed_reader = Cursor::new(&compressed);
                    lzma_rs::lzma_decompress(&mut compressed_reader, &mut Cursor::new(output))?;
                    Ok(())
                }

                #[cfg(not(feature = "lzma"))]
                Err(Error::FeatureDisabled("lzma"))
            }
            CompressionType::Lz4 | CompressionType::Lz4hc => {
                #[cfg(feature = "lz4")]
                {
                    if block.flags & 0x100 > 0 {
                        // UnityCN encryption
                        #[cfg(feature = "unitycn_encryption")]
                        if let Some(decryptor) = self._decryptor.as_ref() {
                            decryptor.decrypt_block(
                                &mut compressed,
                                block.compressed_size as usize,
                                index,
                            )?;
                        }

                        #[cfg(not(feature = "unitycn_encryption"))]
                        return Err(Error::FeatureDisabled("unitycn_encryption"))
                    }
                    lz4_flex::block::decompress_into(&compressed, output)?;
                    Ok(())
                }

                #[cfg(not(feature = "lz4"))]
                Err(Error::FeatureDisabled("lz4"))
            }
            CompressionType::Lzham => {
                Err(Error::Unimplemented("LZHAM is not supported"))
            }
            CompressionType::None => {
                output.copy_from_slice(&compressed);
                Ok(())
            }
        }
    }

    fn decompress_block<T: Read + Seek>(
        &mut self,
        reader: &mut T,
        block: &StorageBlock,
        index: usize,
    ) -> Result<Vec<u8>, Error> {
        let mut uncompressed = vec![0; block.uncompressed_size as usize];
        self.decompress_block_into(reader, block, index, &mut uncompressed)?;
        Ok(uncompressed)
    }
}

impl UnityFile for BundleFile {
    fn from_reader<T: Read + Seek>(reader: &mut T, config: &ExtractionConfig) -> Result<Self, Error>
    where
        Self: Sized,
    {
        BundleFile::from_reader(reader, config)
    }
}
