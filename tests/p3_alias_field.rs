//! P3 2c.32: the receiver of a field read that a **local alias** holds.
//!
//! `local0 = Lazy.h; if (local0 == null) { local0 = new Holder(); Lazy.h = local0; } return local0.v;`
//! leaves the field read at the merge of two writes: `aload_0; getfield Lazy$Holder.v`. The value the
//! receiver comes from is the merge point's own — the entry phi of local 0, and the load of that slot
//! — and the class file's `StackMapTable` states the slot's class there
//! (`append_frame, offset_delta = 20, locals = [ class Lazy$Holder ]`).
//!
//! Where the class went: the two writes of the slot spell the one class **two ways** — the `getstatic`
//! carries the field descriptor's slice (`LLazy$Holder;`) and the `new`'s class entry carries the
//! internal name (`Lazy$Holder`) — and the frame pass's merge compared those names as raw bytes, so
//! it answered `Ref(Unknown)` for a slot both paths agree the class of. The receiver check read that
//! `Unknown` — the phi's own type **is** the frames' entry class for the slot — refused the access,
//! and both the read and its `return` were quoted:
//!
//! ```text
//! // @bytecode 21
//! // @bytecode 24
//! ```
//!
//! The rule is the frames' own answer, and nothing new is read off the body: the two spellings are
//! one type (`L…;` is a wrapping, not part of a reference's identity — the oracle and the recovery
//! layer's own `internal_form` already treat them as one), so the merge keeps the class and the
//! receiver reads `Lazy$Holder` off the value the slot holds.
//!
//! The texts below pin that on the class bytes committed in `tests/fixtures/p3-alias-field/` (see its
//! `README.md` for the command, the version, the bytes and the digest). `get` is the shape the rule
//! is about; `viaParam` and `viaThis` are the two receivers that were already stated by the value
//! itself — a reference parameter and `this` — and must stay exactly what they were.
//!
//! Two things are deliberately **not** here and are pinned where they already live: a receiver whose
//! stated type differs from the pool's owner (P05/P11 — the rule must not widen to a subtype), and a
//! static access, which has no receiver at all.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 445 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-alias-field/v8/Lazy.class");

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

/// One member of the sample, as the class presentation holds it.
fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's own recovery run: the member has to have been run, so a member that was renamed, or
/// one whose body this run could not reach at all, fails here instead of covering less than this
/// file claims.
fn run_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` is not a member this run recovered a body for: {other:?}"),
    }
}

/// One member's body, presented in full: Java, structured, with at least one statement and no
/// `// jarde:` marker — the planes have to agree with the text this file reads.
fn presented<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    let run = run_of(report, name);
    assert!(run.produced(), "`{name}` produced no artifact: {run:?}");
    assert_eq!(
        run.representation,
        Representation::Java,
        "`{name}`: {run:?}"
    );
    assert_eq!(run.quality, Quality::Structured, "`{name}`: {run:?}");
    assert_eq!(
        member(report, name).markers,
        Vec::<String>::new(),
        "`{name}` is recovered in full"
    );
    &member(report, name).text
}

// ---------------------------------------------------------------------------------------------
// P3 2c.32: the alias receiver's class is the frames' answer for its slot
// ---------------------------------------------------------------------------------------------

/// `get()I` is the shape: a local is read from a static field, written again on one arm, and the
/// field is read back through that local after the merge. The read is presented, the local is the
/// declaration the frames state for it, and no bytecode is quoted anywhere in the body.
#[test]
fn a_local_alias_receiver_is_presented_from_the_frames_slot_type() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Lazy");

    let get = presented(&report, "get");
    assert!(
        get.contains("return local0.v;"),
        "the tail reads the field through the local the merge wrote:\n{get}"
    );
    assert!(
        !get.contains("@bytecode"),
        "no instruction of the body is quoted:\n{get}"
    );
    assert!(
        !get.contains("this run does not state"),
        "the receiver's type is stated by the frames:\n{get}"
    );
    // The declaration the frames state for the alias, and the two static accesses beside it: the
    // read that fills the local and the write the arm performs.
    assert!(
        get.contains("Lazy$Holder local0;"),
        "the alias is declared with the class the frames state for its slot:\n{get}"
    );
    assert!(
        get.contains("local0 = Lazy.h;"),
        "the alias is filled by the static read:\n{get}"
    );
    assert!(
        get.contains("Lazy.h = local0;"),
        "the arm's static write keeps its own text:\n{get}"
    );
    // The class-level declaration of the field the body reads and writes: the volatile modifier is
    // the class's own fact and no rule of this build moves it.
    assert!(
        report
            .fields
            .iter()
            .any(|field| field.declaration.as_deref() == Some("static volatile Lazy$Holder h")),
        "the volatile static field is declared as the class declares it: {:?}",
        report
            .fields
            .iter()
            .map(|field| field.declaration.as_deref())
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------------------------
// The controls: receivers the value itself already states
// ---------------------------------------------------------------------------------------------

/// `viaParam(LLazy$Holder;)I` reads the field through a reference **parameter**, whose value states
/// its own class: the receiver check took this path before the rule and must take it after it.
#[test]
fn a_parameter_receiver_is_unchanged() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Lazy");

    let via_param = presented(&report, "viaParam");
    assert!(
        via_param.contains("return arg1.v;"),
        "the parameter is the receiver's own text:\n{via_param}"
    );
    assert!(
        !via_param.contains("@bytecode"),
        "the read is presented, not quoted:\n{via_param}"
    );
}

/// `viaThis()I` reads the field through `this`, which the frames state as the class being read: the
/// one receiver whose type is a class from the start, and which no rule of this build may requalify.
#[test]
fn a_this_receiver_is_unchanged() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Lazy");

    let via_this = presented(&report, "viaThis");
    assert!(
        via_this.contains("return this.v;"),
        "`this` is the receiver's own text:\n{via_this}"
    );
    assert!(
        !via_this.contains("@bytecode") && !via_this.contains("arg0"),
        "the read is presented through `this` and no local:\n{via_this}"
    );
}
