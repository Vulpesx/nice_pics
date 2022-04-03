use std::{str::FromStr, fmt::{Display, write}};
use std::convert::TryFrom;

use anyhow::{bail, Result};

type Error = anyhow::Error;

/// A chunk type is a 4 byte array of valid ASCII chars
/// however they should not be treated as chars.
/// This type code is only relevent to software that uses it
/// the only thing that matters to the file is the case of the chars
/// or the 5 bit of each byte.
#[derive(Debug, PartialEq)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    /// The case / bit 5 of the first byte on signifies if the chunk
    /// is critical, if critical it should remain even if its use is unknown.
    pub fn is_critical(&self) -> bool {
        !ChunkType::get_bit_at(self.bytes[0], 5).unwrap()
    }

    /// The case / bit 5 of the seccond byte signifies if the chunk
    /// is public. Public means it is part of the PNG specification or
    /// is registered in the list of PNG special purpose-public chunks.
    pub fn is_public(&self) -> bool {
        !ChunkType::get_bit_at(self.bytes[1], 5).unwrap()
    }

    /// the case / bit 5 of the third byte has no meaning as of writing and
    /// is reserved. this bit should always be true / uppercase. even when
    /// lowercase / false it should not be treated differently.
    pub fn is_reserved_bit_valid(&self) -> bool {
        !ChunkType::get_bit_at(self.bytes[2], 5).unwrap()
    }

    /// The case / bit 5 of the fourth byte signifies if it is safe to copy
    /// this should always be the case unless the data in the chunk is calculated
    /// based on previous chunks.
    pub fn is_safe_to_copy(&self) -> bool {
        ChunkType::get_bit_at(self.bytes[3], 5).unwrap()
    }

    /// See `is_reserved_bit_valid`
    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }

    /// returns true if bit at nth position is 1 otherwise false
    /// like all other things positions start at 0.
    ///
    /// # Panics
    /// panics if n > 8 as this operates on 8-bit bytes 
    fn get_bit_at(input: u8, n: u8) -> Result<bool, Error> {
        if n <= 8 {
            Ok(input & (1 << n) != 0)
        } else {
            bail!("nth bit is out of scope: n = {}", n);
        }
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    /// Tries to creat a `ChunkType` from `[u8; 4]`
    /// all bytes must be valid ASCII
    ///
    /// # Errors
    /// returns an Error if a byte is invalid ASCII
    ///
    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        let range: Vec<u8> = (b'A'..=b'Z').chain(b'a'..=b'z').collect();
        for n in value {
            if !range.contains(&n) {
                bail!("invalid byte: {}/{}", n, n.to_string());
            }
        }

        Ok(ChunkType {bytes: value})
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    /// Tries to create a `ChunkType` from `&str`
    /// &str must be 4 bytes long.
    ///
    /// # Errors
    /// returns an Error if len != 4 
    fn from_str(s: &str) -> Result<Self, Self::Err> {
       if s.len() != 4 { bail!("invalid len of: {}", s.len()); }
       let s: Vec<u8> = s.bytes().take(4).collect();
       let mut bytes = [0u8; 4];
       for i in 0..4 {
           bytes[i] = s[i];
       }

       ChunkType::try_from(bytes)
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
