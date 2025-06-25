use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PPtr {
    pub m_FileID: i64,
    pub m_PathID: i64,
}