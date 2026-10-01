//! `recover-platform-collection-widening`: the JDK 8 `java.util` collection hierarchy one call
//! argument widens to its ancestor interface — the collection API shape where a concrete
//! implementation is passed to a parameter declared as the interface.
//!
//! The committed inputs are the patrol's own `G1.class` and this slice's `CWV.class` / `CWN.class`
//! (javac 23.0.1 `--release 8 -g:none`, SHA-256 in the evidence README), and the recovered source
//! of the positive classes recompiles under `javac --release 8` and runs identically to the frozen
//! original under `java -Xverify:all`.
//!
//! * the **text**: `G1.use` presents `max((java.util.List) local0)` — the required spelling keeps
//!   the call on the descriptor's own parameter, and the result local is usable again;
//! * the **closed set**: every positive variant recovers its interface argument and every negative
//!   keeps its refusal verbatim (user classes, user subclasses, `java.util.concurrent`, and the
//!   `java.util` types this slice leaves out);
//! * the **runtime**: the frozen originals and the recovered classes print the same lines under
//!   `java -Xverify:all`; the recorded JADX column (`results/jadx-*-run.txt`) matched when the
//!   evidence was frozen.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const G1: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/collection-widening-patrol/fixture/G1.class"
);
const CWV: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/collection-widening-patrol/widen/original/CWV.class"
);
const CWN: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/collection-widening-patrol/widen/original/CWN.class"
);

/// The one run of the original `G1.main` under `java -Xverify:all` (`fixture/orig.out`).
const G1_EXPECTED_RUN: &str = "zeta:a\n6\n";

/// The one run of the original `CWV.main`: one labelled line per closed-set row the family
/// exercises.
const CWV_EXPECTED_RUN: &str = "\
map:hashmap
treemap:treemap
linkedmap:linkedmap
hashtable:hashtable
properties:1
set:1
treeset:1
linkedhashset:1
collection:2
list:coll
iterable:2
linkedlist:linked
vector:1
stack:stack
nestedList:inner
nested:2
deque:1
dequeCollection:1
listToCollection:2
setToCollection:1
queueToCollection:1
collectionToIterable:2
dequeToQueue:1
sortedSetToSet:1
navigableSetToSortedSet:1
navigableMapToSortedMap:navigablemapinterface
";

fn opened(bytes: &[u8], class: &str) -> (artifact::ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
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
            loader: LoaderId("app".to_owned()),
        },
    };
    (snapshot, request)
}

fn class_source_of(
    bytes: &[u8],
    class: &str,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let (snapshot, request) = opened(bytes, class);
    Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            budget,
        )
        .unwrap()
}

fn recover(bytes: &[u8], class: &str) -> ClassSourceReport {
    let mut budget = task_budget(&[]).unwrap();
    match class_source_of(bytes, class, &mut budget) {
        OperationOutcome::Performed(report) => report,
        other => panic!("complete class source expected: {other:?}"),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn scratch(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "jarde-p3-collection-widening-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create comparison directory");
    root
}

fn compile(dir: &Path, sources: &[&str]) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-Xlint:-options")
        .arg("-cp")
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(sources.iter().map(|source| dir.join(source)))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run(dir: &Path, main_class: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(main_class)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn g1_use_recovers_the_collection_argument_and_its_result_local() {
    let report = recover(G1, "G1");
    let text = &report.text;
    assert!(
        !text.contains("no safe reference conversion evidence"),
        "the collection argument refusal is closed:\n{text}"
    );
    let use_body = &member(&report, "use").text;
    assert!(
        !use_body.contains("@bytecode"),
        "use still quotes bytecode:\n{use_body}"
    );
    // The required spelling keeps the call on the descriptor's own `List` parameter, and the
    // result local that the refusal used to strand is declared and returned.
    for statement in [
        "java.lang.String local1 = (java.lang.String) max((java.util.List) local0);",
        "java.lang.String local2 = (java.lang.String) pick((java.lang.Object) \"a\", (java.lang.Object) \"b\");",
        "return local1 + \":\" + local2;",
    ] {
        assert!(
            use_body.contains(statement),
            "use misses `{statement}`:\n{use_body}"
        );
    }
    // The methods the baseline already recovered keep their exact presentation: only the one
    // refusal site changed.
    assert!(
        member(&report, "max")
            .text
            .contains("local1 = (java.lang.Comparable) arg0.get(0);"),
        "max changed:\n{}",
        member(&report, "max").text
    );
    // The unrelated `autoboxLoop` gap (BCI 44) is a different family and stays as it was.
    assert_eq!(text.matches("@bytecode").count(), 1, "{text}");
}

#[test]
fn recovered_g1_runtime_matches_the_original_class() {
    let report = recover(G1, "G1");

    let original_dir = scratch("g1-original");
    fs::write(original_dir.join("G1.class"), G1).unwrap();
    let original_run = run(&original_dir, "G1");

    let recovered_dir = scratch("g1-recovered");
    fs::write(recovered_dir.join("G1.java"), &report.text).unwrap();
    compile(&recovered_dir, &["G1.java"]);
    let recovered_run = run(&recovered_dir, "G1");

    assert_eq!(original_run, G1_EXPECTED_RUN, "{original_run}");
    assert_eq!(recovered_run, original_run, "the recovered class diverges");
}

#[test]
fn collection_family_recovers_every_row_with_the_required_spelling() {
    let report = recover(CWV, "CWV");
    let text = &report.text;
    assert!(
        !text.contains("no safe reference conversion evidence"),
        "no family member may refuse:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "the whole class presents:\n{text}"
    );

    // One representative per closed-set shape: the `Map`/`Set`/`List`/`Deque` implementation rows,
    // the interface rows, and the multi-level walks (`ArrayList -> Collection -> Iterable`,
    // `TreeSet -> NavigableSet -> SortedSet -> Set`, `Properties -> Hashtable -> Map`).
    for statement in [
        "java.lang.System.out.println(\"map:\" + mapValue((java.util.Map) local1));",
        "java.lang.System.out.println(\"linkedmap:\" + mapValue((java.util.Map) local3));",
        "java.lang.System.out.println(\"hashtable:\" + mapValue((java.util.Map) local4));",
        "java.lang.System.out.println(\"properties:\" + mapSize((java.util.Map) local5));",
        "java.lang.System.out.println(\"treeset:\" + setSize((java.util.Set) local7));",
        "java.lang.System.out.println(\"list:\" + listValue((java.util.List) local9));",
        "java.lang.System.out.println(\"iterable:\" + iterableSize((java.lang.Iterable) local9));",
        "java.lang.System.out.println(\"stack:\" + listValue((java.util.List) local12));",
        "java.lang.System.out.println(\"deque:\" + dequeSize((java.util.Deque) local16));",
        "java.lang.System.out.println(\"dequeToQueue:\" + dequeToQueue((java.util.Deque) local21));",
        "java.lang.System.out.println(\"sortedSetToSet:\" + sortedSetToSet((java.util.SortedSet) local22));",
        "java.lang.System.out.println(\"navigableMapToSortedMap:\" + navigableMapToSortedMap((java.util.NavigableMap) local24));",
    ] {
        assert!(
            text.contains(statement),
            "family misses `{statement}`:\n{text}"
        );
    }
    // The interface-to-interface call sites present the parameter's own frame type and cross one
    // interface row, so the required spelling appears there too.
    for (name, statement) in [
        (
            "listToCollection",
            "collectionSize((java.util.Collection) arg0)",
        ),
        (
            "setToCollection",
            "collectionSize((java.util.Collection) arg0)",
        ),
        (
            "queueToCollection",
            "collectionSize((java.util.Collection) arg0)",
        ),
        (
            "collectionToIterable",
            "iterableSize((java.lang.Iterable) arg0)",
        ),
        ("dequeToQueue", "queueSize((java.util.Queue) arg0)"),
        ("sortedSetToSet", "setSize((java.util.Set) arg0)"),
        (
            "navigableSetToSortedSet",
            "sortedSetSize((java.util.SortedSet) arg0)",
        ),
        (
            "navigableMapToSortedMap",
            "sortedMapValue((java.util.SortedMap) arg0)",
        ),
    ] {
        assert!(
            member(&report, name).text.contains(statement),
            "`{name}` misses `{statement}`:\n{}",
            member(&report, name).text
        );
    }

    let original_dir = scratch("cwv-original");
    fs::write(original_dir.join("CWV.class"), CWV).unwrap();
    let original_run = run(&original_dir, "CWV");

    let recovered_dir = scratch("cwv-recovered");
    fs::write(recovered_dir.join("CWV.java"), &report.text).unwrap();
    compile(&recovered_dir, &["CWV.java"]);
    let recovered_run = run(&recovered_dir, "CWV");

    assert_eq!(original_run, CWV_EXPECTED_RUN, "{original_run}");
    assert_eq!(recovered_run, original_run, "the recovered family diverges");
}

#[test]
fn table_out_presentations_keep_the_refusal_and_the_original_runtime() {
    let report = recover(CWN, "CWN");
    let text = &report.text;
    assert_eq!(
        text.matches("no safe reference conversion evidence")
            .count(),
        7,
        "the seven table-out calls stay refused:\n{text}"
    );

    // A user class that reaches `List` only through `AbstractList`: no row states the user type.
    let my_list = &member(&report, "main").text;
    assert!(
        my_list.contains("presents `CWN$MyList` but the invocation requires `java.util.List`"),
        "{my_list}"
    );
    // A user subclass of a table class is still a user type.
    assert!(
        my_list.contains("presents `CWN$MySubList` but the invocation requires `java.util.List`"),
        "{my_list}"
    );
    // The `java.util` subpackage.
    assert!(
        my_list.contains(
            "presents `java.util.concurrent.ConcurrentHashMap` but the invocation requires `java.util.Map`"
        ),
        "{my_list}"
    );
    // The `java.util` types this slice leaves out of the enumerated domain.
    assert!(
        my_list
            .contains("presents `java.util.EnumSet` but the invocation requires `java.util.Set`"),
        "{my_list}"
    );
    assert!(
        my_list.contains(
            "presents `java.util.IdentityHashMap` but the invocation requires `java.util.Map`"
        ),
        "{my_list}"
    );
    // A `java.util` type outside the collection hierarchy, and the enum constant family.
    assert!(
        my_list.contains(
            "presents `java.util.Date` but the invocation requires `java.lang.Comparable`"
        ),
        "{my_list}"
    );
    assert!(
        my_list.contains("presents `CWN$Kind` but the invocation requires `java.lang.Enum`"),
        "{my_list}"
    );

    // The original class's own runtime is valid even where its recovery must stay refused.
    let original_dir = scratch("cwn-original");
    for (name, bytes) in [
        ("CWN.class", CWN),
        (
            "CWN$MyList.class",
            include_bytes!(
                "../openspec/evidence/java-syntax-2026-10-02/collection-widening-patrol/widen/original/CWN$MyList.class"
            )
            .as_slice(),
        ),
        (
            "CWN$MySubList.class",
            include_bytes!(
                "../openspec/evidence/java-syntax-2026-10-02/collection-widening-patrol/widen/original/CWN$MySubList.class"
            )
            .as_slice(),
        ),
        (
            "CWN$Kind.class",
            include_bytes!(
                "../openspec/evidence/java-syntax-2026-10-02/collection-widening-patrol/widen/original/CWN$Kind.class"
            )
            .as_slice(),
        ),
    ] {
        fs::write(original_dir.join(name), bytes).unwrap();
    }
    let original_run = run(&original_dir, "CWN");
    assert_eq!(
        original_run,
        "myList:user\nsubList:sub\nconcurrent:concurrent\nenumSet:1\nidentity:identity\ndate:28\n",
        "{original_run}"
    );
}

#[test]
fn collection_recovery_stops_and_cancels_before_any_published_output() {
    let mut bounded =
        task_budget(&[BudgetOverride::new("output_bytes", 8).expect("a legal output cap")])
            .unwrap();
    match class_source_of(CWV, "CWV", &mut bounded) {
        OperationOutcome::Incomplete(selection) => {
            assert!(
                !matches!(selection.execution, ExecutionReport::Complete { .. }),
                "an eight-byte output budget is not a complete run"
            );
        }
        OperationOutcome::Performed(report) => panic!(
            "a stopped delivery must not publish a full report: {}",
            report.text
        ),
        other => panic!("the bounded class source answered {other:?}"),
    }

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut cancelled =
        Budget::with_cancellation_token(task_budget(&[]).unwrap().limits().clone(), cancellation);
    match class_source_of(CWV, "CWV", &mut cancelled) {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => panic!(
            "a cancelled delivery must not publish a full report: {}",
            report.text
        ),
        other => panic!("the cancelled class source answered {other:?}"),
    }
}
