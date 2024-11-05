use hl7_definitions::*;

fn print_message_segment(segment: &MessageSegment, indent: usize) {
    for _ in 0..indent {
        print!("  ");
    }
    print!("{}", segment.name);
    if segment.min > 0 {
        print!(" (required)");
    }
    if segment.max > 1 {
        print!(" (repeatable)");
    }
    println!(":");

    match get_segment("2.3", segment.name) {
        Some(_seg) => {},
        None => {
            if let Some(children) = segment.children {
                children.iter().for_each(|child| print_message_segment(child, indent + 1));
            }
        }
    }
}

pub fn main() {
    let a01 = get_message("2.3", "ORM_O01").expect("Can get A01 message");
    println!("{} ({}) segments:", a01.name, a01.description);
    a01.segments.iter().for_each(|segment| print_message_segment(segment, 1));
}
