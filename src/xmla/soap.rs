// SPDX-License-Identifier: MPL-2.0

use crate::connection::XmlaError;
use quick_xml::{
    Writer,
    events::{BytesEnd, BytesStart, Event},
};
use roxmltree::Node;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlaOperationContent {
    /// Validates the structure without executing.
    None,
    /// Schema only.
    Schema,
    /// Requested rows only.
    Data,
    /// Schema and rows; default.
    SchemaData,
    /// Schema plus multidimensional OlapInfo.
    Metadata,
}

impl XmlaOperationContent {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Schema => "Schema",
            Self::Data => "Data",
            Self::SchemaData => "SchemaData",
            Self::Metadata => "Metadata",
        }
    }
}

pub(crate) trait ToXml {
    fn to_xml(&self, writer: &mut Writer<Vec<u8>>) -> io::Result<()>;
}

pub(crate) trait FromXml: Sized {
    fn from_xml(node: Node) -> Result<Self, XmlaError>;
}

pub trait ToSoap {
    fn to_soap(&self) -> io::Result<Vec<u8>>;
}

impl<T: ToXml> ToSoap for T {
    fn to_soap(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());

        let mut envelope = BytesStart::new("soap:Envelope");
        envelope.push_attribute(("xmlns:soap", "http://schemas.xmlsoap.org/soap/envelope/"));

        writer.write_event(Event::Start(envelope))?;
        writer.write_event(Event::Start(BytesStart::new("soap:Body")))?;
        self.to_xml(&mut writer)?;
        writer.write_event(Event::End(BytesEnd::new("soap:Body")))?;
        writer.write_event(Event::End(BytesEnd::new("soap:Envelope")))?;

        Ok(writer.into_inner())
    }
}
