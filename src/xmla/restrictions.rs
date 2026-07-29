// SPDX-License-Identifier: MPL-2.0

use crate::connection::XmlaError;
use crate::xmla::soap::ToXml;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use regex::Regex;
use std::collections::HashMap;
use std::io;
use std::sync::LazyLock;

static XML_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z_][A-Za-z0-9_.-]*$").unwrap());

#[derive(Debug, Default, Clone)]
pub struct XmlaRestrictions {
    values: HashMap<String, String>,
}

fn is_valid_xml_name(name: &str) -> bool {
    XML_NAME_REGEX.is_match(name)
}

impl XmlaRestrictions {
    pub fn add(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), XmlaError> {
        let key = key.into();

        if !is_valid_xml_name(&key) {
            return Err(XmlaError::SerializationError(format!(
                "Invalid XML restriction name: {key:?}"
            )));
        }
        self.values.insert(key, value.into());
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}

impl ToXml for XmlaRestrictions {
    fn to_xml(&self, writer: &mut Writer<Vec<u8>>) -> io::Result<()> {
        writer.write_event(Event::Start(BytesStart::new("Restrictions")))?;
        writer.write_event(Event::Start(BytesStart::new("RestrictionList")))?;
        for (key, value) in self.entries() {
            writer.write_event(Event::Start(BytesStart::new(key)))?;
            writer.write_event(Event::Text(BytesText::new(value)))?;
            writer.write_event(Event::End(BytesEnd::new(key)))?;
        }
        writer.write_event(Event::End(BytesEnd::new("RestrictionList")))?;
        writer.write_event(Event::End(BytesEnd::new("Restrictions")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_restriction_name() {
        let mut restrictions = XmlaRestrictions::default();

        assert!(restrictions.add("invalid key", "value").is_err());
    }
}
