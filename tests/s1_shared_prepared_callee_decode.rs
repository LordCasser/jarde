//! S1 / 2.3：当前类任务内共享完整 Body 事实；本目标独占进程级解码计数。
#![cfg(feature = "test-support")]

use jarde::*;
use jarde_jvm::callee::{CalleeCandidate, CalleeReadRequest, read_prepared_callees};
use jarde_jvm::engine::{analyze_method_ir_owning_the_read, analyze_prepared_method_ir};
use jarde_reader::observation;
use jarde_reader::prepared::{MethodCodeAttribute, PreparedClass};
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;

const HOLDER: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Holder.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 8,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn environment(snapshot: &ArtifactSnapshot, prefix: Option<&[u8]>) -> ResolutionEnvironment {
    let root = match prefix {
        Some(prefix) => LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: snapshot.id().clone(),
                root_container: ContainerId("root".to_owned()),
                steps: Vec::new(),
            },
            prefix: ArchiveNameBytes(prefix.to_vec()),
        },
        None => LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        },
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![root],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: Vec::new(),
    }
}

struct Sample {
    snapshot: ArtifactSnapshot,
    definition: PhysicalDefinitionId,
    prefix: Option<Vec<u8>>,
}

impl Sample {
    fn standalone(bytes: &[u8]) -> Self {
        let engine = Engine::new();
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
            .expect("class bytes open");
        let inspected = engine
            .inspect_header(
                &snapshot,
                ClassTarget::Root,
                &mut budget,
                InspectionMode::Strict,
            )
            .expect("class header reads");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: inspected.source.class_bytes,
            variant: PhysicalVariant::Base,
        };
        Self {
            snapshot,
            definition,
            prefix: None,
        }
    }

    fn content(&self) -> &[ArtifactSnapshot] {
        slice::from_ref(&self.snapshot)
    }
    fn env(&self) -> ResolutionEnvironment {
        environment(&self.snapshot, self.prefix.as_deref())
    }
    fn method_request(&self) -> MethodAnalysisRequest {
        MethodAnalysisRequest {
            environment: self.env(),
            method: PhysicalMethodId {
                owner: self.definition.clone(),
                name: JvmBytes(b"value".to_vec()),
                descriptor: JvmBytes(b"()I".to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        }
    }
    fn candidate(call_site: u32) -> [CalleeCandidate; 1] {
        [CalleeCandidate::new(
            call_site,
            b"Holder".to_vec(),
            b"value".to_vec(),
            b"()I".to_vec(),
        )]
    }
    fn with_prepared<T>(
        &self,
        budget: &mut Budget,
        f: impl FnOnce(&PreparedClass<'_>, &mut Budget) -> T,
    ) -> T {
        let read = if let Some(entry) = self.definition.entry() {
            self.snapshot
                .prepared_class(entry, budget)
                .expect("entry read")
        } else {
            self.snapshot
                .prepared_root_class(budget)
                .expect("root read")
        };
        let prepared = PreparedClass::prepare(&read, budget).expect("class prepares");
        f(&prepared, budget)
    }
}

fn stored_zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}

fn archive_sample(snapshot: &ArtifactSnapshot, raw_name: &[u8]) -> Sample {
    let listing = Engine::new()
        .list_class_declarations(
            snapshot,
            &PhysicalScope::SnapshotAll,
            &mut Budget::new(limits()),
        )
        .expect("archive class declarations list");
    let definition = listing
        .items
        .iter()
        .find(|item| {
            item.definition
                .entry()
                .is_some_and(|entry| entry.raw_name.0 == raw_name)
        })
        .expect("requested class entry exists")
        .definition
        .clone();
    Sample {
        snapshot: snapshot.clone(),
        definition,
        prefix: Some(raw_name[..2].to_vec()),
    }
}

fn read_one(
    sample: &Sample,
    prepared: &PreparedClass<'_>,
    budget: &mut Budget,
    call_site: u32,
) -> CalleeReadReport {
    let candidates = Sample::candidate(call_site);
    read_prepared_callees(
        sample.content(),
        prepared,
        &CalleeReadRequest::new(&sample.env(), &sample.definition, &candidates),
        budget,
    )
    .expect("callee is read")
}

#[test]
fn callee_code_quota_preserves_the_first_error_and_stops_before_the_next_body() {
    let sample = Sample::standalone(HOLDER);
    let mut bounds = limits();
    bounds.code_bytes = 0;
    let mut budget = Budget::new(bounds);
    let before = observation::snapshot().body_decode_attempts;
    sample.with_prepared(&mut budget, |prepared, budget| {
        let candidates = [
            CalleeCandidate::new(2, b"Holder".to_vec(), b"value".to_vec(), b"()I".to_vec()),
            CalleeCandidate::new(4, b"Holder".to_vec(), b"<init>".to_vec(), b"(I)V".to_vec()),
        ];
        let error = read_prepared_callees(
            sample.content(),
            prepared,
            &CalleeReadRequest::new(&sample.env(), &sample.definition, &candidates),
            budget,
        )
        .expect_err("a body quota failure cannot become a partial callee or a later cancellation");
        assert!(
            matches!(
                error,
                Error::BudgetExceeded {
                    dimension: BudgetDimension::CodeBytes,
                    limit: 0,
                    consumed: 0,
                    requested: 1
                }
            ),
            "{error:?}"
        );
        assert_eq!(budget.usage().method_bodies, 1);
    });
    assert_eq!(observation::snapshot().body_decode_attempts - before, 1);
}

#[test]
fn live_prepared_callee_facts_are_shared_only_when_complete_and_keep_polling() {
    let body_before = observation::snapshot().body_decode_attempts;

    // Cache none/cold/hot/too-small: live PreparedClass facts alone must make the second
    // callee read free of another Code decode. A cache hit/refusal cannot define that lifetime.
    for cache_case in ["none", "cold", "hot", "tiny"] {
        let sample = Sample::standalone(HOLDER);
        let cache = (cache_case != "none").then(|| match cache_case {
            "tiny" => FactsCache::current(FactsCapacity::new(1, 1)),
            _ => FactsCache::current(FactsCapacity::new(16, 1 << 20)),
        });
        let mut budget = cache.clone().map_or_else(
            || {
                Budget::new(Limits {
                    method_bodies: 1,
                    ..limits()
                })
            },
            |cache| {
                Budget::new(Limits {
                    method_bodies: 1,
                    ..limits()
                })
                .with_facts_cache(cache)
            },
        );
        if cache_case == "hot" {
            class_facts(HOLDER, &mut budget).expect("warm the facts cache");
        }
        let before = observation::snapshot().body_decode_attempts;
        sample.with_prepared(&mut budget, |prepared, budget| {
            let first = read_one(&sample, prepared, budget, 2);
            let second = read_one(&sample, prepared, budget, 19);
            assert_eq!(first.members().len(), 1);
            assert_eq!(second.members().len(), 1);
            assert_eq!(first.reads()[0].reason, ReadReason::CalleeMemberBody);
            assert_eq!(second.reads()[0].reason, ReadReason::CalleeMemberBody);
            assert!(std::ptr::eq(
                first.members()[0].body().unwrap().facts(),
                second.members()[0].body().unwrap().facts()
            ));
            assert_eq!(budget.usage().method_bodies, 1);
            let original_execution = first.members()[0].body().unwrap().facts().execution.clone();
            let ordinal = prepared.locate_method(b"value", b"()I")[0];
            let mut reuse_only = Budget::new(Limits {
                method_bodies: 0,
                code_bytes: 0,
                ..limits()
            });
            let shared = prepared
                .shared_method_code(ordinal, &mut reuse_only)
                .unwrap();
            assert_eq!(shared.execution, original_execution);
            assert_eq!(reuse_only.usage().method_bodies, 0);
            assert_eq!(reuse_only.usage().code_bytes, 0);
        });
        assert_eq!(
            observation::snapshot().body_decode_attempts - before,
            1,
            "cache={cache_case}"
        );
    }

    // A prepared driver analysis hands its completed facts to the same-class callee read.
    {
        let sample = Sample::standalone(HOLDER);
        let mut budget = Budget::new(limits());
        let before = observation::snapshot().body_decode_attempts;
        sample.with_prepared(&mut budget, |prepared, budget| {
            let analysis = analyze_prepared_method_ir(
                sample.content(),
                prepared,
                &sample.method_request(),
                budget,
            )
            .expect("prepared driver analysis");
            assert!(analysis.ir().code().is_some());
            let candidates = Sample::candidate(4);
            let report = read_prepared_callees(
                sample.content(),
                prepared,
                &CalleeReadRequest::new(&sample.env(), &sample.definition, &candidates)
                    .with_analyzed_method(analysis.ir()),
                budget,
            )
            .expect("callee reuses prepared driver facts");
            assert!(std::ptr::eq(
                analysis.ir().code().unwrap(),
                report.members()[0].body().unwrap().facts()
            ));
        });
        assert_eq!(observation::snapshot().body_decode_attempts - before, 1);
    }

    // The ordinary owning read follows the same handoff once its trusted read is prepared.
    {
        let sample = Sample::standalone(HOLDER);
        let mut budget = Budget::new(limits());
        let before = observation::snapshot().body_decode_attempts;
        let analyzed = analyze_method_ir_owning_the_read(
            sample.content(),
            &sample.method_request(),
            &mut budget,
        )
        .expect("ordinary owning driver analysis");
        let (analysis, read) = analyzed.into_parts();
        let read = read.expect("driver owns the class read");
        let prepared = PreparedClass::prepare(&read, &mut budget).expect("owned read prepares");
        let candidates = Sample::candidate(6);
        let report = read_prepared_callees(
            sample.content(),
            &prepared,
            &CalleeReadRequest::new(&sample.env(), &sample.definition, &candidates)
                .with_analyzed_method(analysis.ir()),
            &mut budget,
        )
        .expect("callee reuses owning driver facts");
        assert!(std::ptr::eq(
            analysis.ir().code().unwrap(),
            report.members()[0].body().unwrap().facts()
        ));
        assert_eq!(observation::snapshot().body_decode_attempts - before, 1);
    }

    // Equal class bytes at distinct archive coordinates remain distinct physical facts and decode
    // once per PreparedClass. The live memo is owned by each prepared physical read.
    {
        let archive = stored_zip(&[(b"a/Holder.class", HOLDER), (b"b/Holder.class", HOLDER)]);
        let mut open_budget = Budget::new(limits());
        let snapshot =
            ArtifactSnapshot::open(ArtifactInput::bytes(archive), &mut open_budget).unwrap();
        let a = archive_sample(&snapshot, b"a/Holder.class");
        let b = archive_sample(&snapshot, b"b/Holder.class");
        assert_ne!(a.definition, b.definition);
        assert_eq!(a.definition.class_bytes, b.definition.class_bytes);
        let before = observation::snapshot().body_decode_attempts;
        for sample in [&a, &b] {
            let mut budget = Budget::new(limits());
            sample.with_prepared(&mut budget, |prepared, budget| {
                let one = read_one(sample, prepared, budget, 2);
                let two = read_one(sample, prepared, budget, 19);
                assert_eq!(one.members().len(), 1);
                assert_eq!(two.members().len(), 1);
            });
        }
        assert_eq!(observation::snapshot().body_decode_attempts - before, 2);
    }

    // A cancelled request cannot take a memo hit without polling first.
    {
        let sample = Sample::standalone(HOLDER);
        let token = CancellationToken::new();
        let mut budget = Budget::with_cancellation_token(limits(), token.clone());
        let before = observation::snapshot().body_decode_attempts;
        sample.with_prepared(&mut budget, |prepared, budget| {
            let report = read_one(&sample, prepared, budget, 2);
            assert_eq!(report.members().len(), 1);
            let ordinal = prepared.locate_method(b"value", b"()I")[0];
            token.cancel();
            assert!(matches!(
                prepared.shared_method_code(ordinal, budget),
                Err(Error::Cancelled { .. })
            ));
        });
        assert_eq!(observation::snapshot().body_decode_attempts - before, 1);
    }

    // A damaged decode is returned but never memoized: the same ordinal enters the decoder twice.
    {
        let sample = Sample::standalone(HOLDER);
        let mut changed = HOLDER.to_vec();
        let mut budget = Budget::new(limits());
        sample.with_prepared(&mut budget, |prepared, _| {
            let ordinal = prepared.locate_method(b"value", b"()I")[0];
            let slot = prepared.slot(ordinal).unwrap();
            let shell = match &slot.code {
                MethodCodeAttribute::Single(shell) => shell,
                other => panic!("Holder.value has one Code shell: {other:?}"),
            };
            let opcode = usize::try_from(shell.content_span.start).unwrap() + 8;
            changed[opcode] = 0xff;
        });
        let damaged = Sample::standalone(&changed);
        let before = observation::snapshot().body_decode_attempts;
        let mut budget = Budget::new(limits());
        damaged.with_prepared(&mut budget, |prepared, budget| {
            let ordinal = prepared.locate_method(b"value", b"()I")[0];
            for _ in 0..2 {
                if let Ok(facts) = prepared.shared_method_code(ordinal, budget) {
                    assert!(facts.stopped_at.is_some());
                    assert!(!matches!(facts.execution, ExecutionReport::Complete { .. }));
                }
            }
            assert_eq!(budget.usage().method_bodies, 2);
        });
        assert_eq!(observation::snapshot().body_decode_attempts - before, 2);
    }

    // Empty candidate demand is header-only and never enters a Code decoder.
    {
        let sample = Sample::standalone(HOLDER);
        let mut budget = Budget::new(Limits {
            method_bodies: 0,
            ..limits()
        });
        let before = observation::snapshot().body_decode_attempts;
        sample.with_prepared(&mut budget, |prepared, budget| {
            let candidates: [CalleeCandidate; 0] = [];
            let report = read_prepared_callees(
                sample.content(),
                prepared,
                &CalleeReadRequest::new(&sample.env(), &sample.definition, &candidates),
                budget,
            )
            .expect("header-only request");
            assert!(report.members().is_empty());
        });
        assert_eq!(observation::snapshot().body_decode_attempts - before, 0);
        assert_eq!(budget.usage().method_bodies, 0);
    }

    // 普通高层恢复没有辅助 Body；Header/类型证明不会引入额外解码。
    {
        let sample = Sample::standalone(HOLDER);
        let mut budget = Budget::new(Limits {
            method_bodies: 1,
            ..limits()
        });
        let before = observation::snapshot().body_decode_attempts;
        let recovered = Engine::new()
            .recover_method(
                sample.content(),
                jarde::SourceTarget::JAVA_8,
                &sample.method_request(),
                &mut budget,
            )
            .unwrap();
        assert!(recovered.callees().is_none());
        assert_eq!(observation::snapshot().body_decode_attempts - before, 1);
    }

    // 解码前的预算失败也不记成负结果；后续合法需求仍可完成。
    {
        let sample = Sample::standalone(HOLDER);
        let mut budget = Budget::new(limits());
        sample.with_prepared(&mut budget, |prepared, _| {
            let ordinal = prepared.locate_method(b"value", b"()I")[0];
            let mut stopped = Budget::new(Limits {
                method_bodies: 0,
                ..limits()
            });
            assert!(matches!(
                prepared.shared_method_code(ordinal, &mut stopped),
                Err(Error::BudgetExceeded {
                    dimension: BudgetDimension::MethodBodies,
                    ..
                })
            ));
            let mut retry = Budget::new(Limits {
                method_bodies: 1,
                ..limits()
            });
            let facts = prepared.shared_method_code(ordinal, &mut retry).unwrap();
            assert!(matches!(facts.execution, ExecutionReport::Complete { .. }));
        });
    }

    // memo 的拥有者是当前类任务；任务及最后一个消费者释放后，Body 事实也释放。
    {
        let sample = Sample::standalone(HOLDER);
        let mut budget = Budget::new(limits());
        let weak = sample.with_prepared(&mut budget, |prepared, budget| {
            let ordinal = prepared.locate_method(b"value", b"()I")[0];
            let code = prepared.shared_method_code(ordinal, budget).unwrap();
            std::sync::Arc::downgrade(&code)
        });
        assert!(weak.upgrade().is_none());
    }

    assert_eq!(
        observation::snapshot().body_decode_attempts - body_before,
        14
    );
}

fn renamed_holder(name: &[u8]) -> Vec<u8> {
    let needle = b"\x01\x00\x06Holder";
    let positions: Vec<_> = HOLDER
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == needle).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    let at = positions[0];
    let mut bytes = HOLDER[..at + 1].to_vec();
    bytes.extend_from_slice(&u16::try_from(name.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(&HOLDER[at + needle.len()..]);
    bytes
}

fn check_callee_owner_bytes(owner: JvmBytes, other: JvmBytes) {
    assert_ne!(owner, other);
    assert_eq!(
        String::from_utf8_lossy(&owner.0),
        String::from_utf8_lossy(&other.0)
    );
    let sample = Sample::standalone(&renamed_holder(&owner.0));
    for prepared in [false, true] {
        for (candidate_owner, honest) in [(&owner, true), (&other, false)] {
            let candidates = [CalleeCandidate::new(
                0,
                candidate_owner.0.clone(),
                b"value".to_vec(),
                b"()I".to_vec(),
            )];
            let environment = sample.env();
            let request = CalleeReadRequest::new(&environment, &sample.definition, &candidates);
            let mut budget = Budget::new(limits());
            let report = if prepared {
                sample.with_prepared(&mut budget, |class, budget| {
                    read_prepared_callees(sample.content(), class, &request, budget)
                })
            } else {
                jarde_jvm::callee::read_callees(sample.content(), &request, &mut budget)
            }
            .expect("the actual physical class binds under its raw standalone name");
            assert_eq!(report.raw_class(), &owner);
            assert_eq!(report.clone().without_read_details().raw_class(), &owner);
            if honest {
                assert_eq!(report.members().len(), 1);
                assert!(report.members()[0].body().is_some());
                assert_eq!(budget.usage().method_bodies, 1);
            } else {
                assert!(
                    report.members().is_empty(),
                    "distinct raw owners must not read a member through their common lossy text; prepared={prepared}"
                );
                assert_eq!(report.refusals()[0].code(), "callee_not_this_class");
                assert_eq!(budget.usage().method_bodies, 0);
            }
        }
    }
}

#[test]
fn callee_owner_raw_supplementary_names_do_not_merge() {
    check_callee_owner_bytes(JvmBytes::from_unicode("𐐀"), JvmBytes::from_unicode("𐐁"));
}

#[test]
fn callee_owner_raw_unpaired_names_do_not_merge() {
    check_callee_owner_bytes(
        JvmBytes(vec![0xed, 0xa0, 0x81]),
        JvmBytes(vec![0xed, 0xa0, 0x82]),
    );
}
