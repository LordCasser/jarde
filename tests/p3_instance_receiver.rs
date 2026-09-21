//! P3: the instance receiver is spelled by its **identity**, and only by its identity.
//!
//! JVMS 4.10.1.9 puts the receiver in local slot 0 of every member that is not `static`, so slot 0
//! is `this` — not a name the debug table happens to state (`this`, `this$0`, `self`), and not the
//! ordinal name `arg0` a body with no `LocalVariableTable` would otherwise get. Three properties are
//! pinned here, through the entry point the CLI calls ([`Engine::class_source`]) and over committed
//! class files rather than hand-assembled facts:
//!
//! * **the receiver is written `this` wherever the body reads it.** `Holder.value()` reads it for its
//!   field, `Holder(int)` writes through it around its constructor call, and `Shape`'s `default`
//!   method calls through it — the same class files whose texts named that slot `arg0` before this
//!   change (`--release 8 -g:none`: no debug table, so the ordinal rule used to reach slot 0).
//! * **slot 0 of a `static` member is untouched.** `Holder.of(int)` and `Shape.sum(int, int)` read
//!   their first parameter exactly as before — `arg<slot>` with no debug evidence — and no `this`
//!   appears in a member where no receiver exists.
//! * **a receiver the body never reads is not invented**, and the debug table's name for it is not
//!   an alias any more. `Scope.receiver(J)J` (the `-g` sample, whose table names slot 0 `this`) never
//!   reads its receiver: its text is the same and its run no longer reports the slot as a name that
//!   could not be written.
//!
//! Every other slot keeps the naming it had, which is why the declarations here are asserted beside
//! the bodies: a parameter the debug table names or the ordinal rule invents is written the same way
//! it was before this change.

use jarde::*;
use std::slice;

/// The `-g:none` class of P3's declaration fixture: a constructor writing through its own
/// uninitialized `this`, an instance method reading a field, a static method reading its parameter.
const HOLDER: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Holder.class");

/// The interface of the same fixture: a `default` method calling an abstract method on its receiver
/// (a receiver read with no debug evidence at all) and a `static` method of its own.
const SHAPE: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Shape.class");

/// The `-g` sample of P3 3.1: its `LocalVariableTable` names slot 0 `this`, and its `receiver(J)J`
/// never reads that slot.
const SCOPE_DEBUG: &[u8] = include_bytes!("fixtures/p3-scope/v8-debug/Scope.class");

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

/// The member of one class, by the raw name its class file declares.
fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's own text in the assembled source.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &member(report, name).text
}

/// The recovery report of one member's own run.
fn run_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` was read by its own run, got {other:?}"),
    }
}

#[test]
fn an_instance_members_receiver_is_written_as_this_where_the_body_reads_it() {
    // `Holder.value()I` is `aload_0; getfield value:I; ireturn`: the receiver is read for its own
    // field, and with no debug table the slot used to be named `arg0`.
    let holder = open(HOLDER);
    let report = class_source_of(&holder, "Holder");
    let value = text_of(&report, "value");
    assert!(value.contains("return this.value;"), "{value}");
    assert!(!value.contains("arg0"), "{value}");
    assert!(!value.contains("this_"), "{value}");
    // The member's own declaration is not part of this change: no receiver is written into the
    // signature and no parameter is renamed.
    assert!(value.contains("public int value()"), "{value}");

    // `Shape.scaled(I)I` — an interface's `default` method — is `aload_0; invokeinterface sides()I;
    // iload_1; imul; ireturn`: the receiver is read as a **call's receiver**, and the parameter above
    // it keeps the ordinal name the same run gives it.
    let shape = open(SHAPE);
    let report = class_source_of(&shape, "Shape");
    let scaled = text_of(&report, "scaled");
    assert!(scaled.contains("return this.sides() * arg1;"), "{scaled}");
    assert!(scaled.contains("public int scaled(int arg1)"), "{scaled}");
    // The abstract member the call names declares no body, and that is unchanged.
    let sides = text_of(&report, "sides");
    assert!(sides.contains("public abstract int sides();"), "{sides}");
}

#[test]
fn a_constructors_own_receiver_is_this_and_its_writes_keep_their_place() {
    // `Holder(int)` is `aload_0; invokespecial Object.<init>; aload_0; iload_1; putfield value:I;
    // return`: the write on its own uninitialized `this` is a **receiver read**, and it stays after
    // the constructor call, where the bytecode put it.
    let holder = open(HOLDER);
    let report = class_source_of(&holder, "Holder");
    let init = text_of(&report, "<init>");
    assert!(init.contains("this.value = arg1;"), "{init}");
    assert!(!init.contains("arg0"), "{init}");
    assert!(init.contains("public Holder(int arg1)"), "{init}");
    let constructor_call = init
        .find("super();")
        .unwrap_or_else(|| panic!("the constructor call is presented:\n{init}"));
    let write = init
        .find("this.value = arg1;")
        .unwrap_or_else(|| panic!("the write through the receiver is presented:\n{init}"));
    assert!(
        constructor_call < write,
        "the write is not moved past the constructor call:\n{init}"
    );
}

#[test]
fn a_static_members_slot_zero_is_still_its_ordinal_parameter() {
    // The counterexample: `Holder.of(int)` reads its own parameter to pass it to `new Holder(…)`,
    // and `Shape.sum(int, int)` reads both of its. Slot 0 of a `static` member is a parameter, so it
    // keeps `arg<slot>` — and no `this` is written in a member that has no receiver.
    let holder = open(HOLDER);
    let report = class_source_of(&holder, "Holder");
    let of = text_of(&report, "of");
    assert!(of.contains("return new Holder(arg0);"), "{of}");
    assert!(!of.contains("this"), "{of}");

    let shape = open(SHAPE);
    let report = class_source_of(&shape, "Shape");
    let sum = text_of(&report, "sum");
    assert!(sum.contains("return arg0 + arg1;"), "{sum}");
    assert!(!sum.contains("this"), "{sum}");
}

#[test]
fn a_receiver_the_body_never_reads_is_not_written_and_is_no_longer_an_alias() {
    // `Scope.receiver(J)J` — from the `-g` sample — is an instance method whose table names slot 0
    // `this`, and whose body (`return a;`) never reads it. The receiver is not written, and the run
    // that states this slot is not a keyword to alias: a receiver is an identity the member's own
    // flags state, not an unspellable source name.
    let scope = open(SCOPE_DEBUG);
    let report = class_source_of(&scope, "Scope");
    let receiver = text_of(&report, "receiver");
    assert!(receiver.contains("return a;"), "{receiver}");
    assert!(!receiver.contains("this"), "{receiver}");
    let run = run_of(&report, "receiver");
    assert!(
        run.aliased_names.is_empty(),
        "the receiver's debug name is not an alias: {:?}",
        run.aliased_names
    );
    assert!(
        !run.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_name_aliased"),
        "{:?}",
        run.diagnostics
    );
    assert_eq!(run.syntax_status, SyntaxStatus::Unchecked);

    // The members beside it are `static` ones and keep the names their table and the ordinal rule
    // give them: the change reaches the receiver and nothing else.
    let simple = text_of(&report, "simple");
    assert!(simple.contains("int x = 5;"), "{simple}");
}
