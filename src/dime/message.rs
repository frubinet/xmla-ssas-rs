// SPDX-License-Identifier: MPL-2.0

use crate::dime::error::DimeError;
use crate::dime::format::TypeFormat;
use crate::dime::options::DimeOptions;
use crate::dime::record::DimeRecord;
use log::debug;
use std::io::{Read, Write};

const CHUNK_SIZE: usize = 4096;
const MAX_DECOMPRESSED_CHUNK_SIZE: u32 = 1024 * 1024; // 1 MiB
const MAX_DECOMPRESSED_MESSAGE_SIZE: usize = 50 * 1024 * 1024; // 50 MiB

pub struct DimeMessage {
    pub options: Option<DimeOptions>,
    pub content_type: String,
    pub data: Vec<u8>,
}

impl DimeMessage {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, DimeError> {
        let mut data = Vec::new();
        let mut options: Option<DimeOptions> = None;
        let mut content_type: Option<String> = None;
        let mut first = true;
        loop {
            let record = DimeRecord::read_from(reader)?;
            validate_record(&record, first)?;
            if first {
                first = false;
            }
            if options.is_none() {
                options = record.options;
            }
            if content_type.is_none() {
                content_type = record.type_value;
            }
            data.extend(record.data);
            if record.is_last_record {
                break;
            }
        }
        let content_type = content_type
            .ok_or_else(|| DimeError::RecordFormatError("Content type not found".to_string()))?;
        Ok(DimeMessage {
            options,
            content_type,
            data,
        })
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), DimeError> {
        if self.content_type.is_empty() {
            return Err(DimeError::RecordFormatError(
                "Empty content type".to_string(),
            ));
        }
        let chunks = self
            .data
            .chunks(CHUNK_SIZE)
            .chain(self.data.is_empty().then_some(&[][..]));
        let chunk_count = self.data.len().div_ceil(CHUNK_SIZE).max(1);

        for (index, chunk) in chunks.enumerate() {
            let record = DimeRecord {
                is_first_record: index == 0,
                is_last_record: index == (chunk_count - 1),
                has_next_chunk: index < (chunk_count - 1),
                type_format: if index != 0 {
                    TypeFormat::Unchanged
                } else {
                    TypeFormat::MediaType
                },
                id: None,
                options: if index == 0 {
                    self.options.clone()
                } else {
                    None
                },
                type_value: (index == 0).then(|| self.content_type.clone()),
                data: chunk.to_vec(),
            };
            record.write_to(writer)?;
        }
        Ok(())
    }

    pub fn is_compressed(&self) -> bool {
        self.content_type.ends_with("+xpress")
    }
}

fn validate_record(record: &DimeRecord, first: bool) -> Result<(), DimeError> {
    if first {
        if !record.is_first_record {
            return Err(DimeError::OutOfOrder("first record expected"));
        }
        if record.type_format != TypeFormat::MediaType {
            return Err(DimeError::UnexpectedTypeFormat(record.type_format));
        }
        if record.type_value.is_none() {
            return Err(DimeError::RecordFormatError(
                "Expected content type".to_string(),
            ));
        }
    } else {
        if record.is_first_record {
            return Err(DimeError::OutOfOrder("first record not expected"));
        }
        if record.type_format != TypeFormat::Unchanged {
            return Err(DimeError::UnexpectedTypeFormat(record.type_format));
        }
        if let Some(record_type_value) = record.type_value.as_ref() {
            return Err(DimeError::UnexpectedContentType(record_type_value.clone()));
        }
        if let Some(record_id) = record.id.as_ref() {
            return Err(DimeError::RecordFormatError(format!(
                "Unexpected ID: {record_id}"
            )));
        }
    }
    if record.is_last_record == record.has_next_chunk {
        return Err(DimeError::InconsistentRecordFlags(
            "Inconsistent record flags: ME=CF",
        ));
    }
    Ok(())
}

pub fn decompress(data: &[u8], content_type: &str) -> Result<Vec<u8>, DimeError> {
    if !content_type.ends_with("+xpress") {
        return Err(DimeError::RecordFormatError(
            "Message not compressed".to_string(),
        ));
    }
    if data.is_empty() {
        return Err(DimeError::RecordFormatError(
            "Compressed input too short: 0".into(),
        ));
    }
    let data_len = data.len();
    debug!("Decompressing {}, length: {}", content_type, data_len);
    let mut decompressed = Vec::new();
    let mut start: usize = 0;
    let mut chunk_count = 0;
    while start < data_len {
        let (decompressed_chunk, consumed) = decompress_chunk(data, start)?;
        debug!(
            "Decompressed chunk #{} {} -> {} bytes",
            chunk_count + 1,
            consumed,
            decompressed_chunk.len()
        );
        let new_size = decompressed
            .len()
            .checked_add(decompressed_chunk.len())
            .ok_or_else(|| {
                DimeError::RecordFormatError("Decompressed message size overflow".into())
            })?;

        if new_size > MAX_DECOMPRESSED_MESSAGE_SIZE {
            return Err(DimeError::RecordFormatError(
                "Decompressed message is too large".into(),
            ));
        }
        decompressed.extend(decompressed_chunk);
        start += consumed;
        chunk_count += 1;
    }
    debug!(
        "Decompressed {} bytes in {} chunk(s)",
        decompressed.len(),
        chunk_count
    );
    Ok(decompressed)
}

fn decompress_chunk(data: &[u8], start: usize) -> Result<(Vec<u8>, usize), DimeError> {
    if data.len() - start < 8 {
        return Err(DimeError::RecordFormatError(
            format!("Compressed input too short: {}", data.len()).to_string(),
        ));
    }
    let original_size_bytes = &data[start..start + 4];
    let compressed_size_bytes = &data[start + 4..start + 8];
    let original_size = u32::from_le_bytes(original_size_bytes.try_into().unwrap());
    if original_size > MAX_DECOMPRESSED_CHUNK_SIZE {
        return Err(DimeError::RecordFormatError(
            "Decompressed block size is too large".to_string(),
        ));
    }
    let compressed_size = u32::from_le_bytes(compressed_size_bytes.try_into().unwrap());
    let payload_start = start
        .checked_add(8)
        .ok_or_else(|| DimeError::RecordFormatError("Compressed offset overflow".into()))?;
    let payload_end = payload_start
        .checked_add(compressed_size as usize)
        .ok_or_else(|| DimeError::RecordFormatError("Compressed size overflow".into()))?;
    let compressed_bytes = data
        .get(payload_start..payload_end)
        .ok_or_else(|| DimeError::RecordFormatError("Truncated compressed chunk".to_string()))?;
    let decompressed_buffer = decompress_buffer(compressed_bytes, original_size)?;
    Ok((decompressed_buffer, payload_end - start))
}

fn decompress_buffer(input_buffer: &[u8], size: u32) -> Result<Vec<u8>, DimeError> {
    let output_buffer_size = size as usize;
    let mut output_buffer = vec![0; output_buffer_size];
    let mut kind_bit: i8 = 0;
    let mut have_nibble = false;
    let mut output_buffer_index: usize = 0;
    let mut input_buffer_index: usize = 0;
    let mut kind: u32 = 0;
    let mut nibble_value: u8 = 0;
    while output_buffer_index < output_buffer_size {
        if kind_bit == 0 {
            let kind_bytes = input_buffer
                .get(input_buffer_index..input_buffer_index + 4)
                .ok_or_else(|| DimeError::RecordFormatError("Truncated kind bitmap".into()))?;
            kind = u32::from_le_bytes(kind_bytes.try_into().unwrap());
            input_buffer_index += 4;
            kind_bit = 32;
        }
        kind_bit -= 1;
        if kind & (1u32 << kind_bit) == 0 {
            output_buffer[output_buffer_index] = *input_buffer
                .get(input_buffer_index)
                .ok_or_else(|| DimeError::RecordFormatError("Truncated literal".into()))?;
            input_buffer_index += 1;
            output_buffer_index += 1;
        } else {
            let length_bytes = input_buffer
                .get(input_buffer_index..input_buffer_index + 2)
                .ok_or_else(|| DimeError::RecordFormatError("Truncated length bytes".into()))?;
            let mut length: u32 = u16::from_le_bytes(length_bytes.try_into().unwrap()) as u32;
            input_buffer_index += 2;
            let offset = length >> 3;
            length &= 7;
            if length == 7 {
                if !have_nibble {
                    have_nibble = true;
                    nibble_value = *input_buffer.get(input_buffer_index).ok_or_else(|| {
                        DimeError::RecordFormatError("Truncated match length nibble".into())
                    })?;
                    length = nibble_value as u32 & 15;
                    input_buffer_index += 1;
                } else {
                    length = nibble_value as u32 >> 4;
                    have_nibble = false;
                }
                if length == 15 {
                    length = *input_buffer.get(input_buffer_index).ok_or_else(|| {
                        DimeError::RecordFormatError("Truncated extended match length".into())
                    })? as u32;
                    input_buffer_index += 1;
                    if length == 255 {
                        let length_bytes = input_buffer
                            .get(input_buffer_index..input_buffer_index + 2)
                            .ok_or_else(|| {
                                DimeError::RecordFormatError("Truncated length bytes".into())
                            })?;
                        length = u16::from_le_bytes(length_bytes.try_into().unwrap()) as u32;
                        input_buffer_index += 2;
                        length = length.checked_sub(22).ok_or_else(|| {
                            DimeError::RecordFormatError("Invalid match length".into())
                        })?;
                    }
                    length += 15;
                }
                length += 7;
            }
            length += 3;
            let remaining = output_buffer_size - output_buffer_index;
            if length as usize > remaining {
                return Err(DimeError::RecordFormatError("Invalid length".into()));
            }
            let distance = offset as usize + 1;
            if distance > output_buffer_index {
                return Err(DimeError::RecordFormatError("Invalid offset".into()));
            }
            while length != 0 {
                output_buffer[output_buffer_index] = output_buffer[output_buffer_index - distance];
                output_buffer_index += 1;
                length -= 1;
            }
        }
    }
    Ok(output_buffer)
}

//TODO: add tests for different sizes, for example: 0, 4095, 4096, and 4097
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        let data = [];
        let error = decompress(&data, "application/xml+xpress").unwrap_err();
        assert_eq!(error.to_string(), "Compressed input too short: 0");
    }

    #[test]
    fn compressed_input_too_short() {
        let data = [0_u8; 1];
        let error = decompress(&data, "application/xml+xpress").unwrap_err();
        assert_eq!(error.to_string(), "Compressed input too short: 1");
    }

    #[test]
    fn decompressed_block_size_exceeded() {
        let mut data = Vec::new();
        data.extend(u32::to_le_bytes(5 * 1024 * 1024)); // original size
        data.extend(u32::to_le_bytes(1024 * 1024)); // compressed size

        let error = decompress(&data, "application/xml+xpress").unwrap_err();
        assert_eq!(error.to_string(), "Decompressed block size is too large");
    }

    #[test]
    fn decompresses_literal_block() {
        let mut data = Vec::new();
        data.extend_from_slice(&3u32.to_le_bytes()); // original size
        data.extend_from_slice(&7u32.to_le_bytes()); // compressed size
        // Zero bitmap: each token used here is a literal.
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(b"abc");

        assert_eq!(decompress(&data, "application/xml+xpress").unwrap(), b"abc");
    }

    #[test]
    fn decompresses_back_reference() {
        let mut data = Vec::new();
        data.extend_from_slice(&6u32.to_le_bytes()); // original size
        data.extend_from_slice(&9u32.to_le_bytes()); // compressed size

        // Three literals followed by one match: bit 28 is set.
        data.extend_from_slice(&0x1000_0000u32.to_le_bytes());
        data.extend_from_slice(b"abc");

        // Distance = stored offset 2 + 1 = 3.
        // Length = stored length 0 + 3 = 3.
        data.extend_from_slice(&0x0010u16.to_le_bytes());

        assert_eq!(
            decompress(&data, "application/xml+xpress").unwrap(),
            b"abcabc"
        );
    }
}
