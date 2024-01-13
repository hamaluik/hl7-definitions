use phf_codegen::Map;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=[hl7-definitions] {}", format!($($tokens)*))
    }
}

#[derive(Deserialize)]
struct Table {
    desc: String,
    values: HashMap<String, String>,
}

fn codegen_tables(mut out: BufWriter<File>) -> BufWriter<File> {
    if std::env::var("CARGO_FEATURE_TABLES").is_err() {
        p!("Tables feature not enabled; tables will NOT be available");
        let table_refs: Map<u16> = Map::new();
        let table_descriptions: Map<u16> = Map::new();
        writeln!(
            &mut out,
            "pub static TABLES: phf::Map<u16, &'static TableValues> = {};",
            table_refs.build()
        )
        .expect("can write to codegen.rs");
        writeln!(
            &mut out,
            "pub static TABLE_DESCRIPTIONS: phf::Map<u16, &'static str> = {};",
            table_descriptions.build()
        )
        .expect("can write to codegen.rs");
        return out;
    }

    let tables =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/tables.json"))
            .expect("can open ./assets/tables.json");
    let tables: HashMap<String, Table> = serde_json::from_str(&tables).expect("can parse tables");
    let tables: HashMap<u16, Table> = tables
        .into_iter()
        .map(|(k, v)| {
            (
                k.as_str()
                    .parse::<u16>()
                    .unwrap_or_else(|_| panic!("can parse {k} as u16")),
                v,
            )
        })
        .collect();

    let mut table_descriptions = Map::new();
    for (k, v) in tables.iter() {
        table_descriptions.entry(k, &format!("r#\"{}\"#", v.desc));
    }
    writeln!(
        &mut out,
        "pub static TABLE_DESCRIPTIONS: phf::Map<u16, &'static str> = {};",
        table_descriptions.build()
    )
    .expect("can write to codegen.rs");

    for (table, v) in tables.iter() {
        let mut values = Map::new();
        for (k, v) in v.values.iter() {
            values.entry(k, &format!("r#\"{v}\"#"));
        }
        writeln!(
            &mut out,
            "pub static TABLE_{table}: phf::Map<&'static str, &'static str> = {};",
            values.build()
        )
        .expect("can write to codegen.rs");
    }
    let mut table_refs = Map::new();
    for table in tables.keys() {
        table_refs.entry(table, &format!("&TABLE_{table}"));
    }
    writeln!(
        &mut out,
        "pub static TABLES: phf::Map<u16, &'static TableValues> = {};",
        table_refs.build()
    )
    .expect("can write to codegen.rs");

    out
}

#[derive(Deserialize)]
struct Definition {
    fields: HashMap<String, Field>,
    segments: HashMap<String, Segment>,
    messages: HashMap<String, Message>,
}

#[derive(Deserialize)]
struct Field {
    desc: String,
    subfields: Vec<SubField>,
}

#[derive(Deserialize)]
struct SubField {
    datatype: String,
    desc: String,
    opt: usize,
    rep: usize,
    len: Option<usize>,
    table: Option<usize>,
}

#[derive(Deserialize)]
struct Segment {
    desc: String,
    fields: Vec<SubField>,
}

#[derive(Deserialize)]
struct Message {
    desc: String,
    name: String,
    segments: MessageSegments,
}

#[allow(unused)]
#[derive(Deserialize)]
struct MessageSegments {
    desc: String,
    segments: Vec<MessageSegment>,
}

#[derive(Deserialize)]
struct MessageSegment {
    name: String,
    desc: String,
    min: usize,
    max: usize,
    children: Option<Vec<MessageSegment>>,
    compounds: Option<Vec<MessageCompound>>,
}

#[derive(Deserialize)]
struct MessageCompound {
    name: Option<String>,
    desc: String,
    min: usize,
    max: usize,
}

fn format_message_segment(s: &MessageSegment) -> String {
    let MessageSegment {
        name,
        desc,
        min,
        max,
        children,
        compounds,
    } = s;

    let children = match children {
        Some(children) => {
            let children = children
                .iter()
                .map(format_message_segment)
                .collect::<Vec<String>>()
                .join(", ");
            format!("Some(&[{children}])")
        }
        None => "None".into(),
    };

    let compounds = match compounds {
        Some(compounds) => {
            let compounds = compounds
                .iter()
                .map(|c| {
                    let MessageCompound { name, desc, min, max } = c;
                    let name = match name {
                        Some(n) => format!(r##"Some(r#"{n}"#)"##),
                        None => "None".into()
                    };
                    format!(r##"MessageCompound {{ name: {name}, description: r#"{desc}"#, min: {min}, max: {max} }}"##)
                })
                .collect::<Vec<String>>()
                .join(", ");
            format!("Some(&[{compounds}])")
        }
        None => "None".into(),
    };

    format!(
        r##"MessageSegment {{ name: r#"{name}"#, description: r#"{desc}"#, min: {min}, max: {max}, children: {children}, compounds: {compounds} }}"##
    )
}

fn codegen_definitions(mut out: BufWriter<File>) -> BufWriter<File> {
    let definitions =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/defs.json"))
            .expect("can open ./assets/defs.json");
    let definitions: HashMap<String, Definition> =
        serde_json::from_str(&definitions).expect("can parse definitions");

    let mut definitions_map = Map::new();
    'versions: for (version, definitions) in definitions.iter() {
        if std::env::var(format!("CARGO_FEATURE_{}", version.replace('.', ""))).is_err() {
            p!("Version {version} feature disabled, version {version} will NOT be available!");
            continue 'versions;
        }

        let version_name = version.replace('.', "_");
        let mut fields = Map::new();
        for (field_id, field) in definitions.fields.iter() {
            let subfields = field
                .subfields
                .iter()
                .map(|s| {
                    let SubField {
                        datatype,
                        desc,
                        opt,
                        rep,
                        len,
                        table,
                    } = s;
                    let opt = map_optionality(*opt);
                    let rep = match rep {
                        0 => "FieldRepeatability::Unbounded".to_string(),
                        1 => "FieldRepeatability::Single".to_string(),
                        n => format!("FieldRepeatability::Bounded({n})"),
                    };
                    let len = match len {
                        Some(len) => format!("Some({len})"),
                        None => "None".to_string(),
                    };
                    let table = match table {
                        None => "None".to_string(),
                        Some(table) => format!("Some({table})"),
                    };
                    format!(r##"SubField {{ datatype: r#"{datatype}"#, description: r#"{desc}"#, optionality: {opt}, repeatability: {rep}, max_length: {len}, table: {table} }}"##)
                })
                .collect::<Vec<String>>()
                .join(", ");
            fields.entry(
                field_id,
                &format!(
                    r#"Field {{ description: "{}", subfields: &[{subfields}]}}"#,
                    field.desc
                ),
            );
        }
        let mut segments = Map::new();
        for (segment_id, segment) in definitions.segments.iter() {
            let fields = segment
                .fields
                .iter()
                .map(|f| {
                    let SubField { datatype, desc, opt, rep, len, table } = f;
                    let opt = map_optionality(*opt);
                    let rep = match rep {
                        0 => "FieldRepeatability::Unbounded".to_string(),
                        1 => "FieldRepeatability::Single".to_string(),
                        n => format!("FieldRepeatability::Bounded({n})"),
                    };
                    let len = match len {
                        Some(len) => format!("Some({len})"),
                        None => "None".to_string(),
                    };
                    let table = match table {
                        None => "None".to_string(),
                        Some(table) => format!("Some({table})"),
                    };
                    format!(r##"SubField {{ datatype: r#"{datatype}"#, description: r#"{desc}"#, optionality: {opt}, repeatability: {rep}, max_length: {len}, table: {table} }}"##)
                })
                .collect::<Vec<String>>()
                .join(", ");
            segments.entry(
                segment_id,
                &format!(
                    r#"Segment {{ description: "{}", fields: &[{fields}]}}"#,
                    segment.desc
                ),
            );
        }
        let mut messages = Map::new();
        for (message_id, message) in definitions.messages.iter() {
            let message_segments = message
                .segments
                .segments
                .iter()
                .map(format_message_segment)
                .collect::<Vec<String>>()
                .join(", ");

            let Message { desc, name, .. } = message;
            messages.entry(message_id, &format!(r##"Message {{ name: r#"{name}"#, description: r#"{desc}"#, segments: &[{message_segments}] }}"##));
        }
        writeln!(
            &mut out,
            "pub static DEFS_V{version_name}_FIELDS: phf::Map<&'static str, Field> = {};",
            fields.build()
        )
        .expect("can write to codegen.rs");
        writeln!(
            &mut out,
            "pub static DEFS_V{version_name}_SEGMENTS: phf::Map<&'static str, Segment> = {};",
            segments.build()
        )
        .expect("can write to codegen.rs");
        writeln!(
            &mut out,
            "pub static DEFS_V{version_name}_MESSAGES: phf::Map<&'static str, Message> = {};",
            messages.build()
        )
        .expect("can write to codegen.rs");

        definitions_map.entry(
            version,
            &format!("Definition {{ fields: &DEFS_V{version_name}_FIELDS, segments: &DEFS_V{version_name}_SEGMENTS, messages: &DEFS_V{version_name}_MESSAGES }}"),
        );
    }

    writeln!(
        &mut out,
        "pub static DEFINITIONS: phf::Map<&'static str, Definition> = {};",
        definitions_map.build()
    )
    .expect("can write to codegen.rs");

    writeln!(
        &mut out,
        "/// All of the versions compiled into the library"
    )
    .expect("can write to codegen.rs");
    writeln!(
        &mut out,
        "pub static VERSIONS: &[&str] = &[{}];",
        definitions
            .keys()
            .map(|k| format!(r#""{k}""#))
            .collect::<Vec<String>>()
            .join(", ")
    )
    .expect("can write to codegen.rs");

    out
}

fn map_optionality(opt: usize) -> &'static str {
    match opt {
        1 => "FieldOptionality::Optional",
        2 => "FieldOptionality::Required",
        3 => "FieldOptionality::Conditional",
        _ => "FieldOptionality::BackwardCompatibility",
    }
}

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut out = BufWriter::new(File::create(path).unwrap());

    writeln!(&mut out, "#[allow(unused)]\npub mod codegen {{\nuse super::*;\npub type TableValues = phf::Map<&'static str, &'static str>;").expect("can write to codegen.rs");
    let out = codegen_tables(out);
    let mut out = codegen_definitions(out);
    writeln!(&mut out, "}}").expect("can write to codegen.rs");
}
