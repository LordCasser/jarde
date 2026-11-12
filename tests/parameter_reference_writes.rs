//! Fixed parameter declarations must accept every physical reference write.
//! These generated classes use public Engine entry points, not compiler byte fingerprints.
use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path::PathBuf, process::Command, slice};

#[derive(Clone, Copy)]
enum ValueSource {
    Unwritten,
    ClassOf,
    Literal,
    Null,
    SecondParameter,
}

struct Case {
    class: String,
    method: &'static str,
    descriptor: String,
    source: ValueSource,
    wide: bool,
    arguments: &'static [&'static str],
}

fn case(
    label: &str,
    target: &str,
    source: ValueSource,
    second: &str,
    wide: bool,
    arguments: &'static [&'static str],
) -> Case {
    let class = format!("{label}{}", if wide { "Shifted" } else { "Original" });
    let target = target.replace("$self", &class);
    Case {
        class,
        method: if wide { "renamed" } else { "apply" },
        descriptor: format!("({target}{second})Ljava/lang/Object;"),
        source,
        wide,
        arguments,
    }
}

fn supported(wide: bool) -> Vec<Case> {
    vec![
        case(
            "ClassToObject",
            "Ljava/lang/Object;",
            ValueSource::ClassOf,
            "",
            wide,
            &["new Object()", "\"text\"", "null"],
        ),
        case(
            "ClassToClass",
            "Ljava/lang/Class;",
            ValueSource::ClassOf,
            "",
            wide,
            &["String.class", "Integer.class", "null"],
        ),
        case(
            "ExactString",
            "Ljava/lang/String;",
            ValueSource::Literal,
            "",
            wide,
            &["\"before\"", "null"],
        ),
        case(
            "IntrinsicParent",
            "Ljava/lang/CharSequence;",
            ValueSource::Literal,
            "",
            wide,
            &["\"before\"", "new StringBuilder(\"before\")", "null"],
        ),
        case(
            "NullStore",
            "Ljava/lang/String;",
            ValueSource::Null,
            "",
            wide,
            &["\"before\"", "null"],
        ),
        case(
            "CopyToObject",
            "Ljava/lang/Object;",
            ValueSource::SecondParameter,
            "Ljava/lang/String;",
            wide,
            &["new Object(), \"after\"", "null, null"],
        ),
        case(
            "ArrayCovariance",
            "[Ljava/lang/Object;",
            ValueSource::SecondParameter,
            "[Ljava/lang/String;",
            wide,
            &["new Object[0], new String[] {\"after\"}", "null, null"],
        ),
        case(
            "ReadOnly",
            "Ljava/lang/String;",
            ValueSource::Unwritten,
            "",
            wide,
            &["\"unchanged\"", "null"],
        ),
    ]
}

fn refused(wide: bool) -> Vec<Case> {
    vec![
        // Like the javac8 getClass counterexample: Class cannot replace this parameter's
        // own class. Missing platform headers are not an assignment proof.
        case(
            "ClassToOwnType",
            "L$self;",
            ValueSource::ClassOf,
            "",
            wide,
            &[],
        ),
        case(
            "ClassToString",
            "Ljava/lang/String;",
            ValueSource::ClassOf,
            "",
            wide,
            &[],
        ),
        // No getClass, allocation dance or stale-value snapshot is involved here.
        case("StringToArray", "[I", ValueSource::Literal, "", wide, &[]),
        // The method is legal JVM even when no supplied Header proves Child extends Parent.
        case(
            "MissingHierarchy",
            "LAbsentParent;",
            ValueSource::SecondParameter,
            "LAbsentChild;",
            wide,
            &[],
        ),
    ]
}

fn physical_parent_case(wide: bool) -> Case {
    case(
        "PhysicalParent",
        "LParameterParent;",
        ValueSource::SecondParameter,
        "LParameterChild;",
        wide,
        &[],
    )
}

fn local(code: &mut Vec<u8>, load: bool, slot: u8, wide: bool) {
    if wide {
        code.extend([0xc4, if load { 0x19 } else { 0x3a }, 0, slot]);
    } else {
        code.push(if load { 0x2a + slot } else { 0x4b + slot });
    }
}

fn code(case: &Case) -> Vec<u8> {
    let mut code = Vec::new();
    // Establish the fixed parameter type in a first, compatible physical write. The
    // later store must still be checked, and the selected-Header relation collector
    // sees the actual entry type rather than only the overwritten slot's frame type.
    if matches!(
        case.source,
        ValueSource::Literal | ValueSource::SecondParameter
    ) {
        local(&mut code, true, 0, case.wide);
        local(&mut code, false, 0, case.wide);
    }
    match case.source {
        ValueSource::Unwritten => {}
        ValueSource::ClassOf => {
            local(&mut code, true, 0, case.wide);
            code.extend([0xb6, 0, 11]);
        }
        ValueSource::Literal => code.extend([0x12, 13]),
        ValueSource::Null => code.push(0x01),
        ValueSource::SecondParameter => local(&mut code, true, 1, case.wide),
    }
    if !matches!(case.source, ValueSource::Unwritten) {
        local(&mut code, false, 0, case.wide);
    }
    local(&mut code, true, 0, case.wide);
    code.push(0xb0);
    code
}

fn u16b(out: &mut Vec<u8>, value: u16) {
    out.extend(value.to_be_bytes());
}
fn u32b(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_be_bytes());
}
fn utf8(out: &mut Vec<u8>, value: &str) {
    out.push(1);
    u16b(out, value.len() as u16);
    out.extend(value.as_bytes());
}
fn pair(out: &mut Vec<u8>, tag: u8, first: u16, second: u16) {
    out.push(tag);
    u16b(out, first);
    u16b(out, second);
}
fn method(out: &mut Vec<u8>, flags: u16, name: u16, descriptor: u16, code: &[u8]) {
    for n in [flags, name, descriptor, 1, 7] {
        u16b(out, n);
    }
    u32b(out, 12 + code.len() as u32);
    u16b(out, 1); // max_stack
    u16b(out, 2); // max_locals
    u32b(out, code.len() as u32);
    out.extend(code);
    u16b(out, 0); // exceptions
    u16b(out, 0); // Code attributes
}

fn class_bytes(case: &Case) -> Vec<u8> {
    let mut out = Vec::new();
    u32b(&mut out, 0xcafebabe);
    for n in [0, 52, 18] {
        u16b(&mut out, n);
    }
    utf8(&mut out, &case.class); // 1
    out.push(7);
    u16b(&mut out, 1); // 2
    utf8(&mut out, "java/lang/Object"); // 3
    out.push(7);
    u16b(&mut out, 3); // 4
    utf8(&mut out, case.method); // 5
    utf8(&mut out, &case.descriptor); // 6
    utf8(&mut out, "Code"); // 7
    utf8(&mut out, "getClass"); // 8
    utf8(&mut out, "()Ljava/lang/Class;"); // 9
    pair(&mut out, 12, 8, 9); // 10
    pair(&mut out, 10, 4, 10); // 11
    utf8(&mut out, "after"); // 12
    out.push(8);
    u16b(&mut out, 12); // 13
    utf8(&mut out, "<init>"); // 14
    utf8(&mut out, "()V"); // 15
    pair(&mut out, 12, 14, 15); // 16
    pair(&mut out, 10, 4, 16); // 17
    for n in [0x21, 2, 4, 0, 0, 2] {
        u16b(&mut out, n);
    }
    method(&mut out, 0x01, 14, 15, &[0x2a, 0xb7, 0, 17, 0xb1]);
    method(&mut out, 0x09, 5, 6, &code(case));
    u16b(&mut out, 0);
    out
}

fn header_class(name: &str, parent: &str) -> Vec<u8> {
    let mut out = Vec::new();
    u32b(&mut out, 0xcafebabe);
    for n in [0, 52, 10] {
        u16b(&mut out, n);
    }
    utf8(&mut out, name); // 1
    out.push(7);
    u16b(&mut out, 1); // 2
    utf8(&mut out, parent); // 3
    out.push(7);
    u16b(&mut out, 3); // 4
    utf8(&mut out, "<init>"); // 5
    utf8(&mut out, "()V"); // 6
    utf8(&mut out, "Code"); // 7
    pair(&mut out, 12, 5, 6); // 8
    pair(&mut out, 10, 4, 8); // 9
    for n in [0x21, 2, 4, 0, 0, 1] {
        u16b(&mut out, n);
    }
    method(&mut out, 0x01, 5, 6, &[0x2a, 0xb7, 0, 9, 0xb1]);
    u16b(&mut out, 0);
    out
}

fn physical_headers() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (
            "ParameterParent.class",
            header_class("ParameterParent", "java/lang/Object"),
        ),
        (
            "ParameterChild.class",
            header_class("ParameterChild", "ParameterParent"),
        ),
    ]
}

fn physical_parent_jar(case: &Case, include_headers: bool) -> Vec<u8> {
    let mut entries = vec![(format!("{}.class", case.class), class_bytes(case))];
    if include_headers {
        entries.extend(
            physical_headers()
                .into_iter()
                .map(|(name, bytes)| (name.to_owned(), bytes)),
        );
    }
    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in entries {
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(name.into_bytes()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(&bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
    }
    output.into_inner()
}

fn budget() -> Budget {
    task_budget(&[]).unwrap()
}
fn source(case: &Case) -> ClassSourceReport {
    source_from(case, class_bytes(case), EnvironmentPolicy::SingleClass)
}

fn source_from(case: &Case, bytes: Vec<u8>, policy: EnvironmentPolicy) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(&case.class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".into()),
        },
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut budget(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("{}: {other:?}", case.class),
    }
}

fn body<'a>(
    report: &'a ClassSourceReport,
    case: &Case,
) -> (&'a ClassSourceMethod, &'a RecoveryReport) {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == case.method.as_bytes())
        .unwrap();
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!("{:?}", method.outcome)
    };
    (method, body)
}

fn assert_supported(case: &Case, report: &ClassSourceReport) {
    let (method, body) = body(report, case);
    assert!(body.produced(), "{}: {body:?}", case.class);
    assert_eq!(body.representation, Representation::Java, "{}", body.text);
    assert_eq!(body.quality, Quality::Structured, "{}", body.text);
    assert_eq!(body.content, RecoveryContent::ContainsStatements);
    assert!(method.markers.is_empty(), "{:?}", method.markers);
    assert!(
        !report.text.contains("not recovered") && !report.text.contains("@bytecode"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("saved0"), "{}", report.text);
}

#[test]
fn fixed_reference_parameters_keep_exact_proved_parent_null_and_unwritten_controls() {
    for wide in [false, true] {
        for case in supported(wide) {
            assert_supported(&case, &source(&case));
        }
    }
}

#[test]
fn selected_physical_parent_edge_authorizes_the_same_store_missing_headers_refuse() {
    for wide in [false, true] {
        let case = physical_parent_case(wide);
        let full = source_from(
            &case,
            physical_parent_jar(&case, true),
            EnvironmentPolicy::PlainJar,
        );
        assert_supported(&case, &full);
        let (_, recovered) = body(&full, &case);
        let child = header_class("ParameterChild", "ParameterParent");
        let selected = recovered
            .artifact
            .binding()
            .unwrap()
            .dependencies()
            .iter()
            .find(|dependency| {
                dependency
                    .definition
                    .location
                    .entry()
                    .is_some_and(|entry| entry.raw_name.0 == b"ParameterChild.class")
            })
            .expect("the assignable edge comes from the selected physical Child Header");
        assert_eq!(
            selected.definition.class_bytes.digest.0,
            blake3::hash(&child).to_hex().to_string()
        );

        // Identical root bytes and request policy; only the two dependency Headers are removed.
        let missing = source_from(
            &case,
            physical_parent_jar(&case, false),
            EnvironmentPolicy::PlainJar,
        );
        assert_eq!(full.class.class_bytes, missing.class.class_bytes);
        let (_, refused) = body(&missing, &case);
        assert!(refused.produced(), "{refused:?}");
        assert_eq!(
            refused.representation,
            Representation::Mixed,
            "{}",
            refused.text
        );
        assert_eq!(refused.quality, Quality::Fallback, "{}", refused.text);
        assert_eq!(
            refused.content,
            RecoveryContent::ExplanationOnly,
            "{}",
            refused.text
        );
        assert!(
            refused.text.contains("reference parameter") && refused.text.contains("Missing"),
            "{}",
            refused.text
        );
        assert!(
            !refused.text.contains("saved0") && !refused.text.contains("arg0 ="),
            "{}",
            refused.text
        );
    }
}

#[test]
fn incompatible_or_missing_reference_writes_refuse_the_whole_member_with_physical_facts() {
    for wide in [false, true] {
        for case in refused(wide) {
            let bytes = class_bytes(&case);
            let report = source(&case);
            let (method, body) = body(&report, &case);
            assert!(body.produced(), "{body:?}");
            assert!(matches!(body.execution, ExecutionReport::Complete { .. }));
            assert_eq!(body.representation, Representation::Mixed, "{}", body.text);
            assert_eq!(body.quality, Quality::Fallback, "{}", body.text);
            assert_eq!(
                body.content,
                RecoveryContent::ExplanationOnly,
                "{}",
                body.text
            );
            assert!(
                report.text.contains("not recovered: the recovery run"),
                "{}",
                report.text
            );
            assert!(
                body.text.contains("reference parameter") && body.text.contains("fixed type"),
                "{}",
                body.text
            );
            assert!(
                !body.text.contains("saved0") && !body.text.contains("arg0 ="),
                "{}",
                body.text
            );
            assert_eq!(method.item.identity.owner, report.class);
            assert_eq!(method.item.descriptor.raw().0, case.descriptor.as_bytes());
            let physical = inspect_method_bytecode(
                &bytes,
                MethodSelector {
                    name: JvmBytes(case.method.as_bytes().to_vec()),
                    descriptor: JvmBytes(case.descriptor.as_bytes().to_vec()),
                },
                &mut budget(),
            )
            .unwrap();
            assert!(physical.stopped_at.is_none());
            // Existing method-wide quotes expose their block anchors, not one entry per
            // instruction. Preserve that contract: every published anchor has the exact
            // physical method and a real instruction BCI, and the actual store is inspectable.
            assert!(!body.source_map.is_empty());
            assert!(
                physical
                    .instructions
                    .iter()
                    .any(|instruction| { matches!(instruction.opcode, 0x3a | 0x4b | 0xc4) })
            );
            for segment in body.source_map.segments() {
                assert!(segment.start() < segment.end());
                assert!(body.text.get(segment.start()..segment.end()).is_some());
                for origin in
                    std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
                {
                    assert_eq!(origin.method(), Some(&method.item.identity));
                    assert!(
                        physical
                            .instructions
                            .iter()
                            .any(|instruction| instruction.bci == origin.bci()),
                        "{}: nonphysical BCI {}",
                        case.class,
                        origin.bci()
                    );
                }
            }
        }
    }
}

fn method_input(case: &Case) -> (ArtifactSnapshot, MethodAnalysisRequest) {
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(class_bytes(case)), &mut budget())
        .unwrap();
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget(),
            InspectionMode::Strict,
        )
        .unwrap();
    let environment = EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::SingleClass,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".into()),
    }
    .build(slice::from_ref(&snapshot))
    .unwrap();
    let request = MethodAnalysisRequest {
        environment,
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(case.method.as_bytes().to_vec()),
            descriptor: JvmBytes(case.descriptor.as_bytes().to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    (snapshot, request)
}

#[test]
fn fixed_parameter_refusal_preserves_budget_and_cancellation_first_causes() {
    use jarde::budget::{BudgetDimension, CountedBudgetDimension};
    for case in [supported(false).remove(0), refused(true).remove(0)] {
        let (snapshot, request) = method_input(&case);
        let run = |budget: &mut Budget| {
            Engine::new()
                .recover_method_with_evidence(
                    slice::from_ref(&snapshot),
                    SourceTarget::JAVA_8,
                    &request,
                    &RecoveryEvidenceRequest::essential(),
                    budget,
                )
                .unwrap()
                .recovery()
                .clone()
        };
        let mut measured = budget();
        let complete = run(&mut measured);
        assert!(complete.produced(), "{complete:?}");
        for (counted, expected) in [
            (
                CountedBudgetDimension::AnalysisSteps,
                BudgetDimension::AnalysisSteps,
            ),
            (CountedBudgetDimension::IrItems, BudgetDimension::IrItems),
            (
                CountedBudgetDimension::OutputBytes,
                BudgetDimension::OutputBytes,
            ),
        ] {
            let needed = measured.usage().counted_usage(counted);
            assert!(needed > 0);
            for limit in [needed, needed - 1] {
                let mut limits = task_limits(&[]).unwrap();
                match counted {
                    CountedBudgetDimension::AnalysisSteps => limits.analysis_steps = limit,
                    CountedBudgetDimension::IrItems => limits.ir_items = limit,
                    CountedBudgetDimension::OutputBytes => limits.output_bytes = limit,
                    _ => unreachable!(),
                }
                let bounded = run(&mut Budget::new(limits));
                if limit == needed {
                    assert!(bounded.produced(), "{bounded:?}");
                    assert_eq!(bounded.text, complete.text);
                } else {
                    assert!(
                        !bounded.produced()
                            && bounded.text.is_empty()
                            && bounded.source_map.is_empty(),
                        "{bounded:?}"
                    );
                    assert!(
                        matches!(bounded.stop(), Some(StopReason::Budget { dimension, .. }) if *dimension == expected),
                        "{bounded:?}"
                    );
                }
            }
        }
        let token = jarde::budget::CancellationToken::new();
        token.cancel();
        let cancelled = run(&mut Budget::with_cancellation_token(
            task_limits(&[]).unwrap(),
            token,
        ));
        assert!(
            !cancelled.produced() && cancelled.text.is_empty() && cancelled.source_map.is_empty(),
            "{cancelled:?}"
        );
        assert!(
            matches!(cancelled.stop(), Some(StopReason::Cancelled { .. })),
            "{cancelled:?}"
        );
    }
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jarde-reference-parameter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(program: &str, args: &[&str], dir: &std::path::Path) -> Vec<u8> {
    let output = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
#[ignore = "needs javac and java; compiles complete unedited Engine classes and executes both sides"]
fn complete_unedited_supported_classes_match_original_execution() {
    let scratch = Scratch::new();
    for wide in [false, true] {
        for case in supported(wide) {
            let original = scratch.0.join(format!("{}-original", case.class));
            let generated = scratch.0.join(format!("{}-generated", case.class));
            fs::create_dir(&original).unwrap();
            fs::create_dir(&generated).unwrap();
            fs::write(
                original.join(format!("{}.class", case.class)),
                class_bytes(&case),
            )
            .unwrap();
            let report = source(&case);
            assert_supported(&case, &report);
            let file = format!("{}.java", case.class);
            // No extraction, comment stripping, replacing declarations or repairing output.
            fs::write(generated.join(&file), &report.text).unwrap();
            let calls = case.arguments.iter().map(|arguments| format!(
                "try {{ Object value = {}.{}({arguments}); System.out.println(value == null ? \"null\" : value instanceof Object[] ? java.util.Arrays.deepToString((Object[]) value) : String.valueOf(value)); }} catch (Throwable failure) {{ System.out.println(failure.getClass().getName()); }}",
                case.class, case.method,
            )).collect::<Vec<_>>().join("\n");
            let harness = format!(
                "public class Probe {{ public static void main(String[] ignored) {{ {calls} }} }}"
            );
            for directory in [&original, &generated] {
                fs::write(directory.join("Probe.java"), &harness).unwrap();
            }
            run("javac", &["--release", "8", "Probe.java"], &original);
            run(
                "javac",
                &["--release", "8", &file, "Probe.java"],
                &generated,
            );
            assert_eq!(
                run("java", &["-Xverify:all", "-cp", ".", "Probe"], &original),
                run("java", &["-Xverify:all", "-cp", ".", "Probe"], &generated),
                "{}",
                case.class
            );
        }
    }
}

#[test]
#[ignore = "needs javac and java; regenerates each whole root, retaining its original dependency classes"]
fn complete_unedited_physical_parent_roots_match_original_execution() {
    let scratch = Scratch::new();
    for wide in [false, true] {
        let case = physical_parent_case(wide);
        let original = scratch.0.join(format!("{}-original", case.class));
        let generated = scratch.0.join(format!("{}-generated", case.class));
        fs::create_dir(&original).unwrap();
        fs::create_dir(&generated).unwrap();
        fs::write(
            original.join(format!("{}.class", case.class)),
            class_bytes(&case),
        )
        .unwrap();
        let report = source_from(
            &case,
            physical_parent_jar(&case, true),
            EnvironmentPolicy::PlainJar,
        );
        assert_supported(&case, &report);
        let file = format!("{}.java", case.class);
        fs::write(generated.join(&file), &report.text).unwrap();
        let harness = format!(
            "public class Probe {{ public static void main(String[] ignored) {{ ParameterParent previous = new ParameterParent(); ParameterChild child = new ParameterChild(); System.out.println({0}.{1}(previous, child) == child); System.out.println({0}.{1}(null, child) == child); System.out.println({0}.{1}(previous, null) == null); System.out.println({0}.{1}(null, null) == null); }} }}",
            case.class, case.method,
        );
        for directory in [&original, &generated] {
            // This gate regenerates the complete requested root only. Both auxiliary
            // classes remain the exact original bytes on both execution classpaths.
            for (name, bytes) in physical_headers() {
                fs::write(directory.join(name), bytes).unwrap();
            }
            fs::write(directory.join("Probe.java"), &harness).unwrap();
        }
        run("javac", &["--release", "8", "Probe.java"], &original);
        run(
            "javac",
            &["--release", "8", &file, "Probe.java"],
            &generated,
        );
        let expected = run("java", &["-Xverify:all", "-cp", ".", "Probe"], &original);
        assert_eq!(expected, b"true\ntrue\ntrue\ntrue\n");
        assert_eq!(
            expected,
            run("java", &["-Xverify:all", "-cp", ".", "Probe"], &generated),
            "{}",
            case.class
        );
    }
}
