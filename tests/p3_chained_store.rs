//! P3 2c.14: the `dup` in front of two local stores is a chained assignment, written once.
//!
//! `int a; int b; a = b = n;` is `iload_0; dup; istore_2; istore_1`: the copy leaves the value on the
//! stack twice and each store takes one copy. Every `Operation::Duplicate` used to be a stated gap,
//! and a store whose value traced back to one was quoted as a value "from an Duplicate at BCI 1", so
//! the member kept a `return local1 + local2;` that read two locals **nothing had declared** — text
//! no compiler accepts.
//!
//! When the two instructions right after a copy are two stores of local slots and the two of them
//! take the two copies it produced, the value is written once: the first store writes the expression
//! the copy duplicated and the second reads the local the first one filled. Writing the expression
//! into both stores would evaluate it twice, which is a program the bytecode does not have — and
//! the neighboring `fill3` is a different meaning of `dup`: its `newarray; dup; index; value;
//! iastore` chain belongs to the array-initializer rule and must never be mistaken for two local
//! stores. Treating each copy as a separate `new int[…]` would make three allocations.
//!
//! The texts below pin the shape over the class bytes committed in `tests/fixtures/p3-chained-store/`
//! (see its `README.md` for the command, the version and the digest). `javac` stores the source's
//! `b` first (`istore_2`) and `a` second (`istore_1`), so the first store the text writes is
//! `local2 = arg0` and the second is `local1 = local2` — the two locals the `return` reads, each
//! declared before it.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 226 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-chained-store/v8/Chain.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls.
///
/// The optional evidence is selected in full, because one of the assertions below is about the
/// segment table: which text accounts for the copy's own bytecode.
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
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
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

/// Where one substring sits in one member's text, stated as the panic a missing one deserves.
fn at(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("the text presents `{needle}`:\n{text}"))
}

#[test]
fn a_dup_in_front_of_two_local_stores_is_written_as_one_value() {
    // `chain(I)I` is `iload_0; dup; istore_2; istore_1; iload_1; iload_2; iadd; ireturn`. Before this
    // change BCI 1 was quoted (`belongs to no shape this run verified`) and BCIs 2 and 3 were quoted
    // as values "from an Duplicate at BCI 1": the body's only statement was a `return` reading two
    // locals the text never declared. Now the first store writes the duplicated expression and the
    // second reads the local the first one filled.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Chain");
    let text = text_of(&report, "chain");
    // The stores, as the two statements the bytecode's order writes: `javac` stores `b` (slot 2)
    // first, so the store that takes the value the copy left on top writes it and the store below
    // reads the local that store filled.
    let first = at(text, "local2 = arg0;");
    let second = at(text, "local1 = local2;");
    let returned = at(text, "return local1 + local2;");
    assert!(
        first < second && second < returned,
        "the value is written once, the local the store filled carries it to the other store, and \
         the `return` reads the two locals in slot order:\n{text}"
    );
    // Both locals are declared, once each, and both declarations precede the `return` that reads
    // them: a `return` over a local nothing declared is the text this change exists to remove.
    let declared2 = at(text, "int local2");
    let declared1 = at(text, "int local1");
    assert!(
        declared2 < second && declared1 < second && first < returned && declared1 < returned,
        "neither store reads a local before its own declaration:\n{text}"
    );
    assert_eq!(
        text.matches("int local1").count(),
        1,
        "the second store's local is declared once:\n{text}"
    );
    assert_eq!(
        text.matches("int local2").count(),
        1,
        "the first store's local is declared once:\n{text}"
    );
    assert!(
        !text.contains("belongs to no shape"),
        "the copy is the shape's own instruction and is presented with it:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "no instruction of this member is left to a quote:\n{text}"
    );
    assert_eq!(
        run_of(&report, "chain").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
    // The copy writes no statement of its own, so its bytecode is accounted for by the text that
    // presents its value: BCI 1 is a **derived** anchor of the first store's expression, and each of
    // the two stores anchors its own statement directly.
    let run = run_of(&report, "chain");
    assert!(
        !run.source_map.derived_of_bci(1).is_empty(),
        "the copy at BCI 1 stays mapped, as a derived anchor of the value it presents: {:?}\n{text}",
        run.source_map.segments()
    );
    for store in [2u32, 3] {
        assert!(
            !run.source_map.direct_of_bci(store).is_empty(),
            "the store at BCI {store} anchors its own statement: {:?}\n{text}",
            run.source_map.segments()
        );
    }
}

#[test]
fn an_array_initializer_copy_is_owned_by_its_separate_rule() {
    // `fill3()[I` is one complete initializer chain. Its three `dup`s are followed by an index,
    // value and `iastore`, so none matches `chained_pair`'s adjacent pair of local stores. The
    // array-initializer rule claims the whole chain as one allocation instead.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Chain");
    let text = text_of(&report, "fill3");
    assert!(
        text.contains("return new int[]{1, 2, 3};"),
        "the initializer rule writes one allocation with all three values:\n{text}"
    );
    assert!(
        !text.contains("@bytecode") && !text.contains("local"),
        "the complete chain is neither quoted nor mispresented as local assignments:\n{text}"
    );
    let run = run_of(&report, "fill3");
    assert_eq!(
        run.content,
        RecoveryContent::ContainsStatements,
        "the recovered initializer is the method's statement:\n{text}"
    );
    for bci in [0u32, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] {
        assert!(
            !run.source_map.of_bci(bci).is_empty(),
            "initializer BCI {bci} is claimed by the complete expression: {:?}\n{text}",
            run.source_map.segments()
        );
    }
}
