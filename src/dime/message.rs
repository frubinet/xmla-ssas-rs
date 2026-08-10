// SPDX-License-Identifier: MPL-2.0

use crate::dime::error::DimeError;
use crate::dime::format::TypeFormat;
use crate::dime::options::DimeOptions;
use crate::dime::record::DimeRecord;
use std::io::{Read, Write};

const CHUNK_SIZE: usize = 4096;

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

//TODO: add tests for different sizes, for example: 0, 4095, 4096, and 4097
