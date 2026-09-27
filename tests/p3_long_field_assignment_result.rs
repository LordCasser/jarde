//! EM-07's one proved `dup2_x1` instance-long assignment result.

use jarde::*;
use std::slice;

const ASSIGNMENT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/em07-long-assignment/input/Assignment.class"
);

fn limits() -> Limits {
    task_budget(&[])
        .expect("task defaults are a bounded budget")
        .limits()
        .clone()
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut Budget::new(limits()),
        )
        .expect("the class fixture opens")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("em07/Assignment"),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    }
}

fn perform(bytes: &[u8], budget: &mut Budget) -> ClassSourceReport {
    match outcome(bytes, budget) {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => {
            panic!(
                "one fixture resolves to one class, got {}",
                candidates.candidates.len()
            )
        }
        OperationOutcome::Incomplete(candidates) => {
            panic!(
                "one fixture is complete, got {}",
                candidates.candidates.len()
            )
        }
    }
}

fn outcome(bytes: &[u8], budget: &mut Budget) -> OperationOutcome<ClassSourceReport> {
    let snapshot = open(bytes);
    Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot),
            &RecoveryEvidenceRequest::all(),
            budget,
        )
        .expect("the class-source request is valid")
}

fn method_text<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing method {name}"))
        .text
}

#[test]
fn the_two_category2_consumers_become_one_field_write_and_one_return() {
    let report = perform(ASSIGNMENT, &mut Budget::new(limits()));
    let text = method_text(&report, "setValue");
    assert!(text.contains("this.value = arg1;"), "{text}");
    assert!(text.contains("return arg1;"), "{text}");
    assert!(!text.contains("@bytecode"), "{text}");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"setValue")
        .unwrap();
    let ClassSourceOutcome::Recovered {
        report: recovery, ..
    } = &method.outcome
    else {
        panic!(
            "setValue should have a recovery report: {:?}",
            method.outcome
        );
    };
    assert_eq!(recovery.content, RecoveryContent::ContainsStatements);
    assert_eq!(
        recovery
            .fields
            .iter()
            .map(|field| (field.bci, field.access, field.presented))
            .collect::<Vec<_>>(),
        vec![(3, "write", true)]
    );
    assert_eq!(recovery.source_map.direct_of_bci(3).len(), 1);
    assert_eq!(recovery.source_map.direct_of_bci(6).len(), 1);
    assert!(!recovery.source_map.derived_of_bci(2).is_empty());
    assert_eq!(text.matches("this.value = arg1;").count(), 1);
    assert_eq!(text.matches("return arg1;").count(), 1);
}

#[test]
fn owner_width_extra_consumers_and_handlers_do_not_project_the_pair() {
    let cases = [
        ("different owner", wrong_owner(ASSIGNMENT)),
        ("different width", wrong_width(ASSIGNMENT)),
        ("inserted operation", insert_code(ASSIGNMENT, 1, &[0x00])),
        (
            "second category2 consumer",
            insert_code(ASSIGNMENT, 7, &[0x5c, 0x58]),
        ),
        ("exception handler", catch_putfield(ASSIGNMENT)),
    ];
    for (label, bytes) in cases {
        let report = perform(&bytes, &mut Budget::new(limits()));
        let text = method_text(&report, "setValue");
        assert!(
            !text.contains("this.value = arg1;"),
            "{label} must refuse the whole assignment result:\n{text}"
        );
    }
}

#[test]
fn output_budget_and_cancellation_publish_no_partial_pair() {
    let complete = perform(ASSIGNMENT, &mut Budget::new(limits()));
    let cap = complete.usage.output_bytes.saturating_sub(1);
    let mut bounded = Budget::new(
        task_budget(&[BudgetOverride::new("output_bytes", cap).unwrap()])
            .unwrap()
            .limits()
            .clone(),
    );
    let stopped = perform(ASSIGNMENT, &mut bounded);
    assert!(matches!(
        stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        }
    ));
    assert_eq!(
        stopped.text.contains("this.value = arg1;"),
        stopped.text.contains("return arg1;"),
        "output budget must expose both statements or neither:\n{}",
        stopped.text
    );

    let mut low_ir = Budget::new(
        task_budget(&[BudgetOverride::new("ir_items", 100).unwrap()])
            .unwrap()
            .limits()
            .clone(),
    );
    match outcome(ASSIGNMENT, &mut low_ir) {
        OperationOutcome::Performed(report) => assert_eq!(
            report.text.contains("this.value = arg1;"),
            report.text.contains("return arg1;"),
            "low IR budget must expose both statements or neither:\n{}",
            report.text
        ),
        OperationOutcome::Incomplete(_) => {}
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one fixture stays unique under an IR budget stop, got {} candidates",
            candidates.candidates.len()
        ),
    }

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token);
    let stopped = outcome(ASSIGNMENT, &mut cancelled);
    assert!(matches!(stopped, OperationOutcome::Incomplete(_)));
}

#[derive(Clone, Copy)]
struct CpEntry {
    tag: u8,
    payload: usize,
}

fn u2(bytes: &[u8], offset: usize) -> usize {
    usize::from(u16::from_be_bytes([bytes[offset], bytes[offset + 1]]))
}

fn u4(bytes: &[u8], offset: usize) -> usize {
    usize::try_from(u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
    .unwrap()
}

fn put_u2(bytes: &mut [u8], offset: usize, value: usize) {
    bytes[offset..offset + 2].copy_from_slice(&u16::try_from(value).unwrap().to_be_bytes());
}

fn put_u4(bytes: &mut [u8], offset: usize, value: usize) {
    bytes[offset..offset + 4].copy_from_slice(&u32::try_from(value).unwrap().to_be_bytes());
}

fn constant_pool(bytes: &[u8]) -> (Vec<Option<CpEntry>>, usize) {
    let count = u2(bytes, 8);
    let mut entries = vec![None; count];
    let mut index = 1;
    let mut offset = 10;
    while index < count {
        let tag = bytes[offset];
        let payload = offset + 1;
        let width = match tag {
            1 => 2 + u2(bytes, payload),
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => 4,
            5 | 6 => 8,
            7 | 8 | 16 | 19 | 20 => 2,
            15 => 3,
            _ => panic!("unexpected constant-pool tag {tag}"),
        };
        entries[index] = Some(CpEntry { tag, payload });
        offset += 1 + width;
        index += if tag == 5 || tag == 6 { 2 } else { 1 };
    }
    (entries, offset)
}

fn utf8<'a>(bytes: &'a [u8], entries: &[Option<CpEntry>], index: usize) -> &'a [u8] {
    let entry = entries[index].unwrap();
    assert_eq!(entry.tag, 1);
    let length = u2(bytes, entry.payload);
    &bytes[entry.payload + 2..entry.payload + 2 + length]
}

fn fieldref(bytes: &[u8]) -> (usize, usize) {
    let (entries, _) = constant_pool(bytes);
    for entry in entries.iter().flatten().filter(|entry| entry.tag == 9) {
        let name_and_type = u2(bytes, entry.payload + 2);
        let nat_entry = entries[name_and_type].unwrap();
        let name = u2(bytes, nat_entry.payload);
        let descriptor = u2(bytes, nat_entry.payload + 2);
        if utf8(bytes, &entries, name) == b"value" && utf8(bytes, &entries, descriptor) == b"J" {
            return (entry.payload, nat_entry.payload + 2);
        }
    }
    panic!("fixture field reference value:J is present")
}

fn class_index(bytes: &[u8], class_name: &[u8]) -> usize {
    let (entries, _) = constant_pool(bytes);
    entries
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, entry)| {
            let entry = (*entry)?;
            (entry.tag == 7 && utf8(bytes, &entries, u2(bytes, entry.payload)) == class_name)
                .then_some(index)
        })
        .unwrap()
}

fn wrong_owner(original: &[u8]) -> Vec<u8> {
    let mut bytes = original.to_vec();
    let (owner_offset, _) = fieldref(&bytes);
    let owner = class_index(&bytes, b"java/lang/Object");
    put_u2(&mut bytes, owner_offset, owner);
    bytes
}

fn wrong_width(original: &[u8]) -> Vec<u8> {
    let mut bytes = original.to_vec();
    let (_, descriptor_offset) = fieldref(&bytes);
    let (entries, cp_end) = constant_pool(&bytes);
    let new_index = entries.len();
    put_u2(&mut bytes, descriptor_offset, new_index);
    bytes.splice(cp_end..cp_end, [1, 0, 1, b'I']);
    put_u2(&mut bytes, 8, new_index + 1);
    bytes
}

fn code_offsets(bytes: &[u8]) -> (usize, usize, usize, usize, usize, usize) {
    let (entries, mut offset) = constant_pool(bytes);
    offset += 6;
    let interfaces = u2(bytes, offset);
    offset += 2 + interfaces * 2;
    let fields = u2(bytes, offset);
    offset += 2;
    for _ in 0..fields {
        offset = skip_member(bytes, offset);
    }
    let methods = u2(bytes, offset);
    offset += 2;
    for _ in 0..methods {
        let name_index = u2(bytes, offset + 2);
        let attr_count = u2(bytes, offset + 6);
        let mut attribute = offset + 8;
        for _ in 0..attr_count {
            let attr_name = u2(bytes, attribute);
            let length_offset = attribute + 2;
            let length = u4(bytes, length_offset);
            let payload = attribute + 6;
            if utf8(bytes, &entries, name_index) == b"setValue"
                && utf8(bytes, &entries, attr_name) == b"Code"
            {
                let code_length_offset = payload + 4;
                let code_start = payload + 8;
                let code_end = code_start + u4(bytes, code_length_offset);
                let handler_count_offset = code_end;
                return (
                    code_start,
                    code_end,
                    code_length_offset,
                    length_offset,
                    handler_count_offset,
                    code_start + u4(bytes, code_length_offset),
                );
            }
            attribute += 6 + length;
        }
        offset = attribute;
    }
    panic!("setValue Code attribute exists")
}

fn skip_member(bytes: &[u8], offset: usize) -> usize {
    let attributes = u2(bytes, offset + 6);
    let mut next = offset + 8;
    for _ in 0..attributes {
        next += 6 + u4(bytes, next + 2);
    }
    next
}

fn insert_code(original: &[u8], position: usize, extra: &[u8]) -> Vec<u8> {
    let mut bytes = original.to_vec();
    let (code_start, _, code_length_offset, attr_length_offset, _, _) = code_offsets(&bytes);
    let insertion = code_start + position;
    bytes.splice(insertion..insertion, extra.iter().copied());
    let code_length = u4(&bytes, code_length_offset) + extra.len();
    put_u4(&mut bytes, code_length_offset, code_length);
    let attr_length = u4(&bytes, attr_length_offset) + extra.len();
    put_u4(&mut bytes, attr_length_offset, attr_length);
    bytes
}

fn catch_putfield(original: &[u8]) -> Vec<u8> {
    let mut bytes = original.to_vec();
    let (code_start, code_end, code_length_offset, attr_length_offset, _, _) = code_offsets(&bytes);
    bytes.splice(code_end..code_end, [0x57, 0x09, 0xad]);
    let new_code_end = code_end + 3;
    let code_length = u4(&bytes, code_length_offset) + 3;
    put_u4(&mut bytes, code_length_offset, code_length);
    let handler_count_offset = new_code_end;
    let count = u2(&bytes, handler_count_offset);
    put_u2(&mut bytes, handler_count_offset, count + 1);
    let handler = [0, 3, 0, 6, 0, 7, 0, 0];
    bytes.splice(handler_count_offset + 2..handler_count_offset + 2, handler);
    let attr_length = u4(&bytes, attr_length_offset) + 11;
    put_u4(&mut bytes, attr_length_offset, attr_length);
    assert_eq!(code_start + 7, new_code_end - 3);
    bytes
}
