// SPDX-License-Identifier: MPL-2.0

use crate::connection::XmlaError;
use crate::xmla::soap::{FromXml, get_body_child};
use roxmltree::{Document, Node};
use std::collections::HashMap;

pub(crate) const XMLA_NS: &str = "urn:schemas-microsoft-com:xml-analysis";
const XMLA_ROWSET_NS: &str = "urn:schemas-microsoft-com:xml-analysis:rowset";

#[derive(Debug, Default)]
pub struct XmlaDiscoverResponse {
    rows: Vec<HashMap<String, String>>,
}

pub(crate) fn check_tag_name(node: Node, ns: &str, tag_name: &str) -> Result<(), XmlaError> {
    if !node.has_tag_name((ns, tag_name)) {
        Err(XmlaError::ParsingError(format!(
            "Expected {{{}}}{} node, found: {{{}}}/{}",
            ns,
            tag_name,
            node.tag_name().namespace().unwrap_or(""),
            node.tag_name().name(),
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn get_first_element_child<'a>(
    parent: Node<'a, 'a>,
    ns: &str,
    tag_name: &str,
) -> Result<Node<'a, 'a>, XmlaError> {
    let node = parent.first_element_child().ok_or_else(|| {
        XmlaError::ParsingError(format!("No {{{}}}{} node in XML", ns, tag_name).to_string())
    })?;
    check_tag_name(node, ns, tag_name)?;
    Ok(node)
}

impl XmlaDiscoverResponse {
    pub(crate) fn add_row(&mut self, row: HashMap<String, String>) {
        self.rows.push(row);
    }

    pub fn rows(&self) -> impl Iterator<Item = &HashMap<String, String>> {
        self.rows.iter()
    }
}

impl FromXml for XmlaDiscoverResponse {
    fn from_xml(node: Node) -> Result<Self, XmlaError> {
        check_tag_name(node, XMLA_NS, "DiscoverResponse")?;
        let return_node = get_first_element_child(node, XMLA_NS, "return")?;
        let root_node = get_first_element_child(return_node, XMLA_ROWSET_NS, "root")?;
        let mut response = XmlaDiscoverResponse::default();
        for row in root_node.children().filter(|n| n.is_element()) {
            let values = row
                .children()
                .filter(|n| n.is_element())
                .map(|field| {
                    (
                        field.tag_name().name().to_string(),
                        field.text().unwrap_or_default().to_string(),
                    )
                })
                .collect::<HashMap<String, String>>();
            response.add_row(values);
        }
        Ok(response)
    }
}

pub fn parse_discover_response(xml: &str) -> Result<XmlaDiscoverResponse, XmlaError> {
    let document = Document::parse(xml)?;
    XmlaDiscoverResponse::from_xml(get_body_child(&document)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;

    const TEST_XML: &str = r#"<DiscoverResponse xmlns="urn:schemas-microsoft-com:xml-analysis">
        <return>
            <root xmlns="urn:schemas-microsoft-com:xml-analysis:rowset"
                  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                  xmlns:msxmla="http://schemas.microsoft.com/analysisservices/2003/xmla">
                <row>
                    <CATALOG_NAME>my_catalog</CATALOG_NAME>
                    <DESCRIPTION/>
                </row>
            </root>
        </return>
    </DiscoverResponse>"#;

    #[test]
    fn test_parse_single_row() -> Result<(), Box<dyn std::error::Error>> {
        let document = Document::parse(TEST_XML)?;
        let response = XmlaDiscoverResponse::from_xml(document.root_element())?;
        assert_eq!(response.rows.len(), 1);
        let first_row = response.rows.first();
        assert!(first_row.is_some());
        let catalog_name = first_row.unwrap().get("CATALOG_NAME");
        assert!(catalog_name.is_some());
        assert_eq!("my_catalog", catalog_name.unwrap());
        let description = first_row.unwrap().get("DESCRIPTION");
        assert!(description.is_some());
        assert_eq!("", description.unwrap());
        Ok(())
    }
}
