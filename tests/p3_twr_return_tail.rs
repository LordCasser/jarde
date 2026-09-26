//! A one-resource fixture isolates the proved post-close `load; return` tail and the cleanup
//! handler instructions that the recovered resource header reconstructs implicitly.

use jarde::*;
use std::slice;

const SAMPLE: &[u8] = include_bytes!("fixtures/p3-multi-resource-twr/TwrReturnTail.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn report(bytes: &[u8], method: &str) -> (String, RecoveryReport) {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("TwrReturnTail"),
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
    };
    let report = match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the frozen class answers the class-source request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class produced a complete answer: {other:?}"),
    };
    let member = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == method.as_bytes())
        .unwrap_or_else(|| panic!("the fixture has {method}()"));
    match &member.outcome {
        ClassSourceOutcome::Recovered { report, .. } => (member.text.clone(), (**report).clone()),
        other => panic!("{method}() has a recovery report: {other:?}"),
    }
}

fn code_span(bytes: &[u8], method_name: &str) -> (usize, usize) {
    fn u2(bytes: &[u8], at: usize) -> u16 {
        u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
    }
    fn u4(bytes: &[u8], at: usize) -> usize {
        u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize
    }
    let cp_count = u2(bytes, 8);
    let mut utf8: Vec<Option<String>> = vec![None; cp_count as usize];
    let mut at = 10;
    let mut index = 1;
    while index < cp_count {
        let tag = bytes[at];
        at += 1;
        match tag {
            1 => {
                let length = u2(bytes, at) as usize;
                at += 2;
                utf8[index as usize] =
                    Some(String::from_utf8_lossy(&bytes[at..at + length]).into());
                at += length;
            }
            3 | 4 => at += 4,
            5 | 6 => {
                at += 8;
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => at += 2,
            9 | 10 | 11 | 12 | 17 | 18 => at += 4,
            15 => at += 3,
            tag => panic!("known constant-pool tag {tag}"),
        }
        index += 1;
    }
    at += 6;
    let interfaces = u2(bytes, at);
    at += 2 + 2 * interfaces as usize;
    let fields = u2(bytes, at);
    at += 2;
    for _ in 0..fields {
        let attributes = u2(bytes, at + 6);
        at += 8;
        for _ in 0..attributes {
            at += 6 + u4(bytes, at + 2);
        }
    }
    let methods = u2(bytes, at);
    at += 2;
    for _ in 0..methods {
        let name = utf8[u2(bytes, at + 2) as usize].as_deref();
        let attributes = u2(bytes, at + 6);
        at += 8;
        for _ in 0..attributes {
            let attribute = utf8[u2(bytes, at) as usize].as_deref();
            let length = u4(bytes, at + 2);
            let body = at + 6;
            if name == Some(method_name) && attribute == Some("Code") {
                let code_length = u4(bytes, body + 4);
                return (body + 8, body + 8 + code_length);
            }
            at += 6 + length;
        }
    }
    panic!("{method_name}() Code attribute exists")
}

fn add_competing_cleanup_handler(bytes: &mut Vec<u8>, method_name: &str) {
    let (code_start, code_end) = code_span(bytes, method_name);
    let row_count = u16::from_be_bytes(bytes[code_end..code_end + 2].try_into().unwrap());
    let entry = code_end + 2 + usize::from(row_count) * 8;
    // In runSaved javac's primary close handler begins at BCI 21. Add a catch-all row for the
    // protected body that targets that block under a new row identity. The TWR proof did not
    // establish this row, so the candidate handler block must remain visible/refused.
    let row = [0, 10, 0, 19, 0, 21, 0, 0];
    bytes.splice(entry..entry, row);
    bytes[code_end..code_end + 2].copy_from_slice(&(row_count + 1).to_be_bytes());
    // The Code attribute's length is twelve bytes before its code array.
    let length_at = code_start - 12;
    let old_length = u32::from_be_bytes(bytes[length_at..length_at + 4].try_into().unwrap());
    bytes[length_at..length_at + 4].copy_from_slice(&(old_length + 8).to_be_bytes());
}

#[test]
fn proved_cleanup_handler_accesses_do_not_hide_the_saved_return_local() {
    let (text, recovered) = report(SAMPLE, "runSaved");
    assert!(
        text.contains("try ("),
        "the proved handler is owned: {text}"
    );
    assert!(
        text.contains("return local2;"),
        "the saved return is in the body: {text}"
    );
    assert!(recovered.fallbacks.is_empty(), "{recovered:?}");
}

#[test]
fn tail_read_of_a_different_local_is_not_misidentified_as_the_saved_body_value() {
    let mut bytes = SAMPLE.to_vec();
    let (start, end) = code_span(&bytes, "runSaved");
    let tail = bytes[start..end]
        .windows(2)
        .position(|pair| pair == [0x1c, 0xac])
        .expect("runSaved() ends with iload_2; ireturn");
    bytes[start + tail] = 0x1a; // iload_0 reads the argument, not the body result slot.
    let (text, recovered) = report(&bytes, "runSaved");
    assert!(
        text.contains("try ("),
        "the ordinary TWR still recovers: {text}"
    );
    assert!(
        text.contains("return arg0;"),
        "the return's actual local is preserved: {text}"
    );
    assert!(!text.contains("return local2;"), "{text}");
    assert!(recovered.fallbacks.is_empty(), "{recovered:?}");
}

#[test]
fn one_resource_layout_keeps_its_saved_return_and_cleanup_owned() {
    let (text, recovered) = report(SAMPLE, "run");
    assert!(
        text.contains("try ("),
        "the exact cleanup ownership does not regress the direct-return layout: {text}"
    );
    assert!(recovered.fallbacks.is_empty(), "{recovered:?}");
}

#[test]
fn a_competing_exception_row_cannot_borrow_the_proved_cleanup_handler() {
    let mut bytes = SAMPLE.to_vec();
    add_competing_cleanup_handler(&mut bytes, "runSaved");
    let (text, recovered) = report(&bytes, "runSaved");
    assert!(
        !text.contains("try ("),
        "the unproved row is not hidden: {text}"
    );
    assert!(!recovered.fallbacks.is_empty(), "{recovered:?}");
}

#[test]
fn effectful_post_close_tail_is_not_absorbed_into_the_resource_body() {
    let (text, recovered) = report(SAMPLE, "effect");
    assert!(
        text.contains("try ("),
        "the ordinary continuation remains a TWR: {text}"
    );
    let try_at = text.find("try (").unwrap();
    let increment_at = text.find(" = local0 + 1;").unwrap();
    let return_at = text.rfind("return local0;").unwrap();
    assert!(
        increment_at > try_at && return_at > increment_at,
        "the post-close effect and return stay outside the resource body: {text}"
    );
    assert!(recovered.fallbacks.is_empty(), "{recovered:?}");
}
