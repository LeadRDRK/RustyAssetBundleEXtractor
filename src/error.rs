use std::fmt::Display;

use num_enum::TryFromPrimitiveError;

use crate::files::bundle_file::CompressionType;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
    TryFromSliceError(std::array::TryFromSliceError),

    FeatureDisabled(&'static str),
    Unimplemented(&'static str),

    InvalidRevision(String),
    InvalidCompressionFlag(u32),
    InvalidEndianness,
    TypeTreeNotFound,
    UnknownSignature,
    InvalidValue(String),
    DecompressionError(String),
    NoUnityCNKey
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::ParseIntError(e) => e.fmt(f),
            Self::TryFromSliceError(e) => e.fmt(f),

            Self::FeatureDisabled(name) => write!(f, "Feature disabled: {name}"),
            Self::Unimplemented(reason) => write!(f, "Unimplemented: {reason}"),

            Self::InvalidRevision(rev) => write!(f, "Invalid revision: {rev}"),
            Self::InvalidCompressionFlag(flag) => write!(f, "Invalid compression flag: {flag}"),
            Self::InvalidEndianness => f.write_str("Invalid endianness"),
            Self::TypeTreeNotFound => f.write_str("Unable to find type tree"),
            Self::UnknownSignature => f.write_str("Unknown signature"),
            Self::InvalidValue(reason) => write!(f, "Invalid value: {reason}"),
            Self::DecompressionError(reason) => write!(f, "Decompression error: {reason}"),
            Self::NoUnityCNKey => f.write_str("UnityCN decryption key was not provided")
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

impl From<TryFromPrimitiveError<CompressionType>> for Error {
    fn from(e: TryFromPrimitiveError<CompressionType>) -> Self {
        Self::InvalidCompressionFlag(e.number)
    }
}

impl From<std::array::TryFromSliceError> for Error {
    fn from(e: std::array::TryFromSliceError) -> Self {
        Self::TryFromSliceError(e)
    }
}

#[cfg(feature = "lzma")]
impl From<lzma_rs::error::Error> for Error {
    fn from(e: lzma_rs::error::Error) -> Self {
        Self::DecompressionError(e.to_string())
    }
}

#[cfg(feature = "lz4")]
impl From<lz4_flex::block::DecompressError> for Error {
    fn from(e: lz4_flex::block::DecompressError) -> Self {
        Self::DecompressionError(e.to_string())
    }
}