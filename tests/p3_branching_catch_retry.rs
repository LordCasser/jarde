//! Public whole-class regression for a proved conditional named-catch retry loop.
use jarde::*;
use jarde_jvm::ir::Quality;
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use std::collections::BTreeSet;
use std::slice;

const DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-branching-named-catch-retry-loops/original/debug/TestBreakInLoop2$TestCls.class"
);
const NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-branching-named-catch-retry-loops/original/no-debug/TestBreakInLoop2$TestCls.class"
);
const OWNER: &str = "jadx/tests/integration/loops/TestBreakInLoop2$TestCls";
const TARGET: &str = "test";
const DESCRIPTOR: &str = "(Ljava/util/List;)V";

struct Input {
    name: &'static str,
    bytes: &'static [u8],
    owner: &'static str,
}

fn request(snapshot: &ArtifactSnapshot, owner: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(owner),
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
    }
}

fn report(input: &Input, all: bool) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded request budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(input.bytes.to_vec()), &mut budget)
        .unwrap_or_else(|error| panic!("{}: input opens: {error:?}", input.name));
    let request = request(&snapshot, input.owner);
    let result = if all {
        engine.class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(&snapshot), &request, &mut budget)
    }
    .unwrap_or_else(|error| panic!("{}: class source: {error:?}", input.name));
    match result {
        OperationOutcome::Performed(report) => report,
        other => panic!(
            "{}: complete class-source result required: {other:?}",
            input.name
        ),
    }
}

fn method<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name.as_bytes()
                && method.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("missing method {name}{descriptor}"))
}

fn method_keys(report: &ClassSourceReport) -> BTreeSet<(Vec<u8>, Vec<u8>)> {
    report
        .methods
        .iter()
        .map(|m| {
            (
                m.item.name.raw().0.clone(),
                m.item.descriptor.raw().0.clone(),
            )
        })
        .collect()
}

fn physical_bcis(bytes: &[u8], name: &str, descriptor: &str) -> BTreeSet<u32> {
    let mut budget = task_budget(&[]).expect("reader budget");
    let facts = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut budget,
    )
    .expect("original method bytecode is inspectable");
    assert!(facts.stopped_at.is_none(), "complete Code facts");
    facts
        .instructions
        .into_iter()
        .map(|instruction| instruction.bci)
        .collect()
}

fn assert_method_source_map(
    input: &Input,
    member: &ClassSourceMethod,
    name: &str,
    descriptor: &str,
) {
    let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
        panic!(
            "{}: {name}{descriptor} has a recovered body: {:?}",
            input.name, member.outcome
        );
    };
    let physical = physical_bcis(input.bytes, name, descriptor);
    assert!(
        !body.source_map.segments().is_empty(),
        "{}: source map exists for {name}{descriptor}",
        input.name
    );
    for segment in body.source_map.segments() {
        assert!(segment.start() <= segment.end());
        assert!(
            body.text.get(segment.start()..segment.end()).is_some(),
            "UTF-8 aligned span"
        );
        for anchor in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(anchor.method(), Some(&member.item.identity));
            assert!(
                physical.contains(&anchor.bci()),
                "BCI {} is a physical instruction in {name}{descriptor}",
                anchor.bci()
            );
        }
    }
}

fn assert_contract(input: &Input, report: &ClassSourceReport) {
    assert_eq!(
        report.methods.len(),
        4,
        "{}: full class retains constructor and all three declared methods",
        input.name
    );
    assert!(report.fields.is_empty());
    let expected = [
        (b"<init>".to_vec(), b"()V".to_vec()),
        (b"test".to_vec(), b"(Ljava/util/List;)V".to_vec()),
        (b"funcB".to_vec(), b"(Ljava/util/List;)Z".to_vec()),
        (b"funcC".to_vec(), b"()Z".to_vec()),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        method_keys(report),
        expected,
        "{}: no method was dropped",
        input.name
    );

    for (name, descriptor) in [
        ("<init>", "()V"),
        ("test", DESCRIPTOR),
        ("funcB", "(Ljava/util/List;)Z"),
        ("funcC", "()Z"),
    ] {
        let item = method(report, name, descriptor);
        assert_eq!(item.item.identity.owner, report.class);
        let ClassSourceOutcome::Recovered { report: body, .. } = &item.outcome else {
            panic!(
                "{}: full source for {name}{descriptor} remains available: {:?}",
                input.name, item.outcome
            );
        };
        assert_eq!(
            body.quality,
            Quality::Structured,
            "{}: {name}{descriptor}\n{}",
            input.name,
            body.text
        );
        assert!(!body.text.contains("@bytecode"));
        assert_method_source_map(input, item, name, descriptor);
    }

    let target = method(report, TARGET, DESCRIPTOR);
    let ClassSourceOutcome::Recovered { report: body, .. } = &target.outcome else {
        unreachable!()
    };
    assert!(
        body.text.contains("funcB("),
        "{}: protected call is retained\n{}",
        input.name,
        body.text
    );
    assert!(
        body.text.contains("break;"),
        "{}: protected normal exit remains a break\n{}",
        input.name,
        body.text
    );
    assert!(
        body.text.contains("throw "),
        "{}: terminal catch leaf rethrows the bound value\n{}",
        input.name,
        body.text
    );
    assert!(
        body.text.contains(".clear();"),
        "{}: retry effect remains\n{}",
        input.name,
        body.text
    );
    assert!(
        body.text.contains("Thread.sleep(100L);"),
        "{}: original retry delay remains\n{}",
        input.name,
        body.text
    );
    assert_eq!(body.text.matches("throw ").count(), 1);
    assert_eq!(body.text.matches(".clear();").count(), 1);
    assert_eq!(body.text.matches("Thread.sleep(100L);").count(), 1);
    let clear = body.text.find(".clear();").unwrap();
    let sleep = body.text.find("Thread.sleep(100L);").unwrap();
    assert!(
        clear < sleep,
        "{}: throw leaf and retry effects retain source order\n{}",
        input.name,
        body.text
    );

    // This original's implementations make funcB return false and funcC return true; therefore
    // the catch is not dynamically reached. The source assertions prove structure, not execution.
    let func_b = method(report, "funcB", "(Ljava/util/List;)Z");
    let func_c = method(report, "funcC", "()Z");
    let ClassSourceOutcome::Recovered { report: b, .. } = &func_b.outcome else {
        unreachable!()
    };
    let ClassSourceOutcome::Recovered { report: c, .. } = &func_c.outcome else {
        unreachable!()
    };
    assert!(b.text.contains("return false"));
    assert!(c.text.contains("return true"));
}

#[test]
fn original_loop2_debug_and_no_debug_default_and_all_keep_the_full_structured_retry_class() {
    for input in [
        Input {
            name: "debug class",
            bytes: DEBUG_CLASS,
            owner: OWNER,
        },
        Input {
            name: "no-debug class",
            bytes: NO_DEBUG_CLASS,
            owner: OWNER,
        },
    ] {
        let default = report(&input, false);
        let all = report(&input, true);
        assert_contract(&input, &default);
        assert_contract(&input, &all);
        assert_eq!(
            default.text, all.text,
            "{}: evidence selection does not change class source",
            input.name
        );
        for (name, descriptor) in [
            ("<init>", "()V"),
            ("test", DESCRIPTOR),
            ("funcB", "(Ljava/util/List;)Z"),
            ("funcC", "()Z"),
        ] {
            let (left, right) = (
                method(&default, name, descriptor),
                method(&all, name, descriptor),
            );
            assert_eq!(
                left.text, right.text,
                "{}: {name}{descriptor} source",
                input.name
            );
            assert_eq!(left.item.identity, right.item.identity);
            let (
                ClassSourceOutcome::Recovered { report: l, .. },
                ClassSourceOutcome::Recovered { report: r, .. },
            ) = (&left.outcome, &right.outcome)
            else {
                panic!("both reports keep structured methods")
            };
            assert_eq!(
                l.source_map, r.source_map,
                "{}: {name}{descriptor} source provenance",
                input.name
            );
        }
    }
}

#[test]
fn reachable_retry_controls_keep_both_polarities_and_full_methods_for_all_compiler_profiles() {
    const CONTROLS: [&[u8]; 4] = [
        include_bytes!(
            "fixtures/recover-branching-named-catch-retry-loops/control/javac8/debug/RetryCatchControl.class"
        ),
        include_bytes!(
            "fixtures/recover-branching-named-catch-retry-loops/control/javac8/no-debug/RetryCatchControl.class"
        ),
        include_bytes!(
            "fixtures/recover-branching-named-catch-retry-loops/control/javac23/debug/RetryCatchControl.class"
        ),
        include_bytes!(
            "fixtures/recover-branching-named-catch-retry-loops/control/javac23/no-debug/RetryCatchControl.class"
        ),
    ];
    for bytes in CONTROLS {
        let input = Input {
            name: "reachable control",
            bytes,
            owner: "recover/loop2/RetryCatchControl",
        };
        let default = report(&input, false);
        let all = report(&input, true);
        assert_eq!(default.text, all.text);
        assert_eq!(default.methods.len(), 9);
        assert_eq!(default.fields.len(), 7);
        for member in &default.methods {
            let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
                panic!("complete control method required")
            };
            assert_eq!(body.quality, Quality::Structured, "complete control method");
        }
        for name in ["run", "runInverted"] {
            let m = method(&default, name, "()V");
            let ClassSourceOutcome::Recovered { report: body, .. } = &m.outcome else {
                unreachable!()
            };
            assert_eq!(body.text.matches("throw ").count(), 1);
            assert_eq!(body.text.matches("retryEffect(").count(), 1);
            assert_method_source_map(&input, m, name, "()V");
        }
    }
}
