// SPDX-License-Identifier: MPL-2.0

use crate::dime::error::DimeError;
use crate::dime::format::TypeFormat;
use crate::dime::io::{ReadExt, WriteExt};
use crate::dime::options::DimeOptions;
use std::fmt;
use std::io::Read;
use std::io::Write;

pub(crate) struct DimeRecord {
    /// DIME MB (Message Begin) flag
    pub is_first_record: bool,
    /// DIME ME (Message End) flag
    pub is_last_record: bool,
    /// DIME CF (Chunk Flag)
    pub has_next_chunk: bool,
    /// DIME TYPE_T field
    pub type_format: TypeFormat,

    /// Dime ID field
    pub id: Option<String>,
    /// DIME options field
    pub options: Option<DimeOptions>,
    /// Value of the DIME TYPE field.
    pub type_value: Option<String>,
    /// DIME data field
    pub data: Vec<u8>,
}

impl DimeRecord {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, DimeError> {
        let first_header_byte = reader.read_u8()?;
        let version = first_header_byte >> 3;
        if version != 1 {
            return Err(DimeError::InvalidVersion(version));
        }
        let message_begin = first_header_byte & 0b0000_0100 != 0;
        let message_end = first_header_byte & 0b0000_0010 != 0;
        let next_chunk = first_header_byte & 0b0000_0001 != 0;

        let second_header_byte = reader.read_u8()?;
        let reserved = second_header_byte & 0b0000_1111;
        if reserved != 0 {
            return Err(DimeError::InvalidReservedBits(reserved));
        }
        let type_format = TypeFormat::try_from(second_header_byte >> 4)?;

        let options_length = reader.read_u16_be()?;
        let id_length = reader.read_u16_be()?;
        let type_length = reader.read_u16_be()?;
        let data_length = reader.read_u32_be()?;
        let data_length =
            usize::try_from(data_length).map_err(|_| DimeError::FieldTooLarge("DATA"))?;

        let options = match options_length {
            0 => None,
            length => Some(DimeOptions::read_from(reader, length)?),
        };
        let id = reader.read_padded_optional_string(usize::from(id_length))?;
        let type_value = reader.read_padded_optional_string(usize::from(type_length))?;
        let data = reader.read_padded(data_length)?;

        Ok(DimeRecord {
            is_first_record: message_begin,
            is_last_record: message_end,
            has_next_chunk: next_chunk,
            type_format,
            options,
            id,
            type_value,
            data,
        })
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), DimeError> {
        let id_length = match &self.id {
            Some(id) => u16::try_from(id.len()).map_err(|_| DimeError::FieldTooLarge("ID"))?,
            None => 0,
        };
        let type_length = match &self.type_value {
            Some(value) => {
                u16::try_from(value.len()).map_err(|_| DimeError::FieldTooLarge("TYPE"))?
            }
            None => 0,
        };
        let data_length =
            u32::try_from(self.data.len()).map_err(|_| DimeError::FieldTooLarge("DATA"))?;

        let mut first_header_byte = 0b0000_1000u8; // VERSION = 1
        if self.is_first_record {
            first_header_byte |= 0b0000_0100;
        }
        if self.is_last_record {
            first_header_byte |= 0b0000_0010;
        }
        if self.has_next_chunk {
            first_header_byte |= 0b0000_0001;
        }
        let second_header_byte = (self.type_format as u8) << 4;
        writer.write_all(&[first_header_byte, second_header_byte])?;
        writer.write_u16_be(if self.options.is_some() { 4 } else { 0 })?; // options_length

        writer.write_u16_be(id_length)?;
        writer.write_u16_be(type_length)?;
        writer.write_u32_be(data_length)?;
        if let Some(options) = &self.options {
            options.write_to(writer)?;
        }
        writer.write_padded_optional_string(self.id.as_deref())?;
        writer.write_padded_optional_string(self.type_value.as_deref())?;
        writer.write_padded(&self.data)?;
        Ok(())
    }
}

impl fmt::Debug for DimeRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DimeRecord")
            .field("is_first_record", &self.is_first_record)
            .field("is_last_record", &self.is_last_record)
            .field("has_next_chunk", &self.has_next_chunk)
            .field("type_format", &self.type_format)
            .field("options", &self.options)
            .field("type_value", &self.type_value)
            .field("id", &self.id)
            .field("data_length", &self.data.len())
            .finish()
    }
}
