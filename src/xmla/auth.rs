// SPDX-License-Identifier: MPL-2.0

use crate::xmla::soap::ToXml;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use std::io;

pub(crate) struct Authenticate {
    pub sspi_handshake: String,
}

impl ToXml for Authenticate {
    fn to_xml(&self, writer: &mut Writer<Vec<u8>>) -> io::Result<()> {
        // <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
        //   <soap:Body>
        //     <Authenticate xmlns="http://schemas.microsoft.com/analysisservices/2003/ext">
        //       <SspiHandshake>{encoded}</SspiHandshake>
        //     </Authenticate>
        //   </soap:Body>
        // </soap:Envelope>
        let mut authenticate = BytesStart::new("Authenticate");
        authenticate.push_attribute((
            "xmlns",
            "http://schemas.microsoft.com/analysisservices/2003/ext",
        ));

        writer.write_event(Event::Start(authenticate))?;
        writer.write_event(Event::Start(BytesStart::new("SspiHandshake")))?;
        writer.write_event(Event::Text(BytesText::new(self.sspi_handshake.as_str())))?;
        writer.write_event(Event::End(BytesEnd::new("SspiHandshake")))?;
        writer.write_event(Event::End(BytesEnd::new("Authenticate")))?;
        Ok(())
    }
}
