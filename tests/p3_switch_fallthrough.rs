//! P3 task 2c.4: proven ordinary switch fallthrough follows target-code order.
//!
//! The frozen lookup/table-switch inputs are in `tests/fixtures/p3-switch-fallthrough/`; the
//! evidence bundle under `openspec/evidence/java-syntax-2026-09-22/switch-fallthrough-order/`
//! contains source, javap, JADX/Jarde output, and executable audit runners.

use jarde::*;
use std::slice;

const LOOKUP: &[u8] =
    include_bytes!("fixtures/p3-switch-fallthrough/v8/SwitchFallthroughOrder.class");
const TABLE: &[u8] =
    include_bytes!("fixtures/p3-switch-fallthrough/v8/TableswitchFallthrough.class");
const DEFAULT_MIDDLE: &[u8] =
    include_bytes!("fixtures/p3-switch-fallthrough/v8/DefaultMiddle.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn choose_text(bytes: &[u8], class: &str) -> String {
    let mut budget = budget();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("fixture opens as a standalone class");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class-source request is valid")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one fixture class answers the request: {other:?}"),
    };
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"choose")
        .unwrap_or_else(|| panic!("{class} has no choose method"))
        .text
        .clone()
}

#[test]
fn lookup_and_table_switches_order_only_proven_fallthrough_and_keep_breaks() {
    let lookup = choose_text(LOOKUP, "SwitchFallthroughOrder");
    let nine = lookup.find("case 9:").expect("case 9 is emitted");
    let one = lookup.find("case 1:").expect("case 1 is emitted");
    let four = lookup.find("case 4:").expect("case 4 is emitted");
    assert!(
        nine < one && one < four,
        "lookup labels follow BCI order:\n{lookup}"
    );
    assert!(
        !lookup[nine..one].contains("break;"),
        "case 9 falls through into case 1:\n{lookup}"
    );
    assert!(
        lookup[one..four].contains("break;"),
        "the independent case 1 still ends with break:\n{lookup}"
    );
    assert!(
        lookup.contains("default:"),
        "default is retained:\n{lookup}"
    );

    let table = choose_text(TABLE, "TableswitchFallthrough");
    let four = table.find("case 4:").expect("case 4 is emitted");
    let one = table.find("case 1:").expect("case 1 is emitted");
    let three = table.find("case 3:").expect("case 3 is emitted");
    assert!(
        four < one && one < three,
        "table labels follow BCI order:\n{table}"
    );
    assert!(
        !table[four..one].contains("break;"),
        "case 4 falls through into the shared case 1/2 entry:\n{table}"
    );
    assert!(
        table[one..three].contains("case 2:") && table[one..three].contains("break;"),
        "shared labels remain together and their arm still ends:\n{table}"
    );
    assert!(table.contains("case 3:") && table.contains("default:"));
}

#[test]
fn default_in_the_middle_keeps_both_proven_fallthrough_edges() {
    let text = choose_text(DEFAULT_MIDDLE, "DefaultMiddle");
    let nine = text.find("case 9:").expect("case 9 is emitted");
    let default = text.find("default:").expect("default is emitted");
    let one = text.find("case 1:").expect("case 1 is emitted");
    let four = text.find("case 4:").expect("case 4 is emitted");
    assert!(nine < default && default < one && one < four, "{text}");
    assert!(!text[nine..default].contains("break;"), "{text}");
    assert!(!text[default..one].contains("break;"), "{text}");
    assert!(text[one..four].contains("break;"), "{text}");
}
