//! P3 2c.10/2c.18: one instance field's `++` is one update, written where its value is returned.
//!
//! `return n++;` and `return ++n;` are eight instructions each, and they differ only in where the
//! `dup_x1` sits: the post-increment copies the field's **old** value before the `iadd` and leaves
//! that copy for the `ireturn`, the pre-increment adds first and copies the **sum**. Both were
//! presented as nothing at all: the receiver's `dup` was quoted (`belongs to no shape this run
//! verified`), the `dup_x1` was "not part of the provable subset", and the `putfield` and the
//! `ireturn` quoted values "from an Other at BCI 5" (or 7) — so the update the body performs and
//! the value it returns existed in no statement.
//!
//! The shape the change reads is exactly those eight consecutive instructions, and the two updates
//! are told apart by an identity of values rather than by the order of the instructions: the copy
//! duplicates what the `getfield` read (a post-increment, whose leftover is the old value) or what
//! the `iadd` produced (a pre-increment, whose leftover is the sum). What makes the shape sound is
//! the rest of its evidence: the receiver's `dup` copies the receiver load, that copy is the
//! receiver the `getfield` reads *and* the value under the one the `dup_x1` duplicates, the
//! `getfield` and the `putfield` were claimed by `field@1` for one member, the constant is `1`, the
//! `iadd` adds it to the field's value, and the `putfield` stores the update's own result.
//!
//! Every other `dup` and `dup_x1` keeps the quote it had. The two-local-store copy of `x = y = n`
//! and the array initializer's copies are pinned by `tests/p3_chained_store.rs`, which the command
//! below runs beside this one: `local2 = arg0; local1 = local2` stays one value, and `fill3` stays
//! an explanation with fewer than three `new int[…]` expressions.
//!
//! The texts below pin the shape over the class bytes committed in `tests/fixtures/p3-field-increment/`
//! (see its `README.md` for the command, the version and the digest).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 240 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-field-increment/v8/Bump.class");

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
/// The optional evidence is selected in full, because the assertions below read the segment table:
/// which text accounts for which instruction of the shape.
fn class_source_with_evidence(
    snapshot: &ArtifactSnapshot,
    name: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
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
        .class_source_with_evidence(slice::from_ref(snapshot), &request, evidence, &mut budget())
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

fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    class_source_with_evidence(snapshot, name, &RecoveryEvidenceRequest::all())
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

/// The recovery report of one member's own run.
fn run_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
        .outcome
    {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` was read by its own run, got {other:?}"),
    }
}

/// The bytecode indices both shapes state (`javap -c`), by their meaning in this sample.
///
/// `post()` is `0 aload_0; 1 dup; 2 getfield n; 5 dup_x1; 6 iconst_1; 7 iadd; 8 putfield n;
/// 11 ireturn` and `pre()` is the same eight with the `dup_x1` at 7 and the `iconst_1; iadd` at
/// 5, 6 — so the `putfield` that performs the update and the `ireturn` that states it are the same
/// two instructions in both, and the six others are the ones the update's text presents.
const UPDATE: u32 = 8;
const RETURN: u32 = 11;

/// The instructions of one shape other than its `putfield` and its `ireturn`, as BCIs.
const PRESENTED: [u32; 6] = [0, 1, 2, 5, 6, 7];

#[test]
fn a_post_increment_returns_the_old_value_and_a_pre_increment_the_sum() {
    // `post()` is `aload_0; dup; getfield n; dup_x1; iconst_1; iadd; putfield n; ireturn`: the copy
    // duplicates the field's old value and the `ireturn` reads that copy, so the update is `n++` —
    // the field is written once with the sum, and the method returns what the field held before it.
    // `pre()` moves the `dup_x1` behind the sum: the copy duplicates the sum and the `ireturn`
    // reads that, so the update is `++n`, written without reading the field a second time.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Bump");
    let post = text_of(&report, "post");
    let pre = text_of(&report, "pre");
    assert!(post.contains("return this.n++;"), "{post}");
    assert!(pre.contains("return ++this.n;"), "{pre}");
    // The two updates are two different programs, and neither may be written as the other: the
    // pre-increment returns the value it computed, the post-increment the one it replaced.
    assert!(!post.contains("++this.n"), "{post}");
    assert!(!pre.contains("this.n++"), "{pre}");
    for text in [post, pre] {
        // The update is one statement, not a field assignment beside the `return`: `this.n = …`
        // would be a second write of the field (and a different value returned).
        assert!(!text.contains("this.n = "), "{text}");
        // Nothing is put in a local to carry the old value: the text has no local of its own.
        assert!(!text.contains("local"), "{text}");
        // No instruction of either member is quoted: the shape's eight instructions are all
        // presented by the statement.
        assert!(!text.contains("@bytecode"), "{text}");
        assert!(!text.contains("belongs to no shape"), "{text}");
    }
    assert_eq!(
        run_of(&report, "post").content,
        RecoveryContent::ContainsStatements,
        "{post}"
    );
    assert_eq!(
        run_of(&report, "pre").content,
        RecoveryContent::ContainsStatements,
        "{pre}"
    );
}

#[test]
fn the_update_is_the_text_the_instructions_that_perform_it_are_anchored_at() {
    // The statement writes the update where the bytecode performs it: the expression's text is
    // anchored **directly** at the `putfield` (the instruction that writes the field), and the six
    // instructions it presents — the receiver load, its copy, the `getfield`, the constant, the
    // `iadd` and the copy the returned value comes from — stay mapped as its derived anchors, so no
    // instruction of the shape is left unaccounted for. The statement's own `return` is anchored at
    // the `ireturn`, exactly as every other return of this layer is.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Bump");
    for (name, update_text) in [("post", "this.n++"), ("pre", "++this.n")] {
        let run = run_of(&report, name);
        assert_eq!(
            run.fields
                .iter()
                .map(|field| (field.bci, field.access, field.presented))
                .collect::<Vec<_>>(),
            vec![(2, "read", true), (8, "write", true)],
            "the proved update's read and write both survive in the final return: {}",
            run.text,
        );
        // The table is mapped over the member's own body text, which is the text the run wrote.
        let text = run.text.as_str();
        let update = run.source_map.direct_of_bci(UPDATE);
        assert_eq!(
            update.len(),
            1,
            "one node's own anchor is the `putfield`: {:?}\n{text}",
            run.source_map.segments()
        );
        assert_eq!(
            update[0].text(text),
            update_text,
            "the `putfield`'s own text is the update it performs: {:?}\n{text}",
            run.source_map.segments()
        );
        for bci in PRESENTED {
            assert!(
                !run.source_map.derived_of_bci(bci).is_empty(),
                "the instruction at BCI {bci} stays mapped as a derived anchor of the text that \
                 presents it: {:?}\n{text}",
                run.source_map.segments()
            );
        }
        assert!(
            !run.source_map.direct_of_bci(RETURN).is_empty(),
            "the `return` is anchored at the `ireturn` that runs it: {:?}\n{text}",
            run.source_map.segments()
        );
    }
}

#[test]
fn field_summary_is_independent_of_details_and_driver_range() {
    let sample = open(SAMPLE);
    let essential =
        class_source_with_evidence(&sample, "Bump", &RecoveryEvidenceRequest::essential());
    let full = class_source_of(&sample, "Bump");
    let range = class_source_with_evidence(
        &sample,
        "Bump",
        &RecoveryEvidenceRequest::all().with_driver_bci_range(BytecodeRange::new(2, 5)),
    );
    for name in ["post", "pre"] {
        let summary = |source: &ClassSourceReport| {
            run_of(source, name)
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code == "jre_field_accesses")
                .map(|diagnostic| diagnostic.message.clone())
                .unwrap_or_else(|| {
                    panic!(
                        "field summary exists: {:?}",
                        run_of(source, name).diagnostics
                    )
                })
        };
        assert_eq!(summary(&essential), summary(&full));
        assert_eq!(summary(&range), summary(&full));
        assert!(summary(&full).contains("2 presented, 0 refused"));
        assert!(run_of(&essential, name).fields.is_empty());
        assert_eq!(run_of(&full, name).fields.len(), 2);
        assert_eq!(
            run_of(&range, name)
                .fields
                .iter()
                .map(|field| field.bci)
                .collect::<Vec<_>>(),
            [2]
        );
    }
}
