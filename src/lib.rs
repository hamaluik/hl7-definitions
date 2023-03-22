use std::fmt::Display;

use phf::Map;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub use codegen::VERSIONS;

/// Get the description of the given table index
///
/// # Example
///
/// ```
/// # use hl7_definitions::*;
/// assert_eq!(table_description(3), Some("Event type"));
/// ```
#[inline]
pub fn table_description(table: u16) -> Option<&'static str> {
    codegen::TABLE_DESCRIPTIONS.get(&table).copied()
}

/// Get a single value from a table
///
/// # Example
///
/// ```
/// # use hl7_definitions::*;
/// assert_eq!(table_value(3, "A01"), Some("ADT/ACK - Admit/visit notification"));
/// ```
#[inline]
pub fn table_value(table: u16, key: &'static str) -> Option<&'static str> {
    codegen::TABLES
        .get(&table)
        .and_then(|table| table.get(key).copied())
}

/// Get _all_ the values for a given table
///
/// # Example
///
/// ```
/// # use hl7_definitions::*;
/// assert_eq!(table_values(7).unwrap().len(), 7);
/// ```
#[inline]
pub fn table_values(table: u16) -> Option<&'static [(&'static str, &'static str)]> {
    codegen::TABLES.get(&table).map(|table| table.entries)
}

/// The root definition for a given version, describing the schema for that HL7 version
#[derive(Debug)]
pub struct Definition {
    /// All the possible fields / datatypes that are present in the version
    pub fields: &'static Map<&'static str, Field>,
    /// All the possible segments that are present in the version
    pub segments: &'static Map<&'static str, Segment>,
    /// All the possible message types that are present in the version
    pub messages: &'static Map<&'static str, Message>,
}

/// How "required" is the field
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FieldOptionality {
    /// The field is optional
    Optional,
    /// The field is required
    Required,
    /// The field is only conditionally required. TODO: better description
    Conditional,
    /// The field is only there for backwards compatibility
    BackwardCompatibility,
}

impl Display for FieldOptionality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldOptionality::Optional => write!(f, "optional"),
            FieldOptionality::Required => write!(f, "required"),
            FieldOptionality::Conditional => write!(f, "conditional"),
            FieldOptionality::BackwardCompatibility => write!(f, "backwards compatibility"),
        }
    }
}

/// How many times a field can be repeated
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FieldRepeatability {
    /// The field can be repeated to infinity
    Unbounded,
    /// There can only be one
    Single,
    /// The field can be repeated `n` many times
    Bounded(usize),
}

impl Display for FieldRepeatability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldRepeatability::Unbounded => write!(f, "unbounded"),
            FieldRepeatability::Single => write!(f, "singular"),
            FieldRepeatability::Bounded(n) => write!(f, "maximum {n}"),
        }
    }
}

/// A field type (could be an HL7 field, component, or sub-component depending on its usage),
/// effectively a datatype
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Field {
    /// The description of the field
    pub description: &'static str,
    /// All possible sub-fields for the field
    pub subfields: &'static [SubField],
}

/// Generally the lowest-level datatype, represents what a component or sub-component can be
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SubField {
    /// The datatype (field) of the sub-field
    pub datatype: &'static str,
    /// A description of the sub-field
    pub description: &'static str,
    /// Whether the sub-field is required or not
    pub optionality: FieldOptionality,
    /// The maximum length of the sub-field; if `None` then unbounded or not applicable
    pub max_length: Option<usize>,
    /// How many times the sub-field can be repeated
    pub repeatability: FieldRepeatability,
    /// What table holds valid values for this sub-field
    pub table: Option<usize>,
}

/// Schema for a segment (`MSH`, `PID`, etc)
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Segment {
    /// The description of the segment
    pub description: &'static str,
    /// The ordered list of fields present in this segment
    pub fields: &'static [SubField],
}

/// Schema for a mesasge (`ADT_A01`, etc)
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Message {
    /// A description of the message
    pub description: &'static str,
    /// The name of the message
    pub name: &'static str,
    /// The segments present in the message
    pub segments: &'static [MessageSegment],
}

/// A segment within a message
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct MessageSegment {
    /// The name of the segment (3 capital letters)
    pub name: &'static str,
    /// A description of the segment
    pub description: &'static str,
    /// Minimum number of times the segment must appear in the message; if `> 0` then it is
    /// required
    pub min: usize,
    /// The maximum number of times the segment must appear in the message
    pub max: usize,
    /// The message may contain child segments; which are included here
    pub children: Option<&'static [MessageSegment]>,
    /// While `children` defines a sequence of segments that could be included in the message,
    /// `compounds` defines a set of segments as choices to be allowed in te same position
    pub compounds: Option<&'static [MessageCompound]>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct MessageCompound {
    pub name: Option<&'static str>,
    pub description: &'static str,
    pub min: usize,
    pub max: usize,
}

/// Query for a root-level definition for the given version
///
/// # Example
///
/// ```
/// # use hl7_definitions::*;
/// assert!(get_definition("2.5.1").is_some());
/// ```
#[inline]
pub fn get_definition(version: &str) -> Option<&'static Definition> {
    codegen::DEFINITIONS.get(version)
}

/// Get a specific field for the given version
///
/// # Example
///
/// ```
/// # use hl7_definitions::*;
/// assert!(get_field("2.5.1", "TS").is_some());
/// ```
pub fn get_field(version: &str, field: &str) -> Option<&'static Field> {
    codegen::DEFINITIONS
        .get(version)
        .and_then(|defs| defs.fields.get(field))
}

/// Get a specific segment for the given version
///
/// # Example
///
/// ```
/// # use hl7_definitions::*;
/// assert!(get_segment("2.5.1", "MSH").is_some());
/// ```
pub fn get_segment(version: &str, segment: &str) -> Option<&'static Segment> {
    codegen::DEFINITIONS
        .get(version)
        .and_then(|defs| defs.segments.get(segment))
}

///
/// # Example
///
/// ```
/// # use hl7_definitions::*;
/// assert!(get_message("2.5.1", "ADT_A01").is_some());
/// ```
pub fn get_message(version: &str, message: &str) -> Option<&'static Message> {
    codegen::DEFINITIONS
        .get(version)
        .and_then(|defs| defs.messages.get(message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_table_descriptions() {
        assert_eq!(
            table_description(1).expect("table exists"),
            "Administrative Sex"
        );
        assert_eq!(
            table_description(895).expect("table exists"),
            "Present On Admission (POA) Indicator"
        );
    }

    #[test]
    fn can_get_table_value() {
        assert_eq!(
            table_value(3, "A08").expect("table value exists"),
            "ADT/ACK -  Update patient information"
        );
    }

    #[test]
    fn can_get_table_values() {
        let values = table_values(91).expect("can get table 91 values");
        assert_eq!(values.len(), 2);
        assert_eq!(
            *values
                .iter()
                .find(|(k, _)| k == &"D")
                .expect("can find entry D"),
            ("D", "Deferred")
        );
        assert_eq!(
            *values
                .iter()
                .find(|(k, _)| k == &"I")
                .expect("can find entry I"),
            ("I", "Immediate")
        );
    }

    #[test]
    fn can_list_versions() {
        assert!(VERSIONS.iter().any(|v| v == &"2.5.1"));
    }

    #[test]
    fn can_get_definitions_for_version() {
        let defs = get_definition("2.5.1").expect("can get definition for v2.5.1");
        assert!(!defs.fields.is_empty());
        assert!(!defs.segments.is_empty());
    }

    #[test]
    fn can_get_fields_for_version() {
        let ad = get_field("2.5.1", "AD").expect("can get AD field for v2.5.1");
        assert_eq!(ad.description, "Address");
        assert_eq!(ad.subfields.len(), 8);
        assert_eq!(ad.subfields[0].datatype, "ST");
        assert_eq!(ad.subfields[0].description, "Street Address");
        assert_eq!(ad.subfields[0].optionality, FieldOptionality::Required);
        assert_eq!(ad.subfields[0].repeatability, FieldRepeatability::Single);
        assert_eq!(ad.subfields[0].max_length, Some(120));
        assert_eq!(ad.subfields[0].table, None);
    }

    #[test]
    fn can_get_segments_for_version() {
        let msh = get_segment("2.5.1", "MSH").expect("can get MSH segment for v2.5.1");
        assert_eq!(msh.description, "Message Header");
        assert_eq!(msh.fields.len(), 21);
        assert_eq!(msh.fields[9].description, "Message Control ID");
    }

    #[test]
    fn can_get_messages_for_version() {
        let a01 = get_message("2.5.1", "ADT_A01").expect("can get ADT_A01 message for v2.5.1");
        assert_eq!(a01.segments.len(), 22);
        let msh = a01.segments[0];
        assert_eq!(msh.name, "MSH");
        assert_eq!(msh.min, 1);
        assert_eq!(msh.max, 1);
    }
}
