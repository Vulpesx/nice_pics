use std::fmt::Display;

use crate::chunk_type::ChunkType;
use crate::crc;
use anyhow::bail;

type Error = anyhow::Error;

/// Png files are made of chunks of varying sizes
/// each chunk has a length, Type, Data and a CRC
/// the length is a u32 constructed from the first 
/// 4 bytes of a chunk and descrives the length of
/// the data field. The next 4 bytes make up the type.
/// Then comes the data which is a `Vec<u8>` of bytes.
/// The last 4 bytes make up the CRC `u32` wich was a
/// pain to calculate.
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    /// Creates a `Chunk` from `ChunkType` and `Vec<u8>`
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let crc = Chunk::calculate_crc(&chunk_type, &data);
        let length: u32 = data.len() as u32;
        Chunk { length, chunk_type, data, crc }
    }

    /// Calculates a 32 bit CRC by calling another function :)
    /// See `crc::crc32`.
    fn calculate_crc(chunk_type: &ChunkType, data: &Vec<u8>) -> u32 {
        let mut buf = chunk_type.bytes().to_vec();
        buf.extend(data);
        crc::crc32(buf.as_ref(), buf.len())
    }

    /// Returns a `Vec<u8>` of the chunk.
    /// containing all fields as `u8`
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = self.length.to_be_bytes().to_vec();
        bytes.extend(self.chunk_type.bytes().iter());
        bytes.extend(self.data.iter());
        bytes.extend(self.crc.to_be_bytes().iter());
        bytes
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    /// Returns data as `String`.
    ///
    /// # Errors
    /// returns an Error if any byte is invalid ASCII
    pub fn data_as_string(&self) -> Result<String, Error> {
        let data: &[u8] = self.data.as_ref();
        for b in data {
            if !b.is_ascii() {
                bail!("invalid char: {}", b);
            }
        }

        Ok(format!("{}", String::from_utf8_lossy(data)))
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    /// Tries to create a `Chunk` from `&[u8]`
    ///
    /// # Errors
    /// returns an Error if chunk type is invalid or crc is wrong
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let length = u32::from_be_bytes(value[0..4].try_into()?);
        let chunk_type: [u8; 4] = value[4..8].try_into()?;
        let chunk_type = ChunkType::try_from(chunk_type)?;
        let data = value[8..value.len()-4].try_into()?;
        let crc = u32::from_be_bytes(value[value.len()-4..].try_into()?);
        let calc_crc = Chunk::calculate_crc(&chunk_type, &data);
        if crc != calc_crc {
            bail!("invalid crc: {}, should be: {}", crc, calc_crc);
        }

        Ok(Chunk { length, chunk_type, data, crc })
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data_as_string().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();
        
        let _chunk_string = format!("{}", chunk);
    }
}
