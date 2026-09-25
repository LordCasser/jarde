//! P3 task 6.7: a member's own `ACC_VARARGS` bit (`0x0080`) writes its **last** parameter as
//! `T...`.
//!
//! The bit is a fact of the member's own `access_flags` and it reaches the declaration of exactly
//! one parameter — the last one — and only when that parameter is an array: `[I` is written
//! `int...` and `[[B` is written `byte[]...` in place of `byte[][]`, which is the element type
//! written as it stands and then the three dots where the last `[]` would have been. Nothing else
//! about the signature moves: the slot the parameter occupies is still the descriptor's own (an
//! array fills one slot) and the body reads that slot as the array it is. No expansion is inferred
//! from the flag — a call site that builds the array (`many(1, 2, 3)`) is out of this presentation's
//! scope — and an array parameter of a member **without** the bit keeps its brackets (`int[] arg0`).
//!
//! The sample is `tests/fixtures/p3-varargs/` (javac 23.0.1 `/usr/bin/javac --release 8 -g:none -d
//! v8 Var.java`; the command is in the README there, and `Var.class` is 255 bytes, SHA-256
//! `90f76ef3…32eaf673`):
//!
//! * `many` is `static int many(int, int...)`, flags `0x0088` (`ACC_STATIC` + `ACC_VARARGS`): its
//!   declaration writes the dots and never `int[] arg1`;
//! * `merge` is `static byte[] merge(byte[]...)`, flags `0x0088`: the descriptor's `[[B` is the
//!   `byte[][]` of the presentation and the last `[]` is the one the dots replace, so the
//!   declaration is `byte[]... arg0` — the element type, then its own `[]`, then the dots;
//! * `plain` is `static int plain(int[])`, flags `0x0008`: the control, an array parameter of a
//!   member that declares no `ACC_VARARGS`, which keeps `int[] arg0`.

use jarde::*;
use std::slice;

/// The varargs sample: `many`, `merge` and `plain` beside the default constructor.
const VAR: &[u8] = include_bytes!("fixtures/p3-varargs/v8/Var.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of a committed sample, under the entry point the CLI calls.
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
        other => panic!("one committed sample answers one definition, got {other:?}"),
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

/// One member's declaration as this presentation published it, without its `;` or its block.
fn member_declaration<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    member(report, name)
        .declaration
        .as_deref()
        .unwrap_or_else(|| panic!("the member `{name}` is spelled"))
}

/// The member's own `ACC_VARARGS` bit (JVMS 4.6-A), as the class file's `access_flags` state it.
fn varargs(report: &ClassSourceReport, name: &str) -> bool {
    member(report, name).item.access_flags & 0x0080 != 0
}

/// The names of the members whose published declaration carries one word.
fn members_carrying(report: &ClassSourceReport, word: &str) -> Vec<String> {
    report
        .methods
        .iter()
        .filter(|method| {
            method
                .declaration
                .as_deref()
                .is_some_and(|declaration| declaration.contains(word))
        })
        .map(|method| String::from_utf8_lossy(&method.item.name.raw().0).into_owned())
        .collect()
}

#[test]
fn a_varargs_members_last_parameter_is_written_with_the_dots() {
    let report = class_source_of(&open(VAR), "Var");
    // The whole presentation, for inspection: the three declarations below are its own lines.
    println!("{}", report.text);

    // The two members whose own flags carry `0x0080` are the two whose last parameter is an array:
    // the bit is the member's own fact, and the flag is read off the item the declaration is
    // spelled from.
    assert!(varargs(&report, "many"));
    assert!(varargs(&report, "merge"));

    // `many` is `(I[I)I`: the last parameter is written `int... arg1` — the array's element type and
    // the dots — and never `int[] arg1`.
    assert_eq!(
        member_declaration(&report, "many"),
        "static int many(int arg0, int... arg1)"
    );
    assert!(
        !text_of(&report, "many").contains("int[]"),
        "the varargs parameter is not spelled as an array:\n{}",
        text_of(&report, "many")
    );

    // `merge` is `([[B)[B`: the dots replace the **last** `[]` of `byte[][]`, so the element type
    // keeps the one bracket it has and the declaration is `byte[]... arg0`.
    assert_eq!(
        member_declaration(&report, "merge"),
        "static byte[] merge(byte[]... arg0)"
    );
    assert!(
        !text_of(&report, "merge").contains("byte[][]"),
        "only the last `[]` is replaced by the dots:\n{}",
        text_of(&report, "merge")
    );

    // The declarations are the class text's own lines, one level in from the class declaration.
    for declaration in [
        "static int many(int arg0, int... arg1)",
        "static byte[] merge(byte[]... arg0)",
        "static int plain(int[] arg0)",
    ] {
        assert!(
            report.text.contains(declaration),
            "the declaration `{declaration}` is a line of the text:\n{}",
            report.text
        );
    }

    // Every member of the table is presented, and the dots appear in exactly the two declarations
    // the members' own flags state them for: nothing else in the sample gains a `...`.
    assert_eq!(report.methods.len(), 4, "every member is presented");
    assert_eq!(members_carrying(&report, "..."), ["many", "merge"]);
}

#[test]
fn an_array_parameter_of_a_member_without_the_flag_keeps_its_brackets() {
    let report = class_source_of(&open(VAR), "Var");

    // `plain` is `([I)I` with flags `0x0008`: no `ACC_VARARGS`, so its array parameter keeps the
    // spelling the descriptor states, and the member is the one control the sample holds for it.
    assert!(!varargs(&report, "plain"));
    assert_eq!(
        member_declaration(&report, "plain"),
        "static int plain(int[] arg0)"
    );

    // The constructor declares no flag and no array either: the two members above are the only
    // ones the dots belong to, and no member without the flag is written with one.
    assert!(!varargs(&report, "<init>"));
    assert_eq!(member_declaration(&report, "<init>"), "public Var()");
    assert_eq!(
        report
            .methods
            .iter()
            .filter(|method| method.item.access_flags & 0x0080 == 0)
            .filter(|method| {
                method
                    .declaration
                    .as_deref()
                    .is_some_and(|declaration| declaration.contains("..."))
            })
            .map(|method| String::from_utf8_lossy(&method.item.name.raw().0).into_owned())
            .collect::<Vec<_>>(),
        Vec::<String>::new(),
        "a member without `ACC_VARARGS` declares no parameter the dots spell"
    );
}
