use jarde::{
    Budget, BytecodeInspection, ExecutionReport, JvmBytes, Limits, MethodSelector,
    VerificationStatus, inspect_method_bytecode,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static UNIQUE: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-jvm-oracle-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create oracle temp directory");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn unlimited() -> Limits {
    Limits {
        input_bytes: u64::MAX,
        archive_entries: u64::MAX,
        entry_bytes: u64::MAX,
        read_bytes: u64::MAX,
        class_bytes: u64::MAX,
        attribute_bytes: u64::MAX,
        code_bytes: u64::MAX,
        result_items: u64::MAX,
        output_bytes: u64::MAX,
        elapsed_millis: u64::MAX,
    }
}

type MethodKey = (String, String);
type BoundaryFact = (u32, u8, u32);

#[derive(Debug)]
struct OracleOutput {
    runtime: String,
    sha256: String,
    version: (u16, u16),
    scope: String,
    facts: BTreeMap<MethodKey, Vec<BoundaryFact>>,
    ends: BTreeMap<MethodKey, u32>,
}

fn parse_oracle(stdout: &str) -> OracleOutput {
    let mut output = OracleOutput {
        runtime: String::new(),
        sha256: String::new(),
        version: (0, 0),
        scope: String::new(),
        facts: BTreeMap::new(),
        ends: BTreeMap::new(),
    };
    for line in stdout.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        match fields.as_slice() {
            ["META", "runtime", value] => output.runtime = (*value).to_owned(),
            ["META", "sha256", value] => output.sha256 = (*value).to_owned(),
            ["META", "version", major, minor] => {
                output.version = (major.parse().unwrap(), minor.parse().unwrap());
            }
            ["META", "scope", value] => output.scope = (*value).to_owned(),
            ["FACT", name, descriptor, bci, opcode, width] => {
                output
                    .facts
                    .entry(((*name).to_owned(), (*descriptor).to_owned()))
                    .or_default()
                    .push((
                        bci.parse().unwrap(),
                        opcode.parse().unwrap(),
                        width.parse().unwrap(),
                    ));
            }
            ["END", name, descriptor, end] => {
                output.ends.insert(
                    ((*name).to_owned(), (*descriptor).to_owned()),
                    end.parse().unwrap(),
                );
            }
            _ => panic!("unrecognized oracle output line: {line:?}"),
        }
    }
    assert!(!output.runtime.is_empty());
    assert_eq!(output.sha256.len(), 64);
    assert_eq!(output.version, (52, 0));
    assert_eq!(output.scope, "instruction_boundary_only_not_verification");
    output
}

fn inspect(bytes: &[u8], name: &str, descriptor: &str) -> BytecodeInspection {
    inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut Budget::new(unlimited()),
    )
    .unwrap()
}

fn slice(bytes: &[u8], start: u64, length: u64) -> &[u8] {
    let start = usize::try_from(start).unwrap();
    let end = start + usize::try_from(length).unwrap();
    &bytes[start..end]
}

#[test]
#[ignore = "requires JDK 25 Class-File API oracle"]
fn jdk25_instruction_boundaries_match_public_bytecode_inspection() {
    let temp = TempDir::new();
    let class_path = temp.0.join("OracleFixture.class");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/JvmBytecodeOracle.java");
    let process = Command::new("/usr/bin/java")
        .arg(&source)
        .arg(&class_path)
        .output()
        .expect("execute /usr/bin/java");
    assert!(
        process.status.success(),
        "Java oracle failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    let stdout = String::from_utf8(process.stdout).expect("oracle output is UTF-8");
    print!("{stdout}");
    let oracle = parse_oracle(&stdout);
    assert!(
        oracle.runtime.starts_with("25."),
        "runtime={}",
        oracle.runtime
    );
    let bytes = fs::read(&class_path).expect("read generated fixture class");

    for ((name, descriptor), expected) in &oracle.facts {
        let report = inspect(&bytes, name, descriptor);
        let actual: Vec<_> = report
            .instructions
            .iter()
            .map(|fact| (fact.bci, fact.opcode, fact.width))
            .collect();
        assert_eq!(&actual, expected, "method {name}{descriptor}");
        assert_eq!(
            actual.last().map(|(bci, _, width)| bci + width),
            oracle
                .ends
                .get(&(name.clone(), descriptor.clone()))
                .copied(),
            "method end {name}{descriptor}"
        );
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert_eq!(report.verification, VerificationStatus::NotPerformed);
        assert!(report.diagnostics.is_empty());

        if name == "dense" {
            assert!(
                actual
                    .iter()
                    .any(|(bci, opcode, _)| *opcode == 0xaa && *bci != 0)
            );
        } else if name == "sparse" {
            assert!(
                actual
                    .iter()
                    .any(|(bci, opcode, _)| *opcode == 0xab && *bci != 0)
            );
        } else if name == "refs" {
            for fact in report
                .instructions
                .iter()
                .filter(|fact| fact.opcode == 0xbb || fact.opcode == 0xb7)
            {
                let raw = slice(&bytes, fact.operands_span.start, fact.operands_span.length);
                let expected_index = u16::from_be_bytes([raw[0], raw[1]]);
                assert_eq!(fact.constant_pool_index, Some(expected_index));
            }
        }
    }
    assert_eq!(oracle.facts.len(), 3);
    println!(
        "SUMMARY\truntime={}\tsha256={}\tdense={}\tsparse={}\trefs={}",
        oracle.runtime,
        oracle.sha256,
        oracle
            .facts
            .iter()
            .find(|((name, _), _)| name == "dense")
            .unwrap()
            .1
            .len(),
        oracle
            .facts
            .iter()
            .find(|((name, _), _)| name == "sparse")
            .unwrap()
            .1
            .len(),
        oracle
            .facts
            .iter()
            .find(|((name, _), _)| name == "refs")
            .unwrap()
            .1
            .len(),
    );
}
