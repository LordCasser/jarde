//! The class-source presentation: one class file assembled into Java text, and the facts that text
//! was spelled from.
//!
//! The contract this file holds has three halves, and each is checked against the library's own
//! values rather than against the text alone:
//!
//! * **one class, one read, one run per member.** The declaration and the member tables are the
//!   class view's own read, and every member body is one single-method recovery — so the report
//!   publishes the very `RecoveryReport` of that run, and a member that declares no body charges no
//!   run at all.
//! * **nothing is disguised.** A member with no `Code`, a member whose run stopped, a member whose
//!   artifact holds no statement and a member whose descriptor cannot be read are each marked in the
//!   text with the same `// jarde:` prefix the report publishes per member, and no empty body is ever
//!   written for a body that was not recovered.
//! * **a member's failure is that member's.** The members beside a stopped one are presented, the
//!   class report's execution plane is non-`Complete`, and a name that several definitions answer to
//!   presents nothing at all.
//!
//! The two committed samples are read as they are (a real compiled class and the refusal sample),
//! and the crafted probe is built here so a case can pin the exact member shapes it is about.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;

const STORE: u16 = 0;

/// The real compiled sample: three members, one of them explanation-only.
const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

/// The committed refusal sample: three of its eight members produce explanation-only artifacts.
const REFUSED_CAST: &[u8] = include_bytes!("fixtures/p3-refused-cast/v8/RefusedCast.class");

// ---------------------------------------------------------------------------------------------
// Fixtures: one class-file builder and one stored-only archive writer
// ---------------------------------------------------------------------------------------------

/// One constant pool that interns each entry once, so a fixture's indices are stable.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn intern(&mut self, entry: Vec<u8>) -> u16 {
        if let Some(index) = self.entries.iter().position(|existing| *existing == entry) {
            return u16::try_from(index + 1).expect("the fixture pool index fits u16");
        }
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("the fixture pool index fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("the fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        self.intern(entry)
    }

    fn class(&mut self, name: &[u8]) -> u16 {
        let name = self.utf8(name);
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.intern(entry)
    }
}

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

/// One field of a fixture class.
struct FieldSpec<'a> {
    flags: u16,
    name: &'a [u8],
    descriptor: &'a [u8],
}

/// One method of a fixture class: its flags, its name and descriptor, and the instructions of its
/// `Code` attribute — or `None` for a member that declares no `Code` at all.
struct MethodSpec<'a> {
    flags: u16,
    name: &'a [u8],
    descriptor: &'a [u8],
    code: Option<&'a [u8]>,
}

/// One class file, written the way the reader reads it.
fn class_file(
    this_class: &[u8],
    super_class: &[u8],
    interfaces: &[&[u8]],
    flags: u16,
    fields: &[FieldSpec<'_>],
    methods: &[MethodSpec<'_>],
) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_index = pool.class(this_class);
    let super_index = pool.class(super_class);
    let interface_indices: Vec<u16> = interfaces.iter().map(|name| pool.class(name)).collect();
    let code_name = pool.utf8(b"Code");
    let field_indices: Vec<(u16, u16)> = fields
        .iter()
        .map(|field| (pool.utf8(field.name), pool.utf8(field.descriptor)))
        .collect();
    let method_indices: Vec<(u16, u16)> = methods
        .iter()
        .map(|method| (pool.utf8(method.name), pool.utf8(method.descriptor)))
        .collect();

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(
        &mut output,
        u16::try_from(pool.entries.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool.entries {
        output.extend_from_slice(entry);
    }
    u16b(&mut output, flags);
    u16b(&mut output, this_index);
    u16b(&mut output, super_index);
    u16b(
        &mut output,
        u16::try_from(interface_indices.len()).expect("the fixture interfaces fit u16"),
    );
    for index in interface_indices {
        u16b(&mut output, index);
    }
    u16b(
        &mut output,
        u16::try_from(fields.len()).expect("the fixture fields fit u16"),
    );
    for (index, field) in fields.iter().enumerate() {
        u16b(&mut output, field.flags);
        u16b(&mut output, field_indices[index].0);
        u16b(&mut output, field_indices[index].1);
        u16b(&mut output, 0);
    }
    u16b(
        &mut output,
        u16::try_from(methods.len()).expect("the fixture methods fit u16"),
    );
    for (index, method) in methods.iter().enumerate() {
        u16b(&mut output, method.flags);
        u16b(&mut output, method_indices[index].0);
        u16b(&mut output, method_indices[index].1);
        let Some(code) = method.code else {
            u16b(&mut output, 0);
            continue;
        };
        let mut body = Vec::new();
        u16b(&mut body, 2);
        u16b(&mut body, 4);
        u32b(
            &mut body,
            u32::try_from(code.len()).expect("the fixture body fits u32"),
        );
        body.extend_from_slice(code);
        u16b(&mut body, 0);
        u16b(&mut body, 0);
        u16b(&mut output, 1);
        u16b(&mut output, code_name);
        u32b(
            &mut output,
            u32::try_from(body.len()).expect("the fixture attribute fits u32"),
        );
        output.extend_from_slice(&body);
    }
    u16b(&mut output, 0);
    output
}

/// `iconst_1; pop; return`: a body whose recovery produces a statement.
const PLAIN_BODY: &[u8] = &[0x04, 0x57, 0xb1];

/// `new` with an incomplete index: a body whose own decode stops inside it, which is what makes the
/// analysis of that member non-`Complete`.
const DAMAGED_BODY: &[u8] = &[0xbb, 0x00];

const CLASS_FLAGS: u16 = 0x0021;
const PUBLIC_METHOD: u16 = 0x0001;
const PUBLIC_STATIC_METHOD: u16 = 0x0009;
const PUBLIC_ABSTRACT_METHOD: u16 = 0x0401;
const PUBLIC_NATIVE_METHOD: u16 = 0x0101;
const PUBLIC_STATIC_FINAL_FIELD: u16 = 0x0019;

/// One class with every member shape this presentation has to state: bodies it recovers, a body
/// whose analysis stops, a third body behind them (so a request that ends early leaves a member it
/// never began), a member with no `Code` for each of the three ways that can be said, a field, and
/// two interfaces.
fn probe_class() -> Vec<u8> {
    class_file(
        b"p/Probe",
        b"java/lang/Object",
        &[b"p/Marker", b"java/io/Serializable"],
        CLASS_FLAGS,
        &[FieldSpec {
            flags: PUBLIC_STATIC_FINAL_FIELD,
            name: b"value",
            descriptor: b"I",
        }],
        &[
            MethodSpec {
                flags: PUBLIC_STATIC_METHOD,
                name: b"good",
                descriptor: b"()V",
                code: Some(PLAIN_BODY),
            },
            MethodSpec {
                flags: PUBLIC_STATIC_METHOD,
                name: b"broken",
                descriptor: b"()V",
                code: Some(DAMAGED_BODY),
            },
            MethodSpec {
                flags: PUBLIC_STATIC_METHOD,
                name: b"third",
                descriptor: b"()V",
                code: Some(PLAIN_BODY),
            },
            MethodSpec {
                flags: PUBLIC_ABSTRACT_METHOD,
                name: b"abstractOne",
                descriptor: b"()V",
                code: None,
            },
            MethodSpec {
                flags: PUBLIC_NATIVE_METHOD,
                name: b"nativeOne",
                descriptor: b"()V",
                code: None,
            },
            MethodSpec {
                flags: PUBLIC_METHOD,
                name: b"contradictory",
                descriptor: b"()V",
                code: None,
            },
        ],
    )
}

/// One class file whose field table declares a field and then stops inside its record: the class
/// itself is readable, the members behind the stop are not, and the report has to state that rather
/// than present a shorter class as a whole one.
fn truncated_member_table_class() -> Vec<u8> {
    let mut pool = Pool::default();
    let this_index = pool.class(b"p/Truncated");
    let super_index = pool.class(b"java/lang/Object");
    let name = pool.utf8(b"value");
    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(
        &mut output,
        u16::try_from(pool.entries.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool.entries {
        output.extend_from_slice(entry);
    }
    u16b(&mut output, CLASS_FLAGS);
    u16b(&mut output, this_index);
    u16b(&mut output, super_index);
    u16b(&mut output, 0);
    u16b(&mut output, 1);
    u16b(&mut output, PUBLIC_STATIC_FINAL_FIELD);
    u16b(&mut output, name);
    output
}

/// One class with `bodies` members that declare a body and one that declares none, so a case can
/// compare two classes whose member counts differ and whose bodies are all recoverable.
///
/// The no-body member is the `abstract` shape this presentation states as a declaration: it is what
/// makes "one preparation + one decode per body" a statement about bodies rather than about records.
fn many_bodies_class(name: &[u8], bodies: usize) -> Vec<u8> {
    let names: Vec<String> = (0..bodies).map(|index| format!("body{index}")).collect();
    let mut methods: Vec<MethodSpec<'_>> = names
        .iter()
        .map(|member| MethodSpec {
            flags: PUBLIC_STATIC_METHOD,
            name: member.as_bytes(),
            descriptor: b"()V",
            code: Some(PLAIN_BODY),
        })
        .collect();
    methods.push(MethodSpec {
        flags: PUBLIC_ABSTRACT_METHOD,
        name: b"declaredOnly",
        descriptor: b"()V",
        code: None,
    });
    class_file(name, b"java/lang/Object", &[], CLASS_FLAGS, &[], &methods)
}

/// One class whose **method** table declares two records and stops inside the second: the first
/// member is a complete record with a valid body, the second is a name and a flags field with no
/// descriptor after them.
///
/// This is the shape a damaged member table has when there is a readable body in front of it, which
/// is what a presentation owes an honest answer about: the prefix member is a member this class
/// declares, and "its body could not be decoded" and "this class declares no such member" are two
/// different statements.
fn stopped_method_table_class() -> Vec<u8> {
    let mut pool = Pool::default();
    let this_index = pool.class(b"p/Stopped");
    let super_index = pool.class(b"java/lang/Object");
    let code_name = pool.utf8(b"Code");
    let good_name = pool.utf8(b"good");
    let descriptor = pool.utf8(b"()V");
    let second_name = pool.utf8(b"second");
    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(
        &mut output,
        u16::try_from(pool.entries.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool.entries {
        output.extend_from_slice(entry);
    }
    u16b(&mut output, CLASS_FLAGS);
    u16b(&mut output, this_index);
    u16b(&mut output, super_index);
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(&mut output, 2); // two method records declared
    // methods[0]: a complete record with a `Code` attribute whose body is a real statement.
    let mut body = Vec::new();
    u16b(&mut body, 2);
    u16b(&mut body, 4);
    u32b(
        &mut body,
        u32::try_from(PLAIN_BODY.len()).expect("the fixture body fits u32"),
    );
    body.extend_from_slice(PLAIN_BODY);
    u16b(&mut body, 0);
    u16b(&mut body, 0);
    u16b(&mut output, PUBLIC_STATIC_METHOD);
    u16b(&mut output, good_name);
    u16b(&mut output, descriptor);
    u16b(&mut output, 1);
    u16b(&mut output, code_name);
    u32b(
        &mut output,
        u32::try_from(body.len()).expect("the fixture attribute fits u32"),
    );
    output.extend_from_slice(&body);
    // methods[1]: flags and a name, and then the file ends.
    u16b(&mut output, PUBLIC_STATIC_METHOD);
    u16b(&mut output, second_name);
    output
}

/// A stored-only archive, built with the repository's own `rawzip` dev-dependency.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is written");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

// ---------------------------------------------------------------------------------------------
// The library side of every case
// ---------------------------------------------------------------------------------------------

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

/// The code one request-level refusal carries.
fn error_code(error: &Error) -> String {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
        other => panic!("unexpected error {other:?}"),
    }
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("the fixture snapshot opens")
}

fn performed<T>(outcome: OperationOutcome<T>) -> T {
    match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "expected one bound class, got {} candidate(s) and no execution",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "expected one bound class, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

/// One class-source request over `snapshot`, under an explicit policy.
fn request(
    snapshot: &ArtifactSnapshot,
    class: ClassRef,
    policy: EnvironmentPolicy,
) -> ClassSourceRequest {
    ClassSourceRequest {
        class,
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    }
}

/// The same request over an explicit scope rather than the whole snapshot.
fn scoped_request(
    snapshot: &ArtifactSnapshot,
    class: ClassRef,
    policy: EnvironmentPolicy,
    scope: PhysicalScope,
) -> ClassSourceRequest {
    let mut request = request(snapshot, class, policy);
    request.environment.scope = scope;
    request
}

/// The usage one execution plane carries, whichever terminal state it states.
fn usage_of(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

/// The container origin of one snapshot's own root container, as the artifact-tree enumeration
/// states it: a snapshot's root container is the snapshot itself.
fn root_origin(snapshot: &ArtifactSnapshot) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.id().clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
    }
}

/// The declared position of one nested container, addressed by the leaf entry that reaches it: the
/// origin comes from the public artifact-tree enumeration, which is the way a caller learns one.
fn nested_root(snapshot: &ArtifactSnapshot, leaf: &[u8]) -> LoadRoot {
    let tree = Engine::new()
        .enumerate_artifact_tree(snapshot, &mut budget())
        .expect("the fixture tree enumerates");
    let origin = tree
        .containers
        .iter()
        .filter(|container| {
            container
                .origin
                .steps
                .last()
                .is_some_and(|step| step.via_raw_name.0 == leaf)
        })
        .map(|container| container.origin.clone())
        .next()
        .unwrap_or_else(|| {
            panic!(
                "the fixture names exactly one container with leaf `{}`",
                String::from_utf8_lossy(leaf)
            )
        });
    LoadRoot::Container {
        origin,
        prefix: ArchiveNameBytes(Vec::new()),
    }
}

/// The class-source of one name in one snapshot, under the whole task budget.
fn class_source_of(
    snapshot: &ArtifactSnapshot,
    name: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceReport {
    performed(
        Engine::new()
            .class_source(
                slice::from_ref(snapshot),
                &request(
                    snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal(name),
                    },
                    policy,
                ),
                &mut budget(),
            )
            .expect("a legal class-source request is answered"),
    )
}

fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    let item = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in {report:?}"));
    &item.text
}

/// Every marker the text carries, in the order they appear, without the leading indentation.
fn markers(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("// jarde:"))
        .collect()
}

/// The one-line marker a member's own text carries, when it carries exactly one.
fn only_marker(text: &str) -> String {
    let found = markers(text);
    assert_eq!(found.len(), 1, "expected one marker in:\n{text}");
    found[0].to_owned()
}

/// Whether every line of the text starts at a multiple of four columns.
fn indentation_is_four_spaces(text: &str) -> bool {
    text.lines().all(|line| {
        let leading = line.len() - line.trim_start_matches(' ').len();
        leading % 4 == 0
    })
}

/// The brace balance of the text, ignoring comment lines (a fallback's reason may quote anything).
fn braces_balance(text: &str) -> i64 {
    let mut balance = 0_i64;
    for line in text.lines() {
        let line = line.trim_start();
        if line.starts_with("//") {
            continue;
        }
        for character in line.chars() {
            match character {
                '{' => balance += 1,
                '}' => balance -= 1,
                _ => {}
            }
        }
        assert!(
            balance >= 0,
            "a closing brace with nothing open before it in:\n{text}"
        );
    }
    balance
}

// ---------------------------------------------------------------------------------------------
// One class, one read, one run per member
// ---------------------------------------------------------------------------------------------

/// A real compiled class is presented whole: the declaration, the fields and the members in the
/// class file's own order, each with the report of its own run.
#[test]
fn one_real_class_is_presented_with_its_declaration_and_its_bodies() {
    let snapshot = open(HISTORICAL.to_vec());
    let mut budget = budget();
    let report = performed(
        Engine::new()
            .class_source(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::dotted("HistoricalControlFlow"),
                    },
                    EnvironmentPolicy::SingleClass,
                ),
                &mut budget,
            )
            .expect("the fixture is a legal request"),
    );

    // The declaration is the read's own item, and the text opens with the line spelled from it.
    let declaration = report.declaration.as_ref().expect("the class is declared");
    assert_eq!(declaration.name, "HistoricalControlFlow");
    assert_eq!(
        declaration.item.definition, report.class,
        "the item is the definition the report names"
    );
    assert_eq!(
        declaration.declaration,
        "public class HistoricalControlFlow extends java.lang.Object"
    );
    assert!(
        report.text.starts_with(
            "// jarde: presentation of `HistoricalControlFlow` from the class file's own \
             declaration and one recovery run per member.\n"
        ),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("public class HistoricalControlFlow extends java.lang.Object {\n"),
        "{}",
        report.text
    );

    // Every member of the class's own table is published, in table order, with its own run.
    let names: Vec<String> = report
        .methods
        .iter()
        .map(|method| method.item.name.escaped())
        .collect();
    assert_eq!(names, ["<init>", "add", "finallyPath"]);
    for method in &report.methods {
        let ClassSourceOutcome::Recovered { report, analysis } = &method.outcome else {
            panic!("every member of this class declares a body: {method:?}");
        };
        // The two halves are of one run, and each is that run's own plane: the analysis finished here
        // (the sample's bodies are all analyzed), and what the recovery layer delivered is stated by
        // the content plane — `contains_statements` for two members and `explanation_only` for the
        // third (see `an_explanation_only_member_is_marked_and_its_text_is_kept`).
        assert!(matches!(
            analysis.execution,
            ExecutionReport::Complete { .. }
        ));
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert!(report.produced());
    }
    // The one body's statement is in the text, under the declaration it belongs to.
    assert!(
        report
            .text
            .contains("    public int add(int arg1, int arg2) {\n"),
        "{}",
        report.text
    );
    assert!(report.text.contains("        return arg1 + arg2;\n"));
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
    assert!(indentation_is_four_spaces(&report.text), "{}", report.text);
    assert!(report.text.ends_with("}\n"));

    // One class header — the binding read, which is also the read the one preparation is built
    // over (D2 3.2: the selected definition is materialized once, never re-read for the
    // preparation) — and one body attempt per member: the member runs decode against that
    // preparation and charge no class read of their own (see
    // `one_preparation_serves_every_member_body`).
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert_eq!(report.usage.method_bodies, 3, "{:?}", report.usage);
    assert_eq!(
        report.execution,
        ExecutionReport::Complete {
            usage: report.usage.clone()
        }
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    // The field and the candidate search are the class view's own planes, beside this one.
    assert!(report.fields.is_empty());
    assert!(
        report
            .coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label == "class_source_bodies" && range.end == 3)
    );
    // The stages every member ran under are published, so one member's run can be reproduced.
    assert_eq!(report.stages, AnalysisStage::ALL.to_vec());
}

/// The same request twice — two fresh snapshots, two fresh budgets — is the same text, byte for
/// byte, and the same per-member markers.
#[test]
fn the_same_request_twice_is_the_same_text() {
    let first = class_source_of(
        &open(HISTORICAL.to_vec()),
        "HistoricalControlFlow",
        EnvironmentPolicy::SingleClass,
    );
    let second = class_source_of(
        &open(HISTORICAL.to_vec()),
        "HistoricalControlFlow",
        EnvironmentPolicy::SingleClass,
    );
    assert_eq!(first.text, second.text);
    assert_eq!(first.coverage, second.coverage);
    assert_eq!(first.fields, second.fields);
    // The reports themselves are not compared whole: a `UsageSnapshot` carries the elapsed clock of
    // the run that produced it. What must not move is every value this presentation *wrote*.
    let written =
        |report: &ClassSourceReport| -> Vec<(String, Option<String>, Vec<String>, String)> {
            report
                .methods
                .iter()
                .map(|method| {
                    (
                        method.declaration.clone().unwrap_or_default(),
                        method.no_body_kind.map(|kind| format!("{kind:?}")),
                        method.markers.clone(),
                        method.text.clone(),
                    )
                })
                .collect()
        };
    assert_eq!(written(&first), written(&second));
}

/// A member that declares no `Code` is a declaration and never a body: no run is charged for it, and
/// the text spells the member Java spells it — with the marker that says why.
#[test]
fn a_member_without_a_body_is_a_declaration_and_never_an_empty_body() {
    let snapshot = open(probe_class());
    let report = class_source_of(&snapshot, "p/Probe", EnvironmentPolicy::SingleClass);

    // The declaration of the whole class, with the interfaces the class file declares.
    let declaration = report.declaration.as_ref().expect("the class is declared");
    assert_eq!(
        declaration.declaration,
        "public class p.Probe extends java.lang.Object implements p.Marker, java.io.Serializable"
    );
    assert_eq!(declaration.name, "p.Probe");

    // The field of the same read, spelled from its descriptor.
    assert_eq!(report.fields.len(), 1);
    let field = &report.fields[0];
    assert_eq!(
        field.declaration.as_deref(),
        Some("public static final int value")
    );
    assert!(field.markers.is_empty());
    assert!(report.text.contains("    public static final int value;\n"));

    // `abstract` and `native` are declarations Java writes without a body, and the marker above each
    // one states that the class file says so.
    let abstract_one = text_of(&report, "abstractOne");
    assert_eq!(
        abstract_one,
        "    // jarde: no body: the member `abstractOne()V` is declared abstract and its \
         declaration carries no Code attribute\n    public abstract void abstractOne();\n"
    );
    let native_one = text_of(&report, "nativeOne");
    assert_eq!(
        native_one,
        "    // jarde: no body: the member `nativeOne()V` is declared native and its declaration \
         carries no Code attribute\n    public native void nativeOne();\n"
    );
    // A member that declares no `Code` and neither flag is a contradiction in the class file, and
    // it is stated as one: the declaration gets a block whose whole content is the marker, so no
    // empty body is ever written for it.
    let contradictory = text_of(&report, "contradictory");
    assert_eq!(
        contradictory,
        "    public void contradictory() {\n        // jarde: no body: the member \
         `contradictory()V` declares no Code attribute and is neither abstract nor native\n    }\n"
    );
    assert!(contradictory.contains("// jarde:"), "not silently dropped");

    // The three members with no body charge nothing: the three bodies are the whole of this
    // request's body work, and the one class read — the binding, over which the one preparation is
    // built — is beside them whatever the member count.
    assert_eq!(report.usage.method_bodies, 3, "{:?}", report.usage);
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    let no_body = report
        .methods
        .iter()
        .filter(|method| method.outcome == ClassSourceOutcome::NoBody)
        .count();
    assert_eq!(no_body, 3);
    assert_eq!(
        report
            .methods
            .iter()
            .filter(|method| method.no_body_kind == Some(NoBodyKind::Abstract))
            .count(),
        1
    );
    assert_eq!(
        report
            .methods
            .iter()
            .filter(|method| method.no_body_kind == Some(NoBodyKind::Native))
            .count(),
        1
    );
}

/// A member whose run stopped is marked in the text, its declaration is still written, the members
/// beside it are still presented, and the class report is not `Complete`.
#[test]
fn a_stopped_member_is_marked_and_does_not_stop_the_class() {
    let snapshot = open(probe_class());
    let report = class_source_of(&snapshot, "p/Probe", EnvironmentPolicy::SingleClass);

    // The member whose decode stopped: its declaration is there, its block holds the marker and
    // nothing else, and the artifact that was produced is the empty one the stop contract states.
    let broken = text_of(&report, "broken");
    let marker = only_marker(broken);
    assert!(
        marker.starts_with("// jarde: not recovered: the recovery run for `broken()V` stopped ("),
        "{marker}"
    );
    assert!(
        broken.starts_with("    public static void broken() {\n"),
        "{broken}"
    );
    assert!(broken.ends_with("    }\n"), "{broken}");
    let ClassSourceOutcome::Recovered {
        report: run,
        analysis,
    } = &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"broken")
        .expect("the member is published")
        .outcome
    else {
        panic!("the member's run was performed and refused its artifact");
    };
    assert_eq!(run.content, RecoveryContent::NotProduced);
    assert_eq!(run.text, "");
    assert!(run.source_map.is_empty(), "a stop carries no map");
    assert!(!matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));

    // The member beside it is presented in full, and the class report states the stop rather than a
    // complete presentation.
    assert!(
        report.text.contains(
            "        // recovered from bytecode; presentation is not claimed to compile\n"
        ),
        "the recoverable member's envelope is in the text: {}",
        report.text
    );
    assert!(
        !matches!(report.execution, ExecutionReport::Complete { .. }),
        "{:?}",
        report.execution
    );
    assert_eq!(report.usage.method_bodies, 3, "{:?}", report.usage);
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
    assert!(indentation_is_four_spaces(&report.text), "{}", report.text);
}

/// A class whose member table stopped is presented as what it is: the declaration the read really
/// established, the members it reached, and the comment that says the rest were never read — no
/// member is invented for the bytes that are missing.
#[test]
fn a_member_table_that_stops_is_stated_and_not_padded() {
    let snapshot = open(truncated_member_table_class());
    let report = class_source_of(&snapshot, "p/Truncated", EnvironmentPolicy::SingleClass);
    assert!(report.declaration.is_some(), "the class itself was read");
    assert!(report.fields.is_empty());
    assert!(report.methods.is_empty());
    assert!(
        report.text.contains("member table stopped at fields[0]"),
        "{}",
        report.text
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error),
        "{:?}",
        report.diagnostics
    );
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == "class_fields"),
        "the range that was never read is marked: {:?}",
        report.coverage
    );
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
    assert!(indentation_is_four_spaces(&report.text), "{}", report.text);
}

/// A member table that stops in front of a readable body keeps both statements apart: the prefix
/// member is presented as the member it is, with the stop's own reason where its body would be, and
/// the record behind the stop is **not** published as a member of this class.
#[test]
fn a_method_table_that_stops_keeps_its_prefix_and_claims_no_more() {
    let snapshot = open(stopped_method_table_class());
    let report = class_source_of(&snapshot, "p/Stopped", EnvironmentPolicy::SingleClass);

    // The class is read and presented, and the one member the read reached is published with its own
    // declaration and the marker that says why no body of it was decoded.
    assert!(report.declaration.is_some(), "the class itself was read");
    assert_eq!(report.methods.len(), 1, "{:?}", report.methods);
    let prefix_member = &report.methods[0];
    assert_eq!(prefix_member.item.name.raw().0, b"good");
    assert_eq!(
        prefix_member.declaration.as_deref(),
        Some("public static void good()")
    );
    assert!(
        prefix_member
            .markers
            .iter()
            .any(|marker| marker.contains("classfile_decode")),
        "the member states the stop rather than a body: {:?}",
        prefix_member.markers
    );
    // The record behind the stop is not a member of this presentation: no member conclusion about it
    // is published, in the report or in the text.
    assert!(!report.text.contains("second"), "{}", report.text);
    assert!(
        report.text.contains("member table stopped at methods[1]"),
        "{}",
        report.text
    );
    // No body was decoded from a table that did not read to its end, and the request is not complete.
    assert_eq!(report.usage.method_bodies, 0, "{:?}", report.usage);
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == "class_methods"),
        "the range the walk never read is marked: {:?}",
        report.coverage
    );
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
    assert!(indentation_is_four_spaces(&report.text), "{}", report.text);
}

/// An artifact that holds no statement is marked, and the artifact itself — the reasons and the
/// bytecode it quotes — stays in the text instead of being dropped.
#[test]
fn an_explanation_only_member_is_marked_and_its_text_is_kept() {
    let snapshot = open(REFUSED_CAST.to_vec());
    let report = class_source_of(&snapshot, "RefusedCast", EnvironmentPolicy::SingleClass);

    let explanation_only: Vec<&ClassSourceMethod> = report
        .methods
        .iter()
        .filter(|method| {
            matches!(
                &method.outcome,
                ClassSourceOutcome::Recovered { report, .. }
                    if report.content == RecoveryContent::ExplanationOnly
            )
        })
        .collect();
    assert_eq!(
        explanation_only.len(),
        3,
        "the sample's three refused casts"
    );
    for method in explanation_only {
        let ClassSourceOutcome::Recovered { report: run, .. } = &method.outcome else {
            unreachable!("filtered above")
        };
        assert!(run.produced());
        assert!(
            only_marker(&method.text).contains("produced no statement"),
            "{}",
            method.text
        );
        assert!(
            method.text.contains("// @bytecode "),
            "the refusals the artifact quotes are kept:\n{}",
            method.text
        );
    }
    // The recovery runs of those members completed: what they could not do is produce a statement,
    // which the content plane and the marker state rather than the execution plane.
    assert_eq!(
        report.execution,
        ExecutionReport::Complete {
            usage: report.usage.clone()
        }
    );
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
}

/// Two definitions of one name are two candidates and no presentation; one of them, named by its
/// own identity, is presented.
#[test]
fn a_name_two_definitions_answer_to_presents_nothing() {
    let bytes = probe_class();
    let snapshot = open(zip_of(&[
        (b"p/Probe.class", &bytes),
        (b"WEB-INF/classes/p/Probe.class", &bytes),
        (b"META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
    ]));
    let engine = Engine::new();
    let ambiguous = engine
        .class_source(
            slice::from_ref(&snapshot),
            &request(
                &snapshot,
                ClassRef::Name {
                    class: ClassNameQuery::internal("p/Probe"),
                },
                EnvironmentPolicy::PlainJar,
            ),
            &mut budget(),
        )
        .expect("a legal request is answered");
    let OperationOutcome::Ambiguous(candidates) = ambiguous else {
        panic!("two origins of one name are two candidates: {ambiguous:?}");
    };
    assert_eq!(candidates.candidates.len(), 2);
    assert_eq!(
        candidates
            .candidates
            .iter()
            .filter(|candidate| matches!(candidate, ClassContentItem::ClassDeclaration(_)))
            .count(),
        2
    );
    assert!(matches!(
        candidates.execution,
        ExecutionReport::Complete { .. }
    ));

    // The first candidate's own identity is directly usable, and it reads exactly that definition.
    let ClassContentItem::ClassDeclaration(chosen) = candidates.candidates[0].clone() else {
        unreachable!("a class search publishes class declarations")
    };
    let report = performed(
        engine
            .class_source(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Definition {
                        definition: chosen.definition.clone(),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &mut budget(),
            )
            .expect("a legal request is answered"),
    );
    assert_eq!(report.class, chosen.definition);
    assert!(
        report
            .text
            .contains("public class p.Probe extends java.lang.Object")
    );
}

/// An identity of another artifact is an input error, never replaced by a same-named definition of
/// the snapshot at hand.
#[test]
fn an_identity_of_another_snapshot_is_an_input_error() {
    let other = open(probe_class());
    let target = open(HISTORICAL.to_vec());
    let definition = class_source_of(&other, "p/Probe", EnvironmentPolicy::SingleClass).class;
    let error = Engine::new()
        .class_source(
            slice::from_ref(&target),
            &request(
                &target,
                ClassRef::Definition { definition },
                EnvironmentPolicy::SingleClass,
            ),
            &mut budget(),
        )
        .expect_err("an identity of another snapshot is refused");
    assert_eq!(error_code(&error), "operation_target_snapshot_mismatch");
}

/// A request whose item budget cannot pay for its own declaration publishes no declaration, no
/// member and no text, and states the stop where every other report states one.
///
/// The item budget a presentation needs before it may publish anything is the reader's own
/// accounting, so this case walks the boundary instead of naming the count: whatever that count
/// becomes, "no declaration" and "no text" stay the same state, and some budget refuses the
/// declaration itself.
#[test]
fn a_request_that_cannot_pay_for_its_declaration_publishes_no_text() {
    let snapshot = open(probe_class());
    let definition = class_source_of(&snapshot, "p/Probe", EnvironmentPolicy::SingleClass).class;
    let mut refused = None;
    for limit in 1..=16 {
        let mut budget =
            task_budget(&[BudgetOverride::new("result_items", limit).expect("a legal override")])
                .expect("the override is legal");
        let outcome = Engine::new().class_source(
            slice::from_ref(&snapshot),
            &request(
                &snapshot,
                ClassRef::Definition {
                    definition: definition.clone(),
                },
                EnvironmentPolicy::SingleClass,
            ),
            &mut budget,
        );
        let outcome = match outcome {
            // The read itself was refused before it materialized anything: nothing was established
            // to publish, which is the request-level refusal every read in this engine states.
            Err(Error::BudgetExceeded { .. }) => continue,
            Err(error) => panic!("at {limit} item(s): unexpected {error:?}"),
            Ok(outcome) => outcome,
        };
        match outcome {
            // The selection itself did not finish: nothing was bound, so nothing is presented.
            OperationOutcome::Incomplete(candidates) => {
                assert!(!matches!(
                    candidates.execution,
                    ExecutionReport::Complete { .. }
                ));
            }
            OperationOutcome::Performed(report) => {
                assert_eq!(
                    report.declaration.is_none(),
                    report.text.is_empty(),
                    "at {limit} item(s): the text is empty exactly when no declaration was \
                     published"
                );
                match &report.declaration {
                    // A declaration this request paid for is in the text it opens.
                    Some(declaration) => assert!(
                        report.text.contains(&declaration.declaration),
                        "at {limit} item(s): {}",
                        report.text
                    ),
                    // Nothing of the class is presented, and the stop is stated where every other
                    // report states one.
                    None => {
                        assert!(report.methods.is_empty());
                        assert!(report.fields.is_empty());
                        assert!(!matches!(
                            report.execution,
                            ExecutionReport::Complete { .. }
                        ));
                        assert!(
                            report.diagnostics.iter().any(|diagnostic| diagnostic.code
                                == "budget_exceeded_result_items"),
                            "{:?}",
                            report.diagnostics
                        );
                        refused = Some(report);
                    }
                }
            }
            OperationOutcome::Ambiguous(_) => panic!("one definition is never ambiguous"),
        }
    }
    assert!(
        refused.is_some(),
        "some item budget refuses the class item charge itself"
    );
}

/// A stop that is the *request's* ends it: the members after it are not begun, the text states how
/// many the class declares beside how many were presented, and the coverage marks the range that
/// was skipped.
#[test]
fn a_request_that_stops_mid_class_states_the_shortfall() {
    let snapshot = open(probe_class());
    // One body attempt: the first member's run is funded, and the second member's charge is refused.
    let mut budget =
        task_budget(&[BudgetOverride::new("method_bodies", 1).expect("a legal override")])
            .expect("the override is legal");
    let report = performed(
        Engine::new()
            .class_source(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("p/Probe"),
                    },
                    EnvironmentPolicy::SingleClass,
                ),
                &mut budget,
            )
            .expect("a legal request is answered"),
    );
    assert_eq!(report.usage.method_bodies, 1, "{:?}", report.usage);
    let ExecutionReport::Partial { reason, .. } = &report.execution else {
        panic!("the stop is the request's: {:?}", report.execution)
    };
    assert_eq!(
        reason,
        &TerminationReason::BudgetExceeded {
            dimension: BudgetDimension::MethodBodies
        }
    );
    // The member whose own run was refused keeps its own result — the stop is that run's, stated by
    // the two planes of it — and the members behind it were never begun.
    assert_eq!(report.methods.len(), 2, "{:?}", report.methods);
    let second = &report.methods[1];
    let ClassSourceOutcome::Recovered {
        report: run,
        analysis,
    } = &second.outcome
    else {
        panic!("the member's run was performed and stopped inside: {second:?}")
    };
    assert_eq!(run.content, RecoveryContent::NotProduced);
    let ExecutionReport::Partial { reason, .. } = &analysis.execution else {
        panic!(
            "the analysis of that member stopped: {:?}",
            analysis.execution
        )
    };
    assert_eq!(
        reason,
        &TerminationReason::BudgetExceeded {
            dimension: BudgetDimension::MethodBodies
        },
        "the analysis of that member is what the refused charge stopped"
    );
    assert_eq!(second.markers.len(), 1);
    assert!(
        second.markers[0].contains("budget_exceeded_method_bodies"),
        "{}",
        second.markers[0]
    );
    // The text states the shortfall by name instead of presenting a shorter class as a whole one.
    assert!(
        report
            .text
            .contains("// jarde: the class file declares 6 method record(s)"),
        "{}",
        report.text
    );
    assert!(report.text.contains("2 were presented"), "{}", report.text);
    assert!(
        report.text.contains("budget_exceeded_method_bodies"),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("nativeOne"),
        "a member that was never begun is not presented: {}",
        report.text
    );
    let skipped: Vec<(u64, u64)> = report
        .coverage
        .artifact_structural
        .skipped
        .iter()
        .filter(|range| range.label == "class_source_bodies")
        .map(|range| (range.start, range.end))
        .collect();
    assert_eq!(
        skipped,
        [(2, 3)],
        "the third body is the member this request never began: {:?}",
        report.coverage
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
}

/// A member whose raw descriptor is not a method descriptor is stated as such: no declaration, no
/// run, and the marker that says both — while the members beside it are presented as usual.
#[test]
fn a_member_whose_descriptor_cannot_be_read_is_stated_and_not_run() {
    let bytes = class_file(
        b"p/Odd",
        b"java/lang/Object",
        &[],
        CLASS_FLAGS,
        &[],
        &[
            MethodSpec {
                flags: PUBLIC_METHOD,
                name: b"fine",
                descriptor: b"()V",
                code: Some(PLAIN_BODY),
            },
            MethodSpec {
                flags: PUBLIC_METHOD,
                name: b"odd",
                descriptor: b"not-a-descriptor",
                code: Some(PLAIN_BODY),
            },
        ],
    );
    let snapshot = open(bytes);
    let report = class_source_of(&snapshot, "p/Odd", EnvironmentPolicy::SingleClass);
    let odd = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"odd")
        .expect("the member is published");
    assert_eq!(odd.outcome, ClassSourceOutcome::Unspelled);
    assert!(odd.declaration.is_none());
    assert_eq!(
        odd.text,
        "    // jarde: not spelled: the descriptor `not-a-descriptor` of the member \
                          `odd` is not a method descriptor, so this presentation writes no \
                          declaration for it and performs no run for it\n"
    );
    // Only the member this presentation can spell was run: the odd one is not work that was skipped.
    assert_eq!(report.usage.method_bodies, 1, "{:?}", report.usage);
    assert!(
        report.text.contains("public void fine()"),
        "{}",
        report.text
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label == "class_source_bodies" && range.end == 1)
    );
    assert!(
        report.coverage.artifact_structural.skipped.is_empty(),
        "{:?}",
        report.coverage
    );
}

/// The report is the library's own value: it serializes to the documented shape, and the text it
/// carries is the field a consumer reads.
#[test]
fn the_report_serializes_with_the_text_it_publishes() {
    let snapshot = open(HISTORICAL.to_vec());
    let report = class_source_of(
        &snapshot,
        "HistoricalControlFlow",
        EnvironmentPolicy::SingleClass,
    );
    let document = serde_json::to_value(&report).expect("the report is a document");
    assert_eq!(
        document["text"].as_str().expect("the text is a string"),
        report.text
    );
    assert_eq!(
        document["class"]["location"]["kind"].as_str(),
        Some("standalone_root")
    );
    assert_eq!(
        document["methods"][1]["item"]["name"]["escaped"].as_str(),
        Some("add")
    );
    assert_eq!(
        document["methods"][1]["outcome"]["kind"].as_str(),
        Some("recovered")
    );
    assert_eq!(
        document["methods"][1]["outcome"]["report"]["text"].as_str(),
        Some(
            report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"add")
                .expect("the member is published")
                .outcome
                .clone()
                .into_recovery_text()
                .as_str()
        )
    );
}

/// The recovery artifact of one member, for the one case above that compares the two texts.
trait IntoRecoveryText {
    fn into_recovery_text(self) -> String;
}

impl IntoRecoveryText for ClassSourceOutcome {
    fn into_recovery_text(self) -> String {
        match self {
            ClassSourceOutcome::Recovered { report, .. } => report.text,
            ClassSourceOutcome::NoBody
            | ClassSourceOutcome::Unspelled
            | ClassSourceOutcome::Refused { .. } => String::new(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The input shapes a class is prepared from
// ---------------------------------------------------------------------------------------------

/// A class in an archive is prepared from the very entry the binding bound, in both container
/// shapes: one under a declared entry prefix (`WEB-INF/classes/`, the WAR class layer) and one
/// inside a nested library the artifact tree reaches.
///
/// Both are presented from the physical definition the name search confirmed, under the environment
/// that declares the position the class really lives in — a prefix root for the class layer, the
/// nested container's own origin for the library — so the preparation reads the entry the binding
/// read and the bodies recover against it. The class bytes are read twice, exactly as for a
/// standalone class: the member count does not enter the class-read shape.
#[test]
fn a_class_in_a_container_is_prepared_from_the_entry_it_lives_in() {
    let class = many_bodies_class(b"p/Container", 2);
    let nested_class = many_bodies_class(b"p/Nested", 2);
    let library = zip_of(&[(b"p/Nested.class", &nested_class)]);
    let snapshot = open(zip_of(&[
        (b"WEB-INF/classes/p/Container.class", &class),
        (b"WEB-INF/lib/L.jar", &library),
        (b"META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
    ]));
    let engine = Engine::new();

    // The WAR class layer: a container root with the entry prefix the class layer really has.
    let class_layer = performed(
        engine
            .class_source(
                slice::from_ref(&snapshot),
                &scoped_request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("p/Container"),
                    },
                    EnvironmentPolicy::ExplicitClasspath {
                        roots: vec![LoadRoot::Container {
                            origin: root_origin(&snapshot),
                            prefix: ArchiveNameBytes(b"WEB-INF/classes/".to_vec()),
                        }],
                    },
                    PhysicalScope::SnapshotAll,
                ),
                &mut budget(),
            )
            .expect("a legal request is answered"),
    );
    assert_eq!(
        class_layer
            .class
            .location
            .entry()
            .expect("an entry")
            .raw_name
            .0,
        b"WEB-INF/classes/p/Container.class"
    );
    // One class read for the whole request (D2 3.2): the search read the definition it elected,
    // and the one preparation is built over *that* read instead of reading the same entry again.
    assert_eq!(
        class_layer.usage.class_headers, 1,
        "{:?}",
        class_layer.usage
    );
    assert_eq!(
        class_layer.usage.method_bodies, 2,
        "{:?}",
        class_layer.usage
    );
    assert_eq!(
        class_layer.usage.class_bytes,
        2 * class_layer.class.class_bytes.length
    );
    assert!(
        class_layer.text.contains("        return;\n"),
        "{}",
        class_layer.text
    );

    // The nested library: the scope is the whole artifact tree, so the search reaches the entry
    // inside `WEB-INF/lib/L.jar`, and the declared position is that nested container's own origin.
    let nested = performed(
        engine
            .class_source(
                slice::from_ref(&snapshot),
                &scoped_request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("p/Nested"),
                    },
                    EnvironmentPolicy::ExplicitClasspath {
                        roots: vec![nested_root(&snapshot, b"WEB-INF/lib/L.jar")],
                    },
                    PhysicalScope::ArtifactTree {
                        root_container: ContainerId("root".into()),
                    },
                ),
                &mut budget(),
            )
            .expect("a legal request is answered"),
    );
    let entry = nested.class.location.entry().expect("an entry");
    assert_eq!(entry.raw_name.0, b"p/Nested.class");
    assert_eq!(
        entry.origin.steps.len(),
        1,
        "the class is one container deep"
    );
    assert!(
        nested
            .coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label == "navigation_candidates"),
        "the search walked the tree scope: {:?}",
        nested.coverage
    );
    assert_eq!(nested.usage.class_headers, 1, "{:?}", nested.usage);
    assert_eq!(nested.usage.method_bodies, 2, "{:?}", nested.usage);
    assert_eq!(
        nested.usage.class_bytes,
        2 * nested.class.class_bytes.length
    );
    assert!(
        nested
            .text
            .contains("public class p.Nested extends java.lang.Object {\n")
    );
    assert!(nested.text.contains("        return;\n"), "{}", nested.text);
    assert_eq!(
        nested.execution,
        ExecutionReport::Complete {
            usage: nested.usage.clone()
        }
    );
}

/// A class the reader's strict structure read refuses — a class whose bytes the tolerant class read
/// accepts but whose whole structure does not decode — keeps its presentation, and each member that
/// declares a body states the preparation's own failure.
///
/// This is what the one preparation owes a class like this: the declaration, the fields and every
/// member declaration stay published (the binding read them), and no body is invented or attempted
/// from a class that could not be prepared. The class reads are the same two as for a healthy class,
/// and one diagnostic per member that declares a body names the reader's own code.
#[test]
fn a_class_that_cannot_be_prepared_keeps_its_presentation() {
    let mut bytes = many_bodies_class(b"p/Trailing", 2);
    bytes.push(0);
    let report = class_source_of(&open(bytes), "p/Trailing", EnvironmentPolicy::SingleClass);

    assert!(report.declaration.is_some(), "the class is presented");
    assert_eq!(report.methods.len(), 3, "{:?}", report.methods);
    assert!(
        report
            .text
            .contains("public class p.Trailing extends java.lang.Object {\n")
    );
    assert!(report.text.contains("public abstract void declaredOnly();"));
    assert!(braces_balance(&report.text) == 0, "{}", report.text);

    let refused: Vec<&ClassSourceMethod> = report
        .methods
        .iter()
        .filter(|method| matches!(method.outcome, ClassSourceOutcome::Refused { .. }))
        .collect();
    assert_eq!(refused.len(), 2, "every body-bearing member states it");
    for method in refused {
        let ClassSourceOutcome::Refused { diagnostics, .. } = &method.outcome else {
            unreachable!("filtered on the refusal above")
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "classfile_trailing_bytes"),
            "the member's own refusal names the reader's code: {diagnostics:?}"
        );
        assert!(
            method
                .markers
                .iter()
                .any(|marker| marker.contains("classfile_trailing_bytes")),
            "{:?}",
            method.markers
        );
    }
    // No body was attempted, and the failure is the request's own plane rather than a success. The
    // class itself was read once — the binding read, over which the preparation that refused these
    // members was attempted (D2 3.2) — and not once per member that states the refusal.
    assert_eq!(report.usage.method_bodies, 0, "{:?}", report.usage);
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
}

// ---------------------------------------------------------------------------------------------
// The read shape: one preparation, one decode per body
// ---------------------------------------------------------------------------------------------

/// The read shape of one presentation of one class: **one** class read whatever the member count —
/// the binding read, which is also the read the one preparation is built over (D2 3.2) — and one
/// decode per member that declares a body.
///
/// This is the assertion task 7.3 asks for, and it is deliberately a *shape* assertion rather than a
/// count of one fixture: two classes whose member counts differ (6 and 9, each with one member that
/// declares no body) are presented under the whole task budget, and the class reads must be the same
/// one while the body attempts follow the bodies. A return to a per-member run reads the class once
/// per member and fails here: `class_headers` would be 1 + 5 and 1 + 8, and `class_bytes` would grow
/// by a class per member.
#[test]
fn one_preparation_serves_every_member_body() {
    let six = class_source_of(
        &open(many_bodies_class(b"p/Six", 5)),
        "p/Six",
        EnvironmentPolicy::SingleClass,
    );
    let nine = class_source_of(
        &open(many_bodies_class(b"p/Nine", 8)),
        "p/Nine",
        EnvironmentPolicy::SingleClass,
    );

    for (report, bodies) in [(&six, 5_u64), (&nine, 8_u64)] {
        // One class read for the binding, which the one preparation is built over (D2 3.2):
        // `class_headers` is the same one for both classes, and the class bytes are parsed exactly
        // twice — the binding's own member walk and the preparation, each counted once for the
        // class's own length, never once per member.
        assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
        assert_eq!(report.usage.method_bodies, bodies, "{:?}", report.usage);
        assert_eq!(
            report.usage.class_bytes,
            2 * report.class.class_bytes.length,
            "the class bytes are parsed by the binding's member walk and by the one preparation \
             built over that same read — two parses of one read, not two reads: {:?}",
            report.usage
        );
        // Every member that declares a body really ran, and the member that declares none is the
        // declaration this presentation writes for it.
        assert_eq!(
            report
                .methods
                .iter()
                .filter(|method| matches!(method.outcome, ClassSourceOutcome::Recovered { .. }))
                .count() as u64,
            bodies
        );
        assert_eq!(
            report
                .methods
                .iter()
                .filter(|method| method.outcome == ClassSourceOutcome::NoBody)
                .count(),
            1
        );
        // The class is presented whole: no member is dropped for having been read through the
        // preparation, and every body's text is in the text.
        assert_eq!(report.methods.len() as u64, bodies + 1);
        assert!(report.text.contains("// jarde: presentation of `p/"));
        assert!(report.text.contains("    public static void body0() {\n"));
        assert!(
            report
                .text
                .contains(&format!("    public static void body{}() {{\n", bodies - 1)),
            "{}",
            report.text
        );
        assert!(braces_balance(&report.text) == 0, "{}", report.text);
    }
    // The two classes differ in member count and not in what one request reads of a class.
    assert_eq!(six.usage.class_headers, nine.usage.class_headers);
    assert!(
        nine.usage.method_bodies > six.usage.method_bodies,
        "the bodies really are more: {:?} vs {:?}",
        six.usage,
        nine.usage
    );

    // And no member's own run read the class: the run after the first adds exactly one body attempt
    // and no class header at all, member after member.
    for (report, bodies) in [(&six, 5_u64), (&nine, 8_u64)] {
        let runs: Vec<&UsageSnapshot> = report
            .methods
            .iter()
            .filter_map(|method| match &method.outcome {
                ClassSourceOutcome::Recovered { analysis, .. } => {
                    Some(usage_of(&analysis.execution))
                }
                ClassSourceOutcome::NoBody
                | ClassSourceOutcome::Unspelled
                | ClassSourceOutcome::Refused { .. } => None,
            })
            .collect();
        assert_eq!(runs.len() as u64, bodies, "every body-bearing member ran");
        for pair in runs.windows(2) {
            assert_eq!(
                pair[1].class_headers, pair[0].class_headers,
                "a member's run charged a class read of its own: {:?}",
                pair[1]
            );
            assert_eq!(
                pair[1].method_bodies,
                pair[0].method_bodies + 1,
                "a member's run is one body attempt: {:?}",
                pair[1]
            );
        }
    }
}
