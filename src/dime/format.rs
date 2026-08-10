// SPDX-License-Identifier: MPL-2.0

use crate::dime::error::DimeError;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeFormat {
    /// TYPE_T = 0x00
    Unchanged = 0x00,
    /// TYPE_T = 0x01
    MediaType = 0x01,
    /// TYPE_T = 0x02
    AbsoluteUri = 0x02,
    /// TYPE_T = 0x03
    Unknown = 0x03,
    /// TYPE_T = 0x04
    NoType = 0x04,
}

impl TryFrom<u8> for TypeFormat {
    type Error = DimeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(TypeFormat::Unchanged),
            0x01 => Ok(TypeFormat::MediaType),
            0x02 => Ok(TypeFormat::AbsoluteUri),
            0x03 => Ok(TypeFormat::Unknown),
            0x04 => Ok(TypeFormat::NoType),
            _ => Err(DimeError::InvalidTypeFormat(value)),
        }
    }
}

impl fmt::Display for TypeFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Unchanged => "Unchanged",
            Self::MediaType => "MediaType",
            Self::AbsoluteUri => "AbsoluteUri",
            Self::Unknown => "Unknown",
            Self::NoType => "NoType",
        };

        f.write_str(name)
    }
}
