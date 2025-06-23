use byteorder::{ByteOrder, ReadBytesExt};
use std::io::Seek;

use crate::Error;

macro_rules! generate_read_array_method{
    ($typ:ty) => {
        paste::item! {
            #[doc = "Reads an array of [`" $typ "`]s. If len is none the reader will determine the length by reading it."]
            fn [< read_ $typ _array >]<T: ByteOrder> (&mut self, len: Option<usize>) -> Result<Vec<$typ>, Error>{
                let len = match len {
                    Some(v) => v,
                    None => self.read_array_len::<T>()?
                };
                let mut buf = vec![0; len];
                self.[< read_ $typ _into >]::<T>(&mut buf)?;
                Ok(buf)
            }
        }
    };
}

pub trait ReadUrexExt: ReadBytesExt {
    fn read_array_len<T: ByteOrder>(&mut self) -> Result<usize, Error> {
        let len = self.read_u32::<T>()?;
        Ok(len as usize)
    }

    fn read_cstr(&mut self) -> Result<String, Error> {
        let mut bytes = Vec::new();
        loop {
            let byte = self.read_u8()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }

        match String::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e).into()),
        }
    }

    fn read_string<T: ByteOrder>(&mut self) -> Result<String, Error> {
        let len = self.read_array_len::<T>()?;
        self.read_string_sized(len)
    }

    fn read_string_sized(&mut self, len: usize) -> Result<String, Error> {
        match String::from_utf8(self.read_bytes_sized(len)?) {
            Ok(s) => Ok(s),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e).into()),
        }
    }

    fn read_bytes<T: ByteOrder>(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.read_array_len::<T>()?;
        self.read_bytes_sized(len)
    }

    fn read_bytes_sized(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        let b = self.read_u8()?;
        Ok(b != 0)
    }

    generate_read_array_method!(i16);
    generate_read_array_method!(i32);
    generate_read_array_method!(i64);
    generate_read_array_method!(u16);
    generate_read_array_method!(u32);
    generate_read_array_method!(u64);
    //generate_read_array_method!(f32);
    //generate_read_array_method!(f64);
}

pub trait ReadSeekUrexExt: ReadUrexExt + Seek {
    fn align(&mut self, align: usize) -> Result<(), Error> {
        let pos = self.stream_position()?;
        let new_pos = (pos + align as u64 - 1) & !(align as u64 - 1);
        let diff = new_pos - pos;
        if diff > 0 {
            self.seek(std::io::SeekFrom::Current(diff as i64))?;
        }
        Ok(())
    }

    fn align4(&mut self) -> Result<(), Error> {
        self.align(4)
    }
}

impl<R: std::io::Read + ?Sized> ReadUrexExt for R {}
impl<R: std::io::Read + Seek + ?Sized> ReadSeekUrexExt for R {}
