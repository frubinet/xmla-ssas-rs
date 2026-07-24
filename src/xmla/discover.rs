// SPDX-License-Identifier: MPL-2.0

use crate::xmla::props::XmlaProperties;
use crate::xmla::soap::ToXml;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use std::io;

pub struct XmlaDiscover {
    pub request_type: String,
    pub properties: XmlaProperties,
}

impl XmlaDiscover {
    pub fn new(request_type: impl Into<String>) -> Self {
        Self {
            request_type: request_type.into(),
            properties: XmlaProperties::default(),
        }
    }
}

impl ToXml for XmlaDiscover {
    fn to_xml(&self, writer: &mut Writer<Vec<u8>>) -> io::Result<()> {
        //     <Discover xmlns="urn:schemas-microsoft-com:xml-analysis">
        //       <RequestType>DBSCHEMA_CATALOGS</RequestType>
        //       <Restrictions>
        //         <RestrictionList/>
        //       </Restrictions>
        //       <Properties>
        //         <PropertyList/>
        //       </Properties>
        //     </Discover>
        let mut discover = BytesStart::new("Discover");
        discover.push_attribute(("xmlns", "urn:schemas-microsoft-com:xml-analysis"));
        writer.write_event(Event::Start(discover))?;
        writer.write_event(Event::Start(BytesStart::new("RequestType")))?;
        writer.write_event(Event::Text(BytesText::new(self.request_type.as_str())))?;
        writer.write_event(Event::End(BytesEnd::new("RequestType")))?;
        writer.write_event(Event::Start(BytesStart::new("Restrictions")))?;
        writer.write_event(Event::Start(BytesStart::new("RestrictionList")))?;
        writer.write_event(Event::End(BytesEnd::new("RestrictionList")))?;
        writer.write_event(Event::End(BytesEnd::new("Restrictions")))?;
        self.properties.to_xml(writer)?;
        writer.write_event(Event::End(BytesEnd::new("Discover")))?;
        Ok(())
    }
}
