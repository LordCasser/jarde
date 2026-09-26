//! The release-8 two-resource TWR proof, including exact construction-site ownership.

use jarde::*;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SAMPLE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/release8/MultiResourceTwr.class"
);
const WRONG_MAIN_END: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/patched-negative/MultiResourceTwr.class"
);
const ORIGINAL_PROBE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/release8/MultiResourceTwr$Probe.class"
);
const ORIGINAL_BODY_FAILURE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/release8/MultiResourceTwr$BodyFailure.class"
);
const ORIGINAL_CLOSE_FAILURE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/release8/MultiResourceTwr$CloseFailure.class"
);
const ORIGINAL_RUNTIME: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-26/multi-resource-twr/release8/runtime.txt"
);

static NEXT_SCRATCH: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the epoch")
            .as_nanos();
        let sequence = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-multi-resource-twr-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create the TWR integration scratch directory");
        Self(path)
    }

    fn subdir(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir(&path).expect("create an isolated class directory");
        path
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_bounded(command: &mut Command, timeout: Duration) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("JDK command is available");
    let started = Instant::now();
    let status = loop {
        match child.try_wait().expect("poll JDK process") {
            Some(status) => break status,
            None if started.elapsed() < timeout => thread::sleep(Duration::from_millis(10)),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("JDK command exceeded {timeout:?}: {command:?}");
            }
        }
    };
    let stdout = read_pipe(child.stdout.take().expect("stdout is captured"));
    let stderr = read_pipe(child.stderr.take().expect("stderr is captured"));
    Output {
        status,
        stdout,
        stderr,
    }
}

fn read_pipe(mut pipe: impl Read) -> Vec<u8> {
    let mut contents = Vec::new();
    pipe.read_to_end(&mut contents).expect("read child output");
    contents
}

fn install_original_family(directory: &Path) {
    for (name, bytes) in [
        ("MultiResourceTwr.class", SAMPLE),
        ("MultiResourceTwr$Probe.class", ORIGINAL_PROBE),
        ("MultiResourceTwr$BodyFailure.class", ORIGINAL_BODY_FAILURE),
        (
            "MultiResourceTwr$CloseFailure.class",
            ORIGINAL_CLOSE_FAILURE,
        ),
    ] {
        fs::write(directory.join(name), bytes).expect("write the frozen Java 8 class family");
    }
}

fn recovered_run_method(text: &str) -> &str {
    let signature = "private static int run() throws java.lang.Exception {";
    let start = text
        .find(signature)
        .expect("the current Engine report contains the run() declaration");
    let open = text[start..]
        .find('{')
        .map(|offset| start + offset)
        .expect("run() opens a method body");
    let mut depth = 0usize;
    let end = text[open..]
        .char_indices()
        .find_map(|(offset, ch)| match ch {
            '{' => {
                depth += 1;
                None
            }
            '}' => {
                depth -= 1;
                (depth == 0).then_some(open + offset + ch.len_utf8())
            }
            _ => None,
        })
        .expect("run() has a matching method brace");
    let method = &text[start..end];
    assert!(
        !method.contains("@bytecode"),
        "run() is completely recovered: {method}"
    );
    method
}

fn recovered_harness(method: &str) -> String {
    let prefix = r#"public final class RecoveredRunHarness {
    private static final StringBuilder EVENTS = new StringBuilder();
    static boolean bodyFails;
    static boolean innerCloseFails;
    static boolean outerCloseFails;

    static void event(String value) {
        if (EVENTS.length() != 0) EVENTS.append(',');
        EVENTS.append(value);
    }

    private static void maybeFailBody() throws Exception {
        if (bodyFails) throw new BodyFailure();
    }

"#;
    let suffix = r#"
    public static void main(String[] args) {
        String mode = args.length == 0 ? "normal" : args[0];
        EVENTS.setLength(0);
        try {
            bodyFails = mode.equals("body") || mode.equals("suppressed");
            innerCloseFails = mode.equals("inner-close") || mode.equals("suppressed");
            outerCloseFails = mode.equals("outer-close") || mode.equals("suppressed");
            int result = run();
            System.out.println("result=" + result + ";events=" + EVENTS);
        } catch (Throwable failure) {
            StringBuilder suppressed = new StringBuilder();
            for (Throwable item : failure.getSuppressed()) {
                if (suppressed.length() != 0) suppressed.append(',');
                suppressed.append(item.getClass().getSimpleName()).append(':').append(item.getMessage());
            }
            System.out.println("throw=" + failure.getClass().getSimpleName() + ':' + failure.getMessage()
                    + ";suppressed=" + suppressed + ";events=" + EVENTS);
        }
    }
}

final class MultiResourceTwr$Probe implements AutoCloseable {
    private final String name;
    MultiResourceTwr$Probe(String name) {
        this.name = name;
        RecoveredRunHarness.event("open-" + name);
    }
    int read() {
        RecoveredRunHarness.event("read");
        return 42;
    }
    public void close() throws Exception {
        RecoveredRunHarness.event("close-" + name);
        if ((name.equals("inner") && RecoveredRunHarness.innerCloseFails)
                || (name.equals("outer") && RecoveredRunHarness.outerCloseFails)) {
            throw new CloseFailure(name);
        }
    }
}

final class BodyFailure extends Exception {
    BodyFailure() { super("body"); }
}

final class CloseFailure extends Exception {
    CloseFailure(String name) { super("close-" + name); }
}
"#;
    let mut source = String::with_capacity(prefix.len() + method.len() + suffix.len());
    source.push_str(prefix);
    source.push_str(method);
    source.push_str(suffix);
    source
}

fn five_mode_trace(class_dir: &Path, class_name: &str) -> String {
    ["normal", "body", "inner-close", "outer-close", "suppressed"]
        .into_iter()
        .map(|mode| {
            let output = run_bounded(
                Command::new("java")
                    .arg("-Xverify:all")
                    .arg("-cp")
                    .arg(class_dir)
                    .arg(class_name)
                    .arg(mode),
                Duration::from_secs(10),
            );
            assert!(
                output.status.success(),
                "{class_name} {mode} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            format!(
                "{mode} {}",
                String::from_utf8(output.stdout)
                    .expect("Java output is UTF-8")
                    .trim_end()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
#[ignore = "needs a JDK on PATH to compile the exact recovered run() text and execute both sides"]
fn recovered_run_compiles_as_java_8_and_matches_original_five_mode_trace() {
    let report = class_source(SAMPLE);
    let (method_text, recovered) = run_report(&report);
    assert!(
        !recovered
            .fallbacks
            .iter()
            .any(|code| code.starts_with("jre_")),
        "the method compiled below must be the current Engine report: {:?}; {:?}",
        recovered.fallbacks,
        recovered.diagnostics
    );
    let method = recovered_run_method(method_text);
    let scratch = Scratch::new();
    let original_dir = scratch.subdir("original");
    install_original_family(&original_dir);

    // This test isolates the exact recovered member: the rest of class-source's full-class output
    // contains unrelated method fallbacks, so a controlled harness supplies only run()'s support
    // methods and resource implementation. The original side executes the complete frozen class
    // family unchanged, and both traces are pinned to the original release-8 runtime evidence.
    let expected = ORIGINAL_RUNTIME.trim_end();
    assert_eq!(five_mode_trace(&original_dir, "MultiResourceTwr"), expected);

    let recovered_dir = scratch.subdir("recovered");
    fs::write(
        scratch.path().join("RecoveredRunHarness.java"),
        recovered_harness(method),
    )
    .expect("write wrapper with the exact run() text from Engine");
    let compile = run_bounded(
        Command::new("javac")
            .arg("--release")
            .arg("8")
            .arg("-g:none")
            .arg("-d")
            .arg(&recovered_dir)
            .arg(scratch.path().join("RecoveredRunHarness.java")),
        Duration::from_secs(30),
    );
    assert!(
        compile.status.success(),
        "javac --release 8 rejected the exact recovered run(): {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    assert_eq!(
        five_mode_trace(&recovered_dir, "RecoveredRunHarness"),
        expected,
        "the recovered resource header preserves return, exception, and suppressed-close order"
    );
}

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn class_source(bytes: &[u8]) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen or table-patched class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("MultiResourceTwr"),
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
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("one frozen class answers the class-source request")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => {
            panic!(
                "one class matched {} candidates",
                candidates.candidates.len()
            )
        }
        OperationOutcome::Incomplete(candidates) => {
            panic!(
                "class-source stopped at {} candidates",
                candidates.candidates.len()
            )
        }
    }
}

fn run_report(report: &ClassSourceReport) -> (&str, &RecoveryReport) {
    let member = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == b"run")
        .expect("the frozen class has run()");
    match &member.outcome {
        ClassSourceOutcome::Recovered { report, .. } => (&member.text, report),
        other => panic!("run() has a recovery report: {other:?}"),
    }
}

/// Find run()'s exception table by parsing only the class-file envelope needed for this fixture.
fn run_table(bytes: &[u8]) -> (usize, usize, Vec<(usize, [u8; 8])>) {
    fn u2(bytes: &[u8], at: usize) -> u16 {
        u16::from_be_bytes([bytes[at], bytes[at + 1]])
    }
    fn u4(bytes: &[u8], at: usize) -> usize {
        u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize
    }
    fn skip_attributes(bytes: &[u8], mut at: usize, count: u16) -> usize {
        for _ in 0..count {
            at += 2 + u4(bytes, at + 2);
        }
        at
    }
    fn skip_members(bytes: &[u8], mut at: usize, count: u16) -> usize {
        for _ in 0..count {
            let attributes = u2(bytes, at + 6);
            at = skip_attributes(bytes, at + 8, attributes);
        }
        at
    }

    let mut at = 8;
    let cp_count = u2(bytes, at);
    at += 2;
    let mut utf8 = vec![None; cp_count as usize];
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
    let fields = u2(bytes, at);
    at = skip_members(bytes, at + 2, fields);
    let methods = u2(bytes, at);
    at += 2;
    for _ in 0..methods {
        let name = utf8[u2(bytes, at + 2) as usize].as_deref();
        let attributes = u2(bytes, at + 6);
        at += 8;
        for _ in 0..attributes {
            let attribute_name = utf8[u2(bytes, at) as usize].as_deref();
            let length = u4(bytes, at + 2);
            let body = at + 6;
            if name == Some("run") && attribute_name == Some("Code") {
                let code_length = u4(bytes, body + 4);
                let count_at = body + 8 + code_length;
                let count = u2(bytes, count_at);
                let table_at = count_at + 2;
                let rows = (0..count)
                    .map(|row| {
                        let row_at = table_at + row as usize * 8;
                        (row_at, bytes[row_at..row_at + 8].try_into().unwrap())
                    })
                    .collect();
                return (at + 2, count_at, rows);
            }
            at += 6 + length;
        }
    }
    panic!("run() Code attribute exists")
}

fn companion_row(bytes: &[u8]) -> (usize, [u8; 8]) {
    let (_, _, rows) = run_table(bytes);
    rows.into_iter()
        .find(|(_, row)| row[..6] == [0, 43, 0, 59, 0, 59])
        .expect("fixture companion row is [43,59) -> 59")
}

fn patch_companion(mut bytes: Vec<u8>, patch: impl FnOnce(&mut [u8; 8])) -> Vec<u8> {
    let (at, mut row) = companion_row(&bytes);
    patch(&mut row);
    bytes[at..at + 8].copy_from_slice(&row);
    bytes
}

#[test]
fn two_resource_header_owns_each_construction_and_preserves_close_evidence() {
    let report = class_source(SAMPLE);
    let (text, recovered) = run_report(&report);
    assert!(
        !recovered
            .fallbacks
            .iter()
            .any(|code| code.starts_with("jre_")),
        "the frozen method should recover as Java: {:?}; {:?}",
        recovered.fallbacks,
        recovered.diagnostics
    );
    assert!(
        text.contains("try (MultiResourceTwr$Probe local0 = new MultiResourceTwr$Probe(\"outer\"); MultiResourceTwr$Probe local1 = new MultiResourceTwr$Probe(\"inner\")) {"),
        "both declarations belong to one header: {text}"
    );
    assert!(
        text.contains("return local2;"),
        "the saved return stays inside the resource body: {text}"
    );
    assert!(!text.contains("@bytecode"), "{text}");
    let header_bcis = [0, 3, 4, 6, 9, 10, 13, 14, 16, 19];
    let cleanup_bcis = [
        33, 34, 37, 38, 41, 42, 43, 44, 45, 48, 51, 52, 53, 54, 57, 58, 59, 60, 61, 64, 67, 68, 69,
        70, 73, 74,
    ];
    for bci in header_bcis {
        assert!(
            recovered
                .source_map
                .text_of_bci(&text, bci)
                .iter()
                .any(|mapped| {
                    mapped.contains("try (MultiResourceTwr$Probe local0 =")
                        && mapped.contains("local1 = new MultiResourceTwr$Probe(\"inner\")")
                }),
            "resource initializer BCI {bci} must map to the complete multi-resource header"
        );
    }
    for bci in header_bcis.into_iter().chain(cleanup_bcis) {
        assert!(
            !recovered.source_map.of_bci(bci).is_empty(),
            "the header, close or suppression instruction at BCI {bci} needs an anchor"
        );
    }
}

#[test]
fn a_main_row_that_ends_at_the_inner_close_is_refused() {
    let report = class_source(WRONG_MAIN_END);
    let (text, recovered) = run_report(&report);
    assert!(
        recovered.fallbacks.contains(&"jre_guard_handler_range"),
        "the outer row must end at its own close group, not the inner one: {:?}",
        recovered.fallbacks
    );
    assert!(!text.contains("try ("), "{text}");
}

#[test]
fn malformed_companion_ranges_types_and_targets_remain_unexplained() {
    let cases: [(&str, Box<dyn Fn(&mut [u8; 8])>); 5] = [
        (
            "missing",
            Box::new(|row| row[..4].copy_from_slice(&[0, 59, 0, 60])),
        ),
        (
            "partial",
            Box::new(|row| row[2..4].copy_from_slice(&58u16.to_be_bytes())),
        ),
        (
            "overlap",
            Box::new(|row| row[..2].copy_from_slice(&42u16.to_be_bytes())),
        ),
        (
            "wrong target",
            Box::new(|row| row[4..6].copy_from_slice(&43u16.to_be_bytes())),
        ),
        (
            "wrong type",
            Box::new(|row| row[6..8].copy_from_slice(&0u16.to_be_bytes())),
        ),
    ];
    for (name, patch) in cases {
        let bytes = patch_companion(SAMPLE.to_vec(), |row| patch(row));
        let report = class_source(&bytes);
        let (text, recovered) = run_report(&report);
        if name == "wrong target" {
            // This verifier-valid target mutation redirects the companion into the inner handler
            // itself, so Guard cannot identify that handler as a unique close level. Region keeps
            // the method quoted because exceptional blocks remain uncovered; requiring Guard to
            // claim a row whose handler identity is ambiguous would widen the proof unsafely.
            assert!(
                recovered.fallbacks.contains(&"jre_region_uncovered_blocks"),
                "{name}: the redirected exceptional path remains refused: {:?}; {:?}\n{text}",
                recovered.fallbacks,
                recovered.diagnostics
            );
        } else {
            assert!(
                recovered.fallbacks.contains(&"jre_guard_unexplained_row")
                    || recovered.fallbacks.contains(&"jre_guard_handler_range")
                    || recovered.fallbacks.contains(&"jre_guard_continuation"),
                "{name}: Guard owns the refusal for malformed companion geometry: {:?}; {:?}\n{text}",
                recovered.fallbacks,
                recovered.diagnostics
            );
        }
        assert!(
            !text.contains("try (") && text.contains("@bytecode"),
            "{name}: the malformed companion keeps the whole method quoted: {text}"
        );
    }
}

#[test]
fn extra_exact_companion_is_not_silently_absorbed() {
    let (code_length_at, count_at, rows) = run_table(SAMPLE);
    let (_, row) = companion_row(SAMPLE);
    let mut bytes = SAMPLE.to_vec();
    let count = u16::from_be_bytes(bytes[count_at..count_at + 2].try_into().unwrap());
    let table_end = rows.last().unwrap().0 + 8;
    bytes.splice(table_end..table_end, row);
    bytes[count_at..count_at + 2].copy_from_slice(&(count + 1).to_be_bytes());
    // The Code attribute length includes the exception-table entry just inserted.
    let code_length = u32::from_be_bytes(
        bytes[code_length_at..code_length_at + 4]
            .try_into()
            .unwrap(),
    );
    bytes[code_length_at..code_length_at + 4].copy_from_slice(&(code_length + 8).to_be_bytes());
    let report = class_source(&bytes);
    let (text, recovered) = run_report(&report);
    assert!(
        recovered.fallbacks.contains(&"jre_guard_handler_range")
            || recovered.fallbacks.contains(&"jre_guard_unexplained_row"),
        "the duplicate protection is ambiguous and refused: {:?}; {:?}\n{text}",
        recovered.fallbacks,
        recovered.diagnostics
    );
    assert!(!text.contains("try ("), "{text}");
}
