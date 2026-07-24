// SPDX-License-Identifier: MPL-2.0

#[derive(Debug, thiserror::Error)]
pub enum DimeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported DIME version: {0}")]
    InvalidVersion(u8),

    #[error("invalid DIME type format: {0}")]
    InvalidTypeFormat(u8),

    #[error("invalid DIME options length: {0}")]
    InvalidOptionsLength(u16),

    #[error("invalid DIME options flags: {0:#010b}")]
    InvalidOptionsFlags(u8),

    #[error("DIME options reserved bytes are not zero")]
    InvalidOptionsReservedBytes,

    #[error("invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("DIME field is too large: {0}")]
    FieldTooLarge(&'static str),

    #[error("invalid DIME reserved bits: {0:#06b}")]
    InvalidReservedBits(u8),
}
