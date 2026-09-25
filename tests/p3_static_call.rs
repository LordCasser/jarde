//! P3 4.4: a static call is written with the class its own pool entry names.
//!
//! `invokestatic` reads no receiver from the stack, so the text of one used to be the member's bare
//! name: `Integer.valueOf(n)` was presented as `valueOf(arg0)`, a name this class does not declare.
//! The class the instruction names is a fact of its own pool entry, and the owner of a static call
//! is written from it — unless that owner **is** the class the body belongs to, where the source's
//! own call was unqualified and `own(n)` stays `own(arg0)` rather than becoming `Calls.own(arg0)`.
//!
//! Nothing else about a static call changes: `valueOf` and `intValue` stay the two real calls they
//! are (no boxing or unboxing is invented in their place), no descriptor is appended to a name that
//! is not ambiguous, and an instance call or a constructor keeps the receiver it had.
//!
//! The texts below pin the shape over the class bytes committed in `tests/fixtures/p3-static-call/`
//! (see its `README.md` for the command, the version and the digest). `boxed` is the shape the
//! change is about — one `invokestatic java/lang/Integer.valueOf` — and `local` is the control: its
//! `invokestatic` names this very class, so the text must not gain a qualifier.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 317 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-static-call/v8/Calls.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one committed sample answers one definition, got {} candidate(s)",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one committed sample answers one definition, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

/// One member's own text in the assembled source.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
        .text
}

#[test]
fn a_static_call_names_the_class_its_pool_entry_names() {
    // `boxed(I)Ljava/lang/Integer;` is `iload_0; invokestatic java/lang/Integer.valueOf:(I)
    // Ljava/lang/Integer;; areturn`: the pool entry names a class this one is not, so the call is
    // written `java.lang.Integer.valueOf(arg0)` and not the bare `valueOf(arg0)`.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Calls");
    let boxed = text_of(&report, "boxed");
    assert!(
        boxed.contains("return java.lang.Integer.valueOf(arg0);"),
        "the call presents the class its own pool entry names:\n{boxed}"
    );
    // The name alone is not the call: this is not the source's `Integer.valueOf` folded into an
    // allocation, and it is not the bare name the pool's owner was dropped from.
    assert!(
        !boxed.contains("intValue"),
        "no unboxing is invented:\n{boxed}"
    );
    assert!(
        !boxed.contains("new "),
        "the call is not folded into an allocation:\n{boxed}"
    );
    assert!(
        !boxed.contains("return valueOf("),
        "the call names the class its pool entry names, not the member alone:\n{boxed}"
    );
    // `local(I)I` is `iload_0; invokestatic Calls.own:(I)I; ireturn`: the pool names **this** class,
    // where the source's own call was unqualified, so no qualifier is written.
    let local = text_of(&report, "local");
    assert!(
        local.contains("return own(arg0);"),
        "a call to this class's own member stays unqualified:\n{local}"
    );
    assert!(
        !local.contains("Calls.own"),
        "the text does not name the class a call of its own does not name:\n{local}"
    );
}
