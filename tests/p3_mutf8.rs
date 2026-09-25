//! P3 6.4: a string constant is read as Modified UTF-8, so a NUL stays a NUL.
//!
//! A class file writes the string `"a\u0000b"` as the constant-pool bytes `61 C0 80 62` — U+0000 is
//! the **two-byte** form Modified UTF-8 gives it (JVMS 4.4.7), never a raw zero byte. That form is
//! the overlong UTF-8 spelling of U+0000, so the standard-UTF-8 read this replaces (`lossy()` was
//! `String::from_utf8_lossy`) refused it and made **two** replacement characters of the one NUL:
//! `return "a\ufffd\ufffdb";` for a class file that states `return "a\u0000b";`. `escape_string`
//! already writes a real U+0000 as `\u0000` — the defect was that no U+0000 ever reached it.
//!
//! The text below pins the fix over the class bytes committed in `tests/fixtures/p3-mutf8/` (see its
//! `README.md` for the compiler, the command, the 196 bytes and the SHA-256). The fixture's own
//! `Utf8` entry is `61 C0 80 62`, so the member's text is the acceptance row itself: it holds
//! `\u0000` and holds no U+FFFD.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 196 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-mutf8/v8/Controls.class");

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

#[test]
fn a_nul_in_a_string_constant_is_presented_as_the_nul_it_is() {
    // The fixture's `#8 = Utf8` entry is `61 C0 80 62`, so `controls` is one `ldc` of one string
    // constant holding one NUL. The acceptance row is the text: it contains `\u0000` and contains no
    // replacement character at all — before the fix the same bytes answered `"a\ufffd\ufffdb"`.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Controls");
    let text = text_of(&report, "controls");
    assert!(
        text.contains("return \"a\\u0000b\";"),
        "the statement is the string constant `escape_string` writes: one NUL as `\\u0000`:\n{text}"
    );
    assert!(
        text.contains("\\u0000"),
        "the recovered text of `controls` holds `\\u0000`:\n{text}"
    );
    assert!(
        !text.contains('\u{fffd}'),
        "no sequence of the constant was read as a replacement character:\n{text}"
    );
    assert!(
        text.matches("\\u0000").count() == 1,
        "one NUL, not two replacements and not a second escape:\n{text}"
    );
}
