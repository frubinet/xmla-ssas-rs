// SPDX-License-Identifier: MPL-2.0

use crate::dime::error::DimeError;
use crate::dime::format::TypeFormat;
use crate::dime::options::DimeOptions;
use crate::dime::record::DimeRecord;
use std::io::{Read, Write};

const CHUNK_SIZE: usize = 4096;

pub struct DimeMessage {
    pub options: Option<DimeOptions>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

impl DimeMessage {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, DimeError> {
        let mut data = Vec::new();
        let mut options: Option<DimeOptions> = None;
        let mut content_type: Option<String> = None;
        // TODO: validate record sequence (is_first_record, is_last_record, etc)
        loop {
            let record = DimeRecord::read_from(reader)?;
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
        Ok(DimeMessage {
            options,
            content_type,
            data,
        })
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), DimeError> {
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
                } else if self.content_type.is_some() {
                    TypeFormat::MediaType
                } else {
                    TypeFormat::NoType
                },
                id: None,
                options: if index == 0 {
                    self.options.clone()
                } else {
                    None
                },
                type_value: if index == 0 && self.content_type.is_some() {
                    self.content_type.clone()
                } else {
                    None
                },
                data: chunk.to_vec(),
            };
            record.write_to(writer)?;
        }
        Ok(())
    }
}

//TODO: add tests for different sizes, for example: 0, 4095, 4096, and 4097
