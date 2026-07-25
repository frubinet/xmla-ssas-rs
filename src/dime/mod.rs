// SPDX-License-Identifier: MPL-2.0

//! DIME framing used by the SSAS TCP transport.
mod error;
mod format;
mod io;
mod message;
mod options;
mod record;

pub(crate) use error::DimeError;
pub(crate) use message::DimeMessage;
pub(crate) use options::DimeOptions;
