// SPDX-License-Identifier: MPL-2.0

use crate::connection::XmlaError;
use quick_xml::{
    Writer,
    events::{BytesEnd, BytesStart, Event},
};
use roxmltree::Node;
use std::io;

const SOAP_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";

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

fn parse_fault(node: &Node) -> XmlaError {
    let fault_code = node
        .children()
        .find(|node| node.is_element() && node.tag_name().name() == "faultcode")
        .and_then(|node| node.text())
        .unwrap_or("Unknown");
    let fault_string = node
        .children()
        .find(|node| node.is_element() && node.tag_name().name() == "faultstring")
        .and_then(|node| node.text())
        .unwrap_or("Unknown");
    XmlaError::ProtocolError(format!("{fault_code}: {fault_string}"))
}

pub(crate) fn get_body_child<'a, 'input: 'a>(
    document: &'a roxmltree::Document<'input>,
) -> crate::connection::Result<Node<'a, 'input>> {
    let body = document
        .descendants()
        .find(|node| node.is_element() && node.has_tag_name((SOAP_NS, "Body")))
        .ok_or_else(|| XmlaError::ProtocolError("No body found".to_string()))?;
    let child = body
        .children()
        .find(|node| node.is_element())
        .ok_or_else(|| XmlaError::ProtocolError("Empty body".to_string()))?;
    if child.has_tag_name((SOAP_NS, "Fault")) {
        return Err(parse_fault(&child));
    }
    Ok(child)
}
