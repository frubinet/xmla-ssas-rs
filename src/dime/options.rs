use crate::dime::error::DimeError;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct DimeOptions {
    /// NEGO
    pub is_negotiated: bool,
    /// REQ_SX
    pub is_request_xml_binary: bool,
    /// REQ_XPRESS
    pub is_request_compressed: bool,
    /// RESP_SX
    pub is_response_xml_binary: bool,
    /// RESP_XPRESS
    pub is_response_compressed: bool,
}

impl DimeOptions {
    pub(crate) fn read_from<R: Read>(
        reader: &mut R,
        options_length: u16,
    ) -> Result<Self, DimeError> {
        if options_length != 4 {
            return Err(DimeError::InvalidOptionsLength(options_length));
        }
        let mut options_buffer = [0u8; 4];
        reader.read_exact(&mut options_buffer)?;
        let options_flags = options_buffer[0];

        if options_flags & 0b1110_0000 != 0 {
            return Err(DimeError::InvalidOptionsFlags(options_flags));
        }
        if options_buffer[1..4] != [0, 0, 0] {
            return Err(DimeError::InvalidOptionsReservedBytes);
        }

        Ok(DimeOptions {
            is_negotiated: options_flags & 0b0000_0001 != 0,
            is_request_xml_binary: options_flags & 0b0000_0010 != 0,
            is_request_compressed: options_flags & 0b0000_0100 != 0,
            is_response_xml_binary: options_flags & 0b0000_1000 != 0,
            is_response_compressed: options_flags & 0b0001_0000 != 0,
        })
    }

    pub(crate) fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), DimeError> {
        let mut options_flags = 0u8;
        if self.is_negotiated {
            options_flags |= 0b0000_0001;
        }
        if self.is_request_xml_binary {
            options_flags |= 0b0000_0010;
        }
        if self.is_request_compressed {
            options_flags |= 0b0000_0100;
        }
        if self.is_response_xml_binary {
            options_flags |= 0b0000_1000;
        }
        if self.is_response_compressed {
            options_flags |= 0b0001_0000;
        }
        writer.write_all(&[options_flags, 0, 0, 0])?;
        Ok(())
    }
}
