use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PPtr {
    pub m_FileID: i64,
    pub m_PathID: i64,
}

impl PPtr {
    fn get_object_handler<'a, R: std::io::Read + std::io::Seek>(
        &'a self,
        asset: &'a crate::files::SerializedFile,
        reader: &'a mut R,
    ) -> Option<crate::files::ObjectHandler<'a, R>> {
        asset.m_Objects
            .iter()
            .find(|x| x.m_PathID == self.m_PathID)
            .map(|object_info| asset.get_object_handler(object_info, reader))
    }
}
