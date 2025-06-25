#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]
pub mod asset_manager;
pub mod config;
pub mod files;
pub mod read_ext;

mod unitycn;

mod error;
pub use error::Error;

mod common_strings;
pub use common_strings::COMMON_STRINGS;

pub mod type_tree;
pub type TypeTreeNode = type_tree::Node;
pub type TypeTreeValue = type_tree::Value;

#[cfg(feature = "objects")]
pub mod objects;

#[cfg(feature = "objects")]
pub use objects::classes::ids as class_ids;