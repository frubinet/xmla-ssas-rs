// SPDX-License-Identifier: MPL-2.0

use crate::dime::error::DimeError;
use std::io;
use std::io::{Read, Write};

pub(crate) trait ReadExt: Read {
    fn read_u8(&mut self) -> Result<u8, DimeError>;
    fn read_u16_be(&mut self) -> Result<u16, DimeError>;
    fn read_u32_be(&mut self) -> Result<u32, DimeError>;
    fn read_padded(&mut self, length: usize) -> io::Result<Vec<u8>>;
    fn read_padded_optional_string(&mut self, length: usize) -> Result<Option<String>, DimeError>;
}

pub(crate) trait WriteExt: Write {
    fn write_u16_be(&mut self, value: u16) -> io::Result<()>;
    fn write_u32_be(&mut self, value: u32) -> io::Result<()>;
    fn write_padded(&mut self, value: &[u8]) -> io::Result<()>;
    fn write_padded_optional_string(&mut self, value: Option<&str>) -> io::Result<()>;
}

impl<R: Read> ReadExt for R {
    fn read_u8(&mut self) -> Result<u8, DimeError> {
        let mut buffer = [0u8; 1];
        self.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    fn read_u16_be(&mut self) -> Result<u16, DimeError> {
        let mut buffer = [0u8; 2];
        self.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    fn read_u32_be(&mut self) -> Result<u32, DimeError> {
        let mut buffer = [0u8; 4];
        self.read_exact(&mut buffer)?;
        Ok(u32::from_be_bytes(buffer))
    }

    fn read_padded(&mut self, length: usize) -> io::Result<Vec<u8>> {
        let padding_length = (4 - length % 4) % 4;

        let mut bytes = vec![0_u8; length];
        self.read_exact(&mut bytes)?;

        let mut padding = [0; 3];
        self.read_exact(&mut padding[..padding_length])?;

        Ok(bytes)
    }

    fn read_padded_optional_string(&mut self, length: usize) -> Result<Option<String>, DimeError> {
        match length {
            0 => Ok(None),
            length => {
                let bytes = self.read_padded(length)?;
                Ok(Some(String::from_utf8(bytes)?))
            }
        }
    }
}

impl<W: Write> WriteExt for W {
    fn write_u16_be(&mut self, value: u16) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_u32_be(&mut self, value: u32) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_padded(&mut self, value: &[u8]) -> io::Result<()> {
        self.write_all(value)?;
        let padding_length = (4 - value.len() % 4) % 4;
        let padding = [0u8; 3];

        self.write_all(&padding[..padding_length])
    }

    fn write_padded_optional_string(&mut self, value: Option<&str>) -> io::Result<()> {
        match value {
            Some(text) => self.write_padded(text.as_bytes()),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    #[test]
    fn read_padded_consumes_expected_number_of_bytes() {
        let cases = [
            (0, 0),
            (1, 4),
            (2, 4),
            (3, 4),
            (4, 4),
            (5, 8),
            (7, 8),
            (8, 8),
            (9, 12),
        ];

        for (length, expected_consumed) in cases {
            let mut input = vec![0xAB; expected_consumed];

            // Marker proving read_padded did not read too far.
            input.push(0xFE);

            let mut cursor = Cursor::new(input);

            let result = cursor.read_padded(length).unwrap();

            assert_eq!(result, vec![0xAB; length]);
            assert_eq!(
                cursor.position() as usize,
                expected_consumed,
                "incorrect bytes consumed for length {length}",
            );

            let mut marker = [0_u8; 1];
            cursor.read_exact(&mut marker).unwrap();

            assert_eq!(marker[0], 0xFE);
        }
    }

    #[test]
    fn read_padded_fails_when_padding_is_incomplete() {
        // Length 5 requires reading 8 bytes.
        let input = vec![0_u8; 7];
        let mut cursor = Cursor::new(input);

        let error = cursor.read_padded(5).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
    }
}
