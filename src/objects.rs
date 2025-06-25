pub use runirip_objects::*;

use crate::files::{ObjectReader, SerializedFile};

trait PPtrExt {
    fn get_object_reader<'a, R: std::io::Read + std::io::Seek>(
        &'a self,
        asset: &'a SerializedFile,
        reader: &'a mut R,
    ) -> Option<ObjectReader<'a, R>>;
}

impl PPtrExt for PPtr {
    fn get_object_reader<'a, R: std::io::Read + std::io::Seek>(
        &'a self,
        asset: &'a SerializedFile,
        reader: &'a mut R,
    ) -> Option<ObjectReader<'a, R>> {
        asset.m_Objects
            .iter()
            .find(|x| x.m_PathID == self.m_PathID)
            .map(|object_info| asset.get_object_reader(object_info, reader))
    }
}