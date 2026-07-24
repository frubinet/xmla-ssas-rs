//! XMLA request models and SOAP serialization.
mod auth;
mod discover;
mod props;
mod soap;

pub(crate) use self::auth::Authenticate;

pub use self::discover::XmlaDiscover;
pub use self::soap::{
    ToSoap,
    XmlaOperationContent,
};
pub use self::props::XmlaProperties;