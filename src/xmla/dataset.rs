// SPDX-License-Identifier: MPL-2.0

use crate::connection::XmlaError;
use crate::xmla::FromXml;
use crate::xmla::soap::get_body_child;
use roxmltree::{Document, Node};
use std::collections::HashMap;

const XMLA_DATASET_NS: &str = "urn:schemas-microsoft-com:xml-analysis:mddataset";

#[derive(Debug, Default)]
pub struct XmlaDataset {
    axes: Vec<XmlaAxis>,
    cells: Vec<XmlaCell>,
    cells_map: HashMap<u32, usize>,
}

#[derive(Debug)]
pub struct XmlaAxis {
    name: String,
    tuples: Vec<XmlaTuple>,
}

#[derive(Debug)]
pub struct XmlaTuple {
    members: Vec<XmlaMember>,
}

#[derive(Debug)]
pub struct XmlaMember {
    hierarchy: String,
    unique_name: String,
    caption: String,
    level_name: String,
    level_number: i32,
    display_info: u32,
}

/// A cell from an XMLA multidimensional dataset.
///
/// Only numeric `Value` properties are currently supported. Integer and
/// floating-point XML schema values are exposed as `f64`. Missing or null
/// values are represented as [`f64::NAN`]. Nonnumeric values cause parsing
/// to fail.
#[derive(Debug, Clone)]
pub struct XmlaCell {
    ordinal: u32,
    value: f64,
    formatted_value: String,
}

fn string_child(node: Node, ns: &str, name: &str) -> Result<String, XmlaError> {
    Ok(node
        .children()
        .find(|n| n.is_element() && n.has_tag_name((ns, name)))
        .ok_or_else(|| XmlaError::ParsingError(format!("No {name} found for member")))?
        .text()
        .ok_or_else(|| XmlaError::ParsingError(format!("Empty {name} in member")))?
        .to_string())
}

fn optional_string_child(node: Node, ns: &str, name: &str) -> Option<String> {
    node.children()
        .find(|child| child.is_element() && child.has_tag_name((ns, name)))
        .and_then(|child| child.text())
        .map(str::to_owned)
}

impl XmlaDataset {
    pub fn axes(&self) -> &[XmlaAxis] {
        &self.axes
    }

    pub fn axis(&self, index: usize) -> Option<&XmlaAxis> {
        self.axes.get(index)
    }

    pub fn cells(&self) -> &[XmlaCell] {
        &self.cells
    }

    pub fn cell(&self, ordinal: u32) -> Option<&XmlaCell> {
        self.cells_map
            .get(&ordinal)
            .and_then(|&index| self.cells.get(index))
    }

    pub fn row_count(&self) -> usize {
        self.axes.get(1).map_or(0, |axis| axis.tuples.len())
    }

    pub fn column_count(&self) -> usize {
        self.axes.first().map_or(0, |axis| axis.tuples.len())
    }

    /// Helper method for two-dimensional results, use cell(ordinal) for other results
    pub fn cell_at(&self, column: u32, row: u32) -> Option<&XmlaCell> {
        let column_count = u32::try_from(self.column_count()).ok()?;
        let row_count = u32::try_from(self.row_count()).ok()?;
        if column >= column_count || row >= row_count {
            return None;
        }
        let ordinal = row.checked_mul(column_count)?.checked_add(column)?;
        self.cells_map
            .get(&ordinal)
            .and_then(|&index| self.cells.get(index))
    }

    pub fn cell_value_at(&self, column: u32, row: u32) -> Option<f64> {
        Some(self.cell_at(column, row)?.value)
    }

    pub fn cell_formatted_value_at(&self, column: u32, row: u32) -> Option<&str> {
        Some(self.cell_at(column, row)?.formatted_value.as_str())
    }
}

impl FromXml for XmlaDataset {
    fn from_xml(node: Node) -> Result<Self, XmlaError> {
        let return_node = crate::xmla::responses::get_first_element_child(
            node,
            crate::xmla::responses::XMLA_NS,
            "return",
        )?;
        let root_node =
            crate::xmla::responses::get_first_element_child(return_node, XMLA_DATASET_NS, "root")?;
        let axes = root_node
            .children()
            .find(|n| n.is_element() && n.has_tag_name((XMLA_DATASET_NS, "Axes")))
            .ok_or_else(|| {
                XmlaError::ParsingError("Axes element expected in dataset".to_string())
            })?;
        let axes = axes
            .children()
            .filter(|n| n.has_tag_name((XMLA_DATASET_NS, "Axis")))
            .map(|n| XmlaAxis::from_xml(n))
            .collect::<Result<Vec<XmlaAxis>, XmlaError>>()?;
        let cell_data = root_node
            .children()
            .find(|n| n.is_element() && n.has_tag_name((XMLA_DATASET_NS, "CellData")))
            .ok_or_else(|| {
                XmlaError::ParsingError("CellData element expected in dataset".to_string())
            })?;
        let cells = cell_data
            .children()
            .filter(|n| n.has_tag_name((XMLA_DATASET_NS, "Cell")))
            .map(|n| XmlaCell::from_xml(n))
            .collect::<Result<Vec<XmlaCell>, XmlaError>>()?;
        let mut cells_map: HashMap<u32, usize> = HashMap::new();
        for (index, cell) in cells.iter().enumerate() {
            let ordinal = cell.ordinal;
            if cells_map.contains_key(&ordinal) {
                return Err(XmlaError::ParsingError(format!(
                    "Duplicate cell ordinal {}",
                    ordinal
                )));
            }
            cells_map.insert(ordinal, index);
        }
        Ok(Self {
            axes,
            cells,
            cells_map,
        })
    }
}

impl XmlaAxis {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn tuples(&self) -> &[XmlaTuple] {
        &self.tuples
    }
}

impl FromXml for XmlaAxis {
    fn from_xml(node: Node) -> Result<Self, XmlaError> {
        crate::xmla::responses::check_tag_name(node, XMLA_DATASET_NS, "Axis")?;
        let name = node
            .attribute("name")
            .ok_or_else(|| XmlaError::ParsingError("Expected name attribute in axis".to_string()))?
            .to_string();
        let tuples = node
            .children()
            .find(|n| n.is_element() && n.has_tag_name((XMLA_DATASET_NS, "Tuples")))
            .ok_or_else(|| {
                XmlaError::ParsingError(format!("Tuples element expected in axis {name}"))
            })?;
        let tuples = tuples
            .children()
            .filter(|node| node.is_element())
            .map(|node| XmlaTuple::from_xml(node));
        Ok(Self {
            name,
            tuples: tuples.collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl XmlaTuple {
    pub fn members(&self) -> &[XmlaMember] {
        &self.members
    }
}

impl FromXml for XmlaTuple {
    fn from_xml(node: Node) -> Result<Self, XmlaError> {
        crate::xmla::responses::check_tag_name(node, XMLA_DATASET_NS, "Tuple")?;
        let members = node
            .children()
            .filter(|node| node.is_element())
            .map(|node| XmlaMember::from_xml(node));
        Ok(Self {
            members: members.collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl XmlaMember {
    pub fn hierarchy(&self) -> &str {
        &self.hierarchy
    }
    pub fn unique_name(&self) -> &str {
        &self.unique_name
    }
    pub fn caption(&self) -> &str {
        &self.caption
    }
    pub fn level_name(&self) -> &str {
        &self.level_name
    }
    pub fn level_number(&self) -> i32 {
        self.level_number
    }
    pub fn display_info(&self) -> u32 {
        self.display_info
    }
}

impl FromXml for XmlaMember {
    fn from_xml(node: Node) -> Result<Self, XmlaError> {
        crate::xmla::responses::check_tag_name(node, XMLA_DATASET_NS, "Member")?;
        let hierarchy = node
            .attribute("Hierarchy")
            .ok_or_else(|| {
                XmlaError::ParsingError("Expected Hierarchy attribute in member".to_string())
            })?
            .to_string();
        let unique_name = string_child(node, XMLA_DATASET_NS, "UName")?;
        let caption = string_child(node, XMLA_DATASET_NS, "Caption")?;
        let level_name = string_child(node, XMLA_DATASET_NS, "LName")?;
        let level_number = string_child(node, XMLA_DATASET_NS, "LNum")?;
        let level_number = level_number.parse::<i32>().map_err(|error| {
            XmlaError::ParsingError(format!("Invalid numeric value {level_number:?}: {error}"))
        })?;
        let display_info = string_child(node, XMLA_DATASET_NS, "DisplayInfo")?;
        let display_info = display_info.parse::<u32>().map_err(|error| {
            XmlaError::ParsingError(format!("Invalid numeric value {display_info:?}: {error}"))
        })?;
        Ok(Self {
            hierarchy,
            unique_name,
            caption,
            level_name,
            level_number,
            display_info,
        })
    }
}

impl XmlaCell {
    pub fn ordinal(&self) -> u32 {
        self.ordinal
    }
    /// Returns the numeric cell value.
    ///
    /// Missing or null values are returned as [`f64::NAN`].
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn formatted_value(&self) -> &str {
        &self.formatted_value
    }
}

/// Basic implementation, values are assumed to be numeric values
/// Other types and calculation errors are not supported
impl FromXml for XmlaCell {
    fn from_xml(node: Node) -> Result<Self, XmlaError> {
        crate::xmla::responses::check_tag_name(node, XMLA_DATASET_NS, "Cell")?;
        let ordinal = node.attribute("CellOrdinal").ok_or_else(|| {
            XmlaError::ParsingError("Expected CellOrdinal attribute in cell".to_string())
        })?;
        let ordinal = ordinal.parse::<u32>().map_err(|error| {
            XmlaError::ParsingError(format!("Invalid numeric value {ordinal:?}: {error}"))
        })?;
        let value = match optional_string_child(node, XMLA_DATASET_NS, "Value") {
            Some(value) => value.parse::<f64>().map_err(|error| {
                XmlaError::ParsingError(format!("Invalid numeric value {value:?}: {error}"))
            })?,
            None => f64::NAN,
        };
        let formatted_value = optional_string_child(node, XMLA_DATASET_NS, "FmtValue");
        Ok(Self {
            ordinal,
            value,
            formatted_value: formatted_value.unwrap_or_default(),
        })
    }
}

pub fn parse_execute_response(xml: &str) -> Result<XmlaDataset, XmlaError> {
    let document = Document::parse(xml)?;
    XmlaDataset::from_xml(get_body_child(&document)?)
}
