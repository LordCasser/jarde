//! Raw member-table names are MUTF-8, including valid non-scalar UTF-16 values.
use jarde_reader::{
    budget::{Budget, Limits},
    classfile::class_member_facts,
    model::JvmString,
};

const SAMPLE: &[u8] = include_bytes!("../../../tests/fixtures/s1-raw-names/RawAlias.class");

fn budget() -> Budget {
    Budget::new(Limits {
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        result_items: 10_000,
        analysis_steps: 1 << 20,
        elapsed_millis: 60_000,
        ..Limits::default()
    })
}

fn renamed(raw: &[u8]) -> Vec<u8> {
    let needle = b"\x01\x00\x05probe";
    let positions: Vec<_> = SAMPLE
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == needle).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    let start = positions[0];
    let mut bytes = SAMPLE[..start + 1].to_vec();
    bytes.extend_from_slice(&(raw.len() as u16).to_be_bytes());
    bytes.extend_from_slice(raw);
    bytes.extend_from_slice(&SAMPLE[start + needle.len()..]);
    bytes
}

#[test]
fn malformed_member_names_stop_before_publishing_the_record() {
    let control = class_member_facts(SAMPLE, &mut budget()).unwrap();
    assert!(control.stopped_at.is_none() && control.methods.len() == 3);
    let mut rows = Vec::new();
    for raw in [
        b"\0".as_slice(),
        &[0xc1, 0x81],
        &[0xed, 0xa0],
        &[0xf0, 0x90, 0x90, 0x80],
    ] {
        let read = class_member_facts(&renamed(raw), &mut budget());
        let refused = match &read {
            Err(jarde_reader::error::Error::InvalidInput { code, .. }) => {
                code.starts_with("classfile_")
            }
            Err(error) => panic!("not a malformed-input refusal: {error}"),
            Ok(table) => table.stopped_at.is_some() && table.methods.len() == 1,
        };
        rows.push(serde_json::json!({"raw":raw,"refused":refused,"read":format!("{read:?}")}));
    }
    if let Ok(path) = std::env::var("JARDE_RAW_MEMBER_PROBE_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
    assert!(rows.iter().all(|r| r["refused"] == true), "{rows:#?}");
}

#[test]
fn a_valid_unpaired_surrogate_retains_its_raw_name_without_becoming_replacement_text() {
    let raw = [0xed, 0xa0, 0x81];
    let table = class_member_facts(&renamed(&raw), &mut budget()).unwrap();
    assert!(table.stopped_at.is_none());
    let name = &table.methods[1].name;
    assert_eq!(name.raw().0, raw);
    assert_eq!(name.utf16(), [0xd801]);
    assert_eq!(name.escaped(), "\\uD801");
    let round_trip: JvmString =
        serde_json::from_value(serde_json::to_value(name).unwrap()).unwrap();
    assert_eq!(&round_trip, name);
}
