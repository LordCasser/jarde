//! `recover-twr-javac8-close-sequence`: the JDK 8 `try`-with-resources closing sequence — a blank
//! primary copy (`aconst_null; astore p`), the rethrow relay the body's own row runs through, and
//! every close guarded twice — recovers as the source `try (T n = …) { … }`, while the newer
//! lowering's presentation and every weakened geometry of the older one keep the answers they had.
//!
//! The two legs are the patrol's own frozen fixtures (one `TR.java`, compiled by Corretto 1.8.0_432
//! and by javac 23 `--release 8`):
//! `openspec/evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/fixture/`.

use jarde::*;
use std::slice;

const REAL_JAVAC8: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/fixture/real-javac8-TR/TR.class"
);
const JAVAC23: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/fixture/javac23-TR/TR.class"
);
/// The pre-change presentation of the javac 23 leg, frozen before this slice was written
/// (`class-source --format text`, content and bookkeeping planes in one document).
const JAVAC23_RENDERED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/results/TR-javac23-rendered.txt"
);

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn class_source(bytes: &[u8]) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("a standalone class fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("TR"),
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
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone fixture has {} matching definitions",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "an ample class-source request stopped with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no method `{name}` in the fixture"))
}

/// The member's own text, as the assembled source writes it.
fn method_text<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    member(report, name).text.as_str()
}

/// Whether the member still carries the presentation's "not recovered" marker.
fn is_refused(report: &ClassSourceReport, name: &str) -> bool {
    method_text(report, name).contains("// jarde: not recovered:")
}

#[test]
fn the_older_lowerings_level_is_recovered_as_a_try_with_resources() {
    let report = class_source(REAL_JAVAC8);
    let one = method_text(&report, "one");
    assert!(
        !is_refused(&report, "one"),
        "the JDK 8 closing sequence recovers: {one}"
    );
    assert!(
        one.contains("try (TR local1 = new TR(arg0)) {"),
        "the header declares the resource the compiler filled:\n{one}"
    );
    assert!(
        one.contains("return local3;"),
        "the body's own return stays inside the braces:\n{one}"
    );
    // The compiler's own machinery — the blank primary copy, the relay, the double guards, the
    // suppression — is what the header and the braces replaced: none of it is source text.
    for absent in [
        "aconst_null",
        "addSuppressed",
        "ifnull",
        "athrow",
        "primar",
        "@bytecode",
    ] {
        assert!(
            !one.contains(absent),
            "the presentation states no `{absent}`:\n{one}"
        );
    }
    // The double-resource statement of the same fixture is outside this slice: it keeps the
    // refusal it had (the patrol's own MVP boundary).
    assert!(
        is_refused(&report, "two"),
        "the two-resource statement keeps its refusal:\n{}",
        method_text(&report, "two")
    );
}

#[test]
fn the_newer_lowerings_presentation_is_byte_for_byte_the_pre_change_one() {
    let report = class_source(JAVAC23);
    let rendered = report.text.as_bytes();
    assert!(
        JAVAC23_RENDERED.starts_with(rendered),
        "the javac 23 leg's content moved:\n{}",
        report.text
    );
}

#[test]
fn a_weakened_geometry_keeps_the_refusal() {
    // Each probe weakens exactly one of the facts the older lowering's proof reads, on the frozen
    // fixture itself: the body's catch-all row (the row that carries the relay's rethrow into the
    // close handler), and the suppression chain's head (the handler that rethrow reaches).
    let weakened: [(&str, fn(Vec<u8>) -> Vec<u8>); 2] = [
        ("body catch-all row typed", |bytes| {
            patch_row(&bytes, 2, |row| {
                row[6..8].copy_from_slice(&15u16.to_be_bytes())
            })
        }),
        ("suppression chain head cut", |bytes| {
            patch_row(&bytes, 4, |row| {
                row[4..6].copy_from_slice(&85u16.to_be_bytes())
            })
        }),
    ];
    for (name, weaken) in weakened {
        let report = class_source(&weaken(REAL_JAVAC8.to_vec()));
        assert!(
            is_refused(&report, "one"),
            "{name}: the weakened geometry stays refused:\n{}",
            method_text(&report, "one")
        );
    }
}

#[test]
fn swapped_close_guard_targets_keep_the_refusal() {
    // The two `ifnull`s of the normal path's close group, with their targets exchanged: the
    // resource's guard jumps to the plain close and the lead's guard to the exit, so neither arm
    // rejoins the exit the proof read.
    let bytes = swap_guard_targets(&REAL_JAVAC8.to_vec(), 17, 21, 46, 42);
    let report = class_source(&bytes);
    assert!(
        is_refused(&report, "one"),
        "the swapped guard targets stay refused:\n{}",
        method_text(&report, "one")
    );
}

/// One exception-table row of `TR.one()`'s `Code` attribute, patched in place.
fn patch_row(bytes: &[u8], ordinal: usize, patch: impl FnOnce(&mut [u8; 8])) -> Vec<u8> {
    let table = one_code(bytes).1;
    let mut row = [0u8; 8];
    row.copy_from_slice(&bytes[table + ordinal * 8..table + ordinal * 8 + 8]);
    patch(&mut row);
    let mut patched = bytes.to_vec();
    patched[table + ordinal * 8..table + ordinal * 8 + 8].copy_from_slice(&row);
    patched
}

/// The two branch instructions' 16-bit operands exchanged, in `TR.one()`'s code: `first` and
/// `second` are the branch BCIs, `first_target` and `second_target` the BCIs they currently reach.
fn swap_guard_targets(
    bytes: &[u8],
    first: u32,
    second: u32,
    first_target: u32,
    second_target: u32,
) -> Vec<u8> {
    let code = one_code(bytes).0;
    let mut patched = bytes.to_vec();
    for (at, target) in [(first, second_target), (second, first_target)] {
        let offset = i64::from(target) - i64::from(at);
        let offset = i16::try_from(offset).expect("a branch target within one method");
        let at = code + at as usize + 1;
        patched[at..at + 2].copy_from_slice(&offset.to_be_bytes());
    }
    patched
}

/// The file offsets of `TR.one()`'s own code, and of its exception table.
fn one_code(bytes: &[u8]) -> (usize, usize) {
    fn u2(bytes: &[u8], at: usize) -> u16 {
        u16::from_be_bytes([bytes[at], bytes[at + 1]])
    }
    fn u4(bytes: &[u8], at: usize) -> usize {
        u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]) as usize
    }
    let mut at = 8;
    let cp_count = u2(bytes, at);
    at += 2;
    let mut utf8: Vec<Option<String>> = vec![None; cp_count as usize];
    let mut index = 1;
    while index < cp_count {
        let tag = bytes[at];
        at += 1;
        match tag {
            1 => {
                let length = u2(bytes, at) as usize;
                at += 2;
                utf8[index as usize] =
                    Some(String::from_utf8_lossy(&bytes[at..at + length]).into_owned());
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
    let skip_member = |at: &mut usize| {
        let attributes = u2(bytes, *at + 6);
        *at += 8;
        for _ in 0..attributes {
            *at += 6 + u4(bytes, *at + 2);
        }
    };
    let fields = u2(bytes, at);
    at += 2;
    for _ in 0..fields {
        skip_member(&mut at);
    }
    let methods = u2(bytes, at);
    at += 2;
    for _ in 0..methods {
        let name = utf8[u2(bytes, at + 2) as usize].clone();
        let descriptor = utf8[u2(bytes, at + 4) as usize].clone();
        let attributes = u2(bytes, at + 6);
        at += 8;
        for _ in 0..attributes {
            let attribute = utf8[u2(bytes, at) as usize].clone();
            let length = u4(bytes, at + 2);
            let body = at + 6;
            if name.as_deref() == Some("one")
                && descriptor.as_deref() == Some("(Ljava/lang/String;)Ljava/lang/String;")
                && attribute.as_deref() == Some("Code")
            {
                let code_length = u4(bytes, body + 4);
                let code = body + 8;
                let count = u2(bytes, code + code_length);
                assert_eq!(count, 5, "`one()` has the five rows the patrol recorded");
                return (code, code + code_length + 2);
            }
            at = body + length;
        }
    }
    panic!("`one(Ljava/lang/String;)Ljava/lang/String;` has a `Code` attribute")
}
