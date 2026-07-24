use crate::xmla::soap::{ToXml, XmlaOperationContent};
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use std::io;

#[derive(Debug, Default)]
pub struct XmlaProperties {
    pub content: Option<XmlaOperationContent>,
}

impl ToXml for XmlaProperties {
    fn to_xml(&self, writer: &mut Writer<Vec<u8>>) -> io::Result<()> {
        writer.write_event(Event::Start(BytesStart::new("Properties")))?;
        writer.write_event(Event::Start(BytesStart::new("PropertyList")))?;
        if let Some(content) = &self.content {
            writer.write_event(Event::Start(BytesStart::new("Content")))?;
            writer.write_event(Event::Text(BytesText::new(content.as_str())))?;
            writer.write_event(Event::End(BytesEnd::new("Content")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("PropertyList")))?;
        writer.write_event(Event::End(BytesEnd::new("Properties")))?;
        Ok(())
    }
}
