// SPDX-License-Identifier: MPL-2.0

use crate::dime::DimeError;
use std::{io, str::Utf8Error};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum XmlaError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("DIME protocol error: {0}")]
    Dime(#[from] DimeError),

    #[error("SSPI authentication error: {0}")]
    Sspi(#[from] sspi::Error),

    #[error("invalid UTF-8: {0}")]
    Utf8(#[from] Utf8Error),

    #[error("XML parsing error: {0}")]
    Xml(#[from] roxmltree::Error),

    #[error("invalid username: {0}")]
    InvalidUsername(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("protocol error: {0}")]
    ProtocolError(String),

    #[error("parsing error: {0}")]
    ParsingError(String),
}

pub type Result<T> = std::result::Result<T, XmlaError>;
