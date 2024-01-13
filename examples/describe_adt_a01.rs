use hl7_definitions::*;

fn print_fields(name: &str, segment: &Segment) {
    for (i, field) in segment.fields.iter().enumerate() {
        let i = i + 1;
        print!(
            "    {name}.{i} - {} [{}] {}, {}",
            field.description, field.datatype, field.optionality, field.repeatability
        );
        if let Some(table) = field.table {
            print!(" (table {table:>0})");
        }
        println!();
    }
}

fn print_message_segment(segment: &MessageSegment) {
    print!("  {}", segment.name);
    if segment.min > 0 {
        print!(" (required)");
    }
    if segment.max > 1 {
        print!(" (repeatable)");
    }
    println!(":");

    match get_segment("2.3", segment.name) {
        Some(seg) => print_fields(segment.name, seg),
        None => {
            if let Some(children) = segment.children {
                children.iter().for_each(print_message_segment);
            }
        }
    }
}

pub fn main() {
    let a01 = get_message("2.3", "ADT_A01").expect("Can get A01 message");
    println!("{} ({}) segments:", a01.name, a01.description);
    a01.segments.iter().for_each(print_message_segment)
}
