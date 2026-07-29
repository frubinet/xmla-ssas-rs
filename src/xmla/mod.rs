// SPDX-License-Identifier: MPL-2.0

//! XMLA request models and SOAP serialization.
mod auth;
mod discover;
mod props;
mod responses;
mod restrictions;
mod soap;

pub(crate) use self::auth::Authenticate;
pub(crate) use soap::FromXml;

pub use self::discover::XmlaDiscover;
pub use self::props::XmlaProperties;
pub use self::responses::XmlaDiscoverResponse;
pub use self::restrictions::XmlaRestrictions;
pub use self::soap::{ToSoap, XmlaOperationContent};
