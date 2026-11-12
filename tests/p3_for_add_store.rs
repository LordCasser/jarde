//! Add/store loop updates move to a `for` header only when the latch owns one pure chain.
//!
//! The labeled loop's label is spelled `loop` since `recover-labeled-loop-tail-coverage`: the
//! presentation replaces the synthesized `jarde_loop_{bci}` with a source-style name; the loop the
//! assertions target is unchanged.

use jarde::*;
use std::slice;

const SIMPLE: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-24/for-add-store-latch/simple/frozen/ForAddStoreSimple.class"
);
const LABELED: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-24/for-add-store-latch/frozen/ForAddStoreLatch.class"
);
const BOUNDARIES: &[u8] =
    include_bytes!("fixtures/p3-for-add-store/v8/ForAddStoreBoundaries.class");

fn method_text(class_bytes: &[u8], class_name: &str, method_name: &str) -> String {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget)
        .expect("frozen class opens");
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
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
    let OperationOutcome::Performed(report) = engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("single-class request succeeds")
    else {
        panic!("one class-source result is expected")
    };
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == method_name.as_bytes())
        .unwrap_or_else(|| panic!("the class has no `{method_name}` member"))
        .text
        .clone()
}

#[test]
fn pure_add_store_latch_is_one_for_update() {
    let method = method_text(SIMPLE, "ForAddStoreSimple", "run");
    assert!(!method.contains("@bytecode"), "{method}");
    assert!(
        method.contains("for (local3 = 0; local3 < arg0; local3 = local3 + arg1)"),
        "{method}"
    );
    assert_eq!(
        method.matches("local3 = local3 + arg1").count(),
        1,
        "{method}"
    );
    assert!(method.contains("local2 = local2 + local3;"), "{method}");
}

#[test]
fn outer_continue_runs_the_update_and_skips_the_body_tail() {
    let method = method_text(LABELED, "ForAddStoreLatch", "run");
    assert!(!method.contains("@bytecode"), "{method}");
    assert!(
        method.contains("loop: for (local3 = 0; local3 < arg0; local3 = local3 + arg1)"),
        "{method}"
    );
    assert!(method.contains("continue loop;"), "{method}");
    assert!(
        method.contains("for (local6 = 0; local6 < 2; local6 = local6 + 1)"),
        "{method}"
    );
    assert_eq!(
        method.matches("local3 = local3 + arg1").count(),
        1,
        "{method}"
    );
    let continue_at = method.find("continue loop;").unwrap();
    let body_tail_at = method.find("local5 = local5 + 1;").unwrap();
    assert!(continue_at < body_tail_at, "{method}");
}

#[test]
fn changed_step_extra_consumer_shared_effect_and_post_use_do_not_move() {
    for name in [
        "changedStep",
        "extraConsumer",
        "sharedTailEffect",
        "observedAfter",
    ] {
        let method = method_text(BOUNDARIES, "ForAddStoreBoundaries", name);
        assert!(!method.contains("for ("), "{name}: {method}");
        assert!(method.contains("while ("), "{name}: {method}");
        if name == "extraConsumer" {
            assert!(!method.contains("@bytecode"), "{method}");
            assert_sequential_statements(&method, false);
        } else {
            assert!(!method.contains("@bytecode"), "{name}: {method}");
        }
    }
    let changed = method_text(BOUNDARIES, "ForAddStoreBoundaries", "changedStep");
    assert!(changed.contains("arg1 = arg1 + 1;"), "{changed}");
    assert!(changed.contains("local3 = local3 + arg1;"), "{changed}");
    let shared = method_text(BOUNDARIES, "ForAddStoreBoundaries", "sharedTailEffect");
    assert!(shared.contains("local3 = local3 + 100;"), "{shared}");
    assert!(shared.contains("local4 = local4 + arg1;"), "{shared}");
    let post = method_text(BOUNDARIES, "ForAddStoreBoundaries", "observedAfter");
    assert!(post.contains("return local2;"), "{post}");
}

// Independent baseline exposure; original aggregate is retained above unchanged.
#[test]
fn changed_step_refusal_is_independently_exposed() {
    let changed = method_text(BOUNDARIES, "ForAddStoreBoundaries", "changedStep");
    assert!(
        changed.contains("while (") && !changed.contains("for (") && !changed.contains("@bytecode"),
        "{changed}"
    );
    assert!(changed.contains("arg1 = arg1 + 1;"), "{changed}");
    assert!(changed.contains("local3 = local3 + arg1;"), "{changed}");
}
#[test]
fn shared_tail_effect_is_independently_exposed() {
    let shared = method_text(BOUNDARIES, "ForAddStoreBoundaries", "sharedTailEffect");
    assert!(
        shared.contains("while (") && !shared.contains("for (") && !shared.contains("@bytecode"),
        "{shared}"
    );
    assert!(shared.contains("local3 = local3 + 100;"), "{shared}");
    assert!(shared.contains("local4 = local4 + arg1;"), "{shared}");
}
#[test]
fn observed_after_is_independently_exposed() {
    let post = method_text(BOUNDARIES, "ForAddStoreBoundaries", "observedAfter");
    assert!(
        post.contains("while (") && !post.contains("for (") && !post.contains("@bytecode"),
        "{post}"
    );
    assert!(post.contains("return local2;"), "{post}");
}

use std::{fs, path::Path, process::Command};
fn report(bytes: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
            loader: LoaderId("app".into()),
        },
    };
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("{other:?}"),
    }
}
fn javac(dir: &Path, files: &[&str], debug: bool) {
    let out = Command::new("javac")
        .args([
            "--release",
            "8",
            if debug { "-g" } else { "-g:none" },
            "-cp",
        ])
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(files.iter().map(|f| dir.join(f)))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
#[derive(Clone, Copy)]
enum SequentialShape {
    SharedRead,
    Loop,
    LocalAssignment,
}

fn check(
    shape: SequentialShape,
    class: &str,
    source: &str,
    driver: &str,
    debug: bool,
    case: &str,
) -> Vec<u32> {
    let root = std::env::temp_dir().join(format!("jarde-sequential-{case}-{}", std::process::id()));
    let original = root.join("original");
    let recovered = root.join("recovered");
    for dir in [&original, &recovered] {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("SequentialDriver.java"), driver).unwrap();
    }
    let file = format!("{class}.java");
    fs::write(original.join(&file), source).unwrap();
    javac(&original, &[&file, "SequentialDriver.java"], debug);
    let bytes = fs::read(original.join(format!("{class}.class"))).unwrap();
    let report = report(&bytes, class);
    fs::write(root.join("source-map-report.txt"), format!("{report:#?}")).unwrap();
    let member = report
        .methods
        .iter()
        .find(|m| !m.item.name.raw().0.starts_with(b"<"))
        .unwrap();
    let method = std::str::from_utf8(&member.item.name.raw().0).unwrap();
    let descriptor = if source.contains("int[] data") {
        "([II)I"
    } else {
        "(II)I"
    };
    let copies = assert_physical_sources(&bytes, &report, method, descriptor);
    match shape {
        SequentialShape::SharedRead => assert_sequential_statements(&member.text, true),
        SequentialShape::Loop => assert_sequential_statements(&member.text, false),
        SequentialShape::LocalAssignment => {}
    }
    if descriptor == "(II)I" {
        assert!(
            member.text.contains("while (") && !member.text.contains("for ("),
            "{}",
            member.text
        );
    }

    fs::write(recovered.join(&file), &report.text).unwrap();
    println!("CASE {case}: {}\n{}", root.display(), report.text);
    javac(&recovered, &[&file, "SequentialDriver.java"], debug);
    let mut outputs = Vec::new();
    for dir in [&original, &recovered] {
        let out = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(dir)
            .arg("SequentialDriver")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{case}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        fs::write(dir.join("stdout.txt"), &out.stdout).unwrap();
        fs::write(dir.join("stderr.txt"), &out.stderr).unwrap();
        outputs.push(out.stdout);
    }
    assert_eq!(outputs[0], outputs[1], "{case}");
    copies
}
#[test]
fn chained_read_whole_class_calibration() {
    let source = "public class SequentialRead { public static int run(int[] data, int index) { int sum=7; int pick=11; try { pick=sum=data[SequentialDriver.index(index)]; } catch(RuntimeException e) { return sum*100+pick; } return sum*100+pick; } }";
    let driver = "public class SequentialDriver { static int calls; static int index(int i) { calls++; return i; } public static void main(String[] args) { int[][] arrays={new int[]{3},null,new int[]{3}}; int[] indices={0,0,2}; int[] expected={303,711,711}; for(int i=0;i<3;i++) { calls=0; int value=SequentialRead.run(arrays[i],indices[i]); if(value!=expected[i] || calls!=1) throw new AssertionError(value+\":\"+calls); System.out.println(value+\":\"+calls); } } }";
    check(
        SequentialShape::SharedRead,
        "SequentialRead",
        source,
        driver,
        false,
        "read",
    );
}
#[test]
fn extra_consumer_whole_class_calibration() {
    let source = "public class SequentialLoop { public static int run(int limit, int step) { int sum=0; int i=0; while(i<limit) { sum+=(i=i+step); } return sum; } }";
    let driver = "public class SequentialDriver { public static void main(String[] args) { int[] limits={0,1,7,7}; int[] steps={1,2,2,3}; int[] expected={0,2,20,18}; for(int i=0;i<4;i++) { int value=SequentialLoop.run(limits[i],steps[i]); if(value!=expected[i]) throw new AssertionError(value); System.out.println(value); } } }";
    check(
        SequentialShape::Loop,
        "SequentialLoop",
        source,
        driver,
        false,
        "loop",
    );
}

#[test]
fn alias_read_before_mutation_calibration() {
    let source = "public class SequentialAlias { public static int run(int[] data, int index) { int sum=7; int pick=11; try { pick=sum=data[SequentialDriver.index(index)]; data[0]=99; } catch(RuntimeException e) { return sum*100+pick; } return sum*100+pick; } }";
    let driver = "public class SequentialDriver { static int calls; static int index(int i) { calls++; return i; } public static void main(String[] args) { int[] data={3}; int value=SequentialAlias.run(data,0); if(value!=303 || calls!=1 || data[0]!=99) throw new AssertionError(value+\":\"+calls+\":\"+data[0]); System.out.println(value+\":\"+calls+\":\"+data[0]); } }";
    check(
        SequentialShape::SharedRead,
        "SequentialAlias",
        source,
        driver,
        false,
        "alias",
    );
}
#[test]
fn effectful_step_calibration() {
    let source = "public class SequentialEffect { public static int run(int limit, int step) { int sum=0; int i=0; while(i<limit) { sum+=(i=i+SequentialDriver.step(step)); } return sum; } }";
    let driver = "public class SequentialDriver { static int calls; static int step(int n) { calls++; if(calls==2) throw new IllegalStateException(); return n; } public static void main(String[] args) { for(int limit:new int[]{0,1,7}) { calls=0; try { int result=SequentialEffect.run(limit,2); if(result!=(limit==0 ? 0 : 2) || calls!=(limit==0 ? 0 : 1)) throw new AssertionError(); System.out.println(result+\":\"+calls); } catch(IllegalStateException e) { if(limit!=7 || calls!=2) throw new AssertionError(); System.out.println(\"threw:\"+calls); } } } }";
    check(
        SequentialShape::Loop,
        "SequentialEffect",
        source,
        driver,
        false,
        "effect",
    );
}

fn assert_physical_sources(
    bytes: &[u8],
    report: &ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> Vec<u32> {
    use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
    let member = report
        .methods
        .iter()
        .find(|m| m.item.name.raw().0 == name.as_bytes())
        .unwrap();
    let ClassSourceOutcome::Recovered {
        report: recovered, ..
    } = &member.outcome
    else {
        panic!("{:?}", member.outcome)
    };
    assert_eq!(
        recovered.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );
    let physical = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut task_budget(&[]).unwrap(),
    )
    .unwrap();
    assert!(physical.stopped_at.is_none());
    for segment in recovered.source_map.segments() {
        assert!(segment.start() < segment.end());
        assert!(recovered.text.get(segment.start()..segment.end()).is_some());
        for origin in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(origin.method(), Some(&member.item.identity));
            assert!(physical.instructions.iter().any(|i| i.bci == origin.bci()));
        }
    }
    assert_eq!(recovered.content, RecoveryContent::ContainsStatements);
    assert!(recovered.fallbacks.is_empty());
    let missing: Vec<_> = physical
        .instructions
        .iter()
        .filter(|instruction| recovered.source_map.of_bci(instruction.bci).is_empty())
        .collect();
    // GOTO opcodes are excluded from this universal mapping assertion. Observed gaps in
    // generated read probes are try-exit transfers; this is not proof for every GOTO omission.
    // Reads, writes, copies, calls, arithmetic consumers and return/conditional branches remain required.
    assert!(
        missing.iter().all(|i| i.opcode == 0xa7),
        "unmapped physical instructions {missing:?}: {}",
        member.text
    );
    println!("unmapped GOTO instructions: {missing:?}");
    physical
        .instructions
        .iter()
        .filter(|i| i.opcode == 0x59)
        .map(|i| i.bci)
        .collect()
}

// Same-origin source-built invariance matrix, not an independent holdout corpus.
#[test]
fn sequential_copy_source_matrix_matches_whole_classes() {
    for debug in [false, true] {
        let mut baseline_read = None;
        let mut baseline_loop = None;
        for (variant, renamed, revalued, shifted) in [
            ("base", false, false, false),
            ("renamed", true, false, false),
            ("revalued", false, true, false),
            ("shifted", false, false, true),
            ("combined", true, true, true),
        ] {
            let method = if renamed { "evaluate" } else { "run" };
            let first = if renamed { "total" } else { "sum" };
            let second = if renamed { "selected" } else { "pick" };
            let old_first = if revalued { 17 } else { 7 };
            let old_second = if revalued { 19 } else { 11 };
            let element = if revalued { 5 } else { 3 };
            let pad = if shifted { 13 } else { 0 };
            let prefix = if shifted { "int padding=13;" } else { "" };
            let suffix = if shifted { "+padding" } else { "" };
            let class = if renamed {
                "AlteredSequentialRead"
            } else {
                "SequentialReadMatrix"
            };
            let source = format!(
                "public class {class} {{ public static int {method}(int[] data,int index) {{ {prefix} int {first}={old_first}; int {second}={old_second}; try {{ {second}={first}=data[SequentialDriver.index(index)]; }} catch(RuntimeException e) {{ return {first}*100+{second}{suffix}; }} return {first}*100+{second}{suffix}; }} }}"
            );
            let expected = element * 101 + pad;
            let failed = old_first * 100 + old_second + pad;
            let driver = format!(
                "public class SequentialDriver {{ static int calls; static int index(int i) {{ calls++; return i; }} public static void main(String[] args) {{ int[][] arrays={{new int[]{{{element}}},null,new int[]{{{element}}}}}; int[] indices={{0,0,2}}; int[] expected={{{expected},{failed},{failed}}}; for(int i=0;i<3;i++) {{ calls=0; int value={class}.{method}(arrays[i],indices[i]); if(value!=expected[i] || calls!=1) throw new AssertionError(value+\":\"+calls); System.out.println(value+\":\"+calls); }} }} }}"
            );
            let locations = check(
                SequentialShape::SharedRead,
                class,
                &source,
                &driver,
                debug,
                &format!("read-{variant}-{debug}"),
            );
            assert!(!locations.is_empty());
            if let Some(base) = &baseline_read {
                if shifted {
                    assert_ne!(&locations, base, "offset must alter physical copy BCI");
                } else {
                    assert_eq!(&locations, base);
                }
            } else {
                baseline_read = Some(locations);
            }
            let class = if renamed {
                "AlteredSequentialLoop"
            } else {
                "SequentialLoopMatrix"
            };
            let initial = if revalued { 5 } else { 0 };
            let source = format!(
                "public class {class} {{ public static int {method}(int limit,int step) {{ {prefix} int {first}={initial}; int i=0; while(i<limit) {{ {first}+=(i=i+step); }} return {first}{suffix}; }} }}"
            );
            let offset = initial + pad;
            let driver = format!(
                "public class SequentialDriver {{ public static void main(String[] args) {{ int[] limits={{0,1,7,7}}; int[] steps={{1,2,2,3}}; int[] expected={{{offset},{},{},{}}}; for(int i=0;i<4;i++) {{ int value={class}.{method}(limits[i],steps[i]); if(value!=expected[i]) throw new AssertionError(value); System.out.println(value); }} }} }}",
                offset + 2,
                offset + 20,
                offset + 18
            );
            let locations = check(
                SequentialShape::Loop,
                class,
                &source,
                &driver,
                debug,
                &format!("loop-{variant}-{debug}"),
            );
            assert!(!locations.is_empty());
            if let Some(base) = &baseline_loop {
                if shifted {
                    assert_ne!(&locations, base, "offset must alter physical copy BCI");
                } else {
                    assert_eq!(&locations, base);
                }
            } else {
                baseline_loop = Some(locations);
            }
        }
    }
}

fn assert_sequential_statements(text: &str, read: bool) {
    let lines: Vec<_> = text.lines().map(str::trim).collect();
    if read {
        let (at, line) = lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.contains(" = ") && line.contains("[SequentialDriver.index("))
            .expect("one observable read assignment");
        assert_eq!(text.matches("SequentialDriver.index(").count(), 1, "{text}");
        let first = line.split(" = ").next().unwrap();
        assert!(
            lines[at + 1].ends_with(&format!(" = {first};")),
            "read followed by its sequential local copy: {text}"
        );
    } else {
        let (at, line) = lines
            .iter()
            .enumerate()
            .find(|(_, line)| {
                line.starts_with("int ") && line.contains(" = ") && line.contains(" + ")
            })
            .expect("one saved addition");
        let name = line
            .strip_prefix("int ")
            .unwrap()
            .split(" = ")
            .next()
            .unwrap();
        assert!(
            lines[at + 1].ends_with(&format!(" = {name};")),
            "saved result assigned to induction before accumulation: {text}"
        );
        assert!(
            lines[at + 2].ends_with(&format!(" + {name};")),
            "one accumulation uses the same saved result: {text}"
        );
        assert_eq!(
            lines
                .iter()
                .filter(|s| s.ends_with(&format!(" = {name};")))
                .count(),
            1,
            "{text}"
        );
    }
}

#[test]
fn embedded_local_assignment_whole_class_matrix() {
    for debug in [false, true] {
        let mut baseline_copies = None;
        for (variant, renamed, revalued, shifted) in [
            ("base", false, false, false),
            ("renamed", true, false, false),
            ("revalued", false, true, false),
            ("shifted", false, false, true),
            ("combined", true, true, true),
        ] {
            let class = if renamed {
                "ChangedInterval"
            } else {
                "SequentialInterval"
            };
            let method = if renamed { "evaluate" } else { "run" };
            let sum = if renamed { "total" } else { "sum" };
            let k = if renamed { "assigned" } else { "k" };
            let old = if revalued { 17 } else { 7 };
            let assigned = if revalued { 4 } else { 2 };
            let element = if revalued { 5 } else { 3 };
            let pad = if shifted { 13 } else { 0 };
            let prefix = if shifted { "int padding=13;" } else { "" };
            let suffix = if shifted { "+padding" } else { "" };
            let source = format!(
                "public class {class} {{ public static int {method}(int[] data,int index) {{ {prefix} int {sum}={old}; int {k}=11; try {{ {sum}={sum}+({k}={assigned})+data[SequentialDriver.index(index)]; }} catch(RuntimeException e) {{ return {sum}*100+{k}{suffix}; }} return {sum}*100+{k}{suffix}; }} }}"
            );
            let success = (old + assigned + element) * 100 + assigned + pad;
            let failure = old * 100 + assigned + pad;
            let driver = format!(
                "public class SequentialDriver {{ static int calls; static int index(int i) {{ calls++; return i; }} public static void main(String[] args) {{ int[][] arrays={{new int[]{{{element}}},null,new int[]{{{element}}}}}; int[] indices={{0,0,2}}; int[] expected={{{success},{failure},{failure}}}; for(int i=0;i<3;i++) {{ calls=0; int result={class}.{method}(arrays[i],indices[i]); if(result!=expected[i] || calls!=1) throw new AssertionError(result+\":\"+calls); System.out.println(result+\":\"+calls); }} }} }}"
            );
            let copies = check(
                SequentialShape::LocalAssignment,
                class,
                &source,
                &driver,
                debug,
                &format!("interval-{variant}-{debug}"),
            );
            assert!(!copies.is_empty());
            if let Some(base) = &baseline_copies {
                if shifted {
                    assert_ne!(&copies, base);
                } else {
                    assert_eq!(&copies, base);
                }
            } else {
                baseline_copies = Some(copies);
            }
        }
    }
}
#[test]
fn intervening_external_effect_whole_class_calibration() {
    let source = "public class SequentialExternalEffect { public static int run(int[] data,int ignored) { int sum=7; int k=11; try { sum=sum+(k=SequentialDriver.effect(data))+data[0]; } catch(RuntimeException e) { return sum*100+k; } return sum*100+k; } }";
    let driver = "public class SequentialDriver { static int calls; static boolean boom; static int effect(int[] data) { calls++; data[0]=99; if(boom) throw new IllegalStateException(); return 2; } public static void main(String[] args) { for(int mode=0;mode<3;mode++) { calls=0; boom=mode==1; int[] data=mode==2 ? null : new int[]{3}; int value=SequentialExternalEffect.run(data,0); int expected=mode==0 ? 10802 : 711; if(value!=expected || calls!=1 || (data!=null && data[0]!=99)) throw new AssertionError(value+\":\"+calls); System.out.println(value+\":\"+calls); } } }";
    check(
        SequentialShape::LocalAssignment,
        "SequentialExternalEffect",
        source,
        driver,
        false,
        "external-effect",
    );
}

#[test]
fn frozen_loop_boundaries_match_whole_unedited_class() {
    let class = "ForAddStoreBoundaries";
    let report = report(BOUNDARIES, class);
    for (name, descriptor) in [
        ("changedStep", "(II)I"),
        ("extraConsumer", "(II)I"),
        ("sharedTailEffect", "(III)I"),
        ("observedAfter", "(II)I"),
    ] {
        assert_physical_sources(BOUNDARIES, &report, name, descriptor);
    }
    let root = std::env::temp_dir().join(format!("jarde-sequential-frozen-{}", std::process::id()));
    let original = root.join("original");
    let recovered = root.join("recovered");
    for dir in [&original, &recovered] {
        fs::create_dir_all(dir).unwrap();
    }
    fs::write(original.join(format!("{class}.class")), BOUNDARIES).unwrap();
    fs::write(recovered.join(format!("{class}.java")), &report.text).unwrap();
    fs::write(root.join("source-map-report.txt"), format!("{report:#?}")).unwrap();
    javac(&recovered, &["ForAddStoreBoundaries.java"], false);
    let driver = "public class SequentialDriver { public static void main(String[] args) { int[] actual={ForAddStoreBoundaries.changedStep(0,2),ForAddStoreBoundaries.changedStep(1,2),ForAddStoreBoundaries.changedStep(7,2),ForAddStoreBoundaries.extraConsumer(0,2),ForAddStoreBoundaries.extraConsumer(1,2),ForAddStoreBoundaries.extraConsumer(7,2),ForAddStoreBoundaries.sharedTailEffect(7,2,2),ForAddStoreBoundaries.observedAfter(7,2)}; int[] expected={0,0,3,0,2,20,410,8}; for(int i=0;i<actual.length;i++) { if(actual[i]!=expected[i]) throw new AssertionError(i+\":\"+actual[i]); System.out.println(actual[i]); } } }";
    let mut outputs = Vec::new();
    for dir in [&original, &recovered] {
        fs::write(dir.join("SequentialDriver.java"), driver).unwrap();
        javac(dir, &["SequentialDriver.java"], false);
        let out = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(dir)
            .arg("SequentialDriver")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        outputs.push(out.stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
    println!("FROZEN {}", root.display());
}
