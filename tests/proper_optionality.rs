use hl7_definitions::{FieldOptionality, get_segment};

#[test]
fn test_proper_optionality() {
    let segment = get_segment("2.3", "MSH").expect("MSH segment");
    let st_field = segment.fields.iter().nth(7).expect("MSH.8");
    assert_eq!(st_field.description, "Security");
    assert_eq!(st_field.optionality, FieldOptionality::Optional);
}
