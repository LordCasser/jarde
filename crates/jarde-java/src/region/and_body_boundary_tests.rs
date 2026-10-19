use super::*;

use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/cf09-complex-if-body-boundary/debug/class/jadx/tests/integration/loops/TestBreakInComplexIf$TestCls.class"
);
const NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/cf09-complex-if-body-boundary/no-debug/class/jadx/tests/integration/loops/TestBreakInComplexIf$TestCls.class"
);
const DESCRIPTOR: &[u8] = b"(Ljava/util/Map;I)I";
const CONTROLS: &[u8] = include_bytes!(
    "../../../../tests/fixtures/cf09-complex-if-body-boundary/controls/HeaderBodyBoundaryControls.class"
);
const CONTROL_DESCRIPTOR: &[u8] = b"(Ljava/util/Map;II)I";

fn generous_limits() -> Limits {
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
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn with_real_loop<R>(
    bytes: &[u8],
    budget: &mut Budget,
    check: impl FnOnce(
        &mut Walker<'_>,
        &CanonicalBlockId,
        usize,
        &BTreeSet<usize>,
        &CanonicalBlockId,
        &CanonicalBlockId,
    ) -> R,
) -> R {
    with_loop_method(bytes, b"test", DESCRIPTOR, budget, check)
}

fn with_loop_method<R>(
    bytes: &[u8],
    method_name: &[u8],
    descriptor: &[u8],
    budget: &mut Budget,
    check: impl FnOnce(
        &mut Walker<'_>,
        &CanonicalBlockId,
        usize,
        &BTreeSet<usize>,
        &CanonicalBlockId,
        &CanonicalBlockId,
    ) -> R,
) -> R {
    let mut read_budget = Budget::new(generous_limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut read_budget)
        .expect("the frozen complete TestCls opens");
    let owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("fixture length fits"),
        },
        variant: PhysicalVariant::Base,
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let analysis = analyze_method_ir(
        &[snapshot.clone()],
        &MethodAnalysisRequest {
            environment: ResolutionEnvironment {
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
            },
            method: PhysicalMethodId {
                owner,
                name: JvmBytes(method_name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut read_budget,
    )
    .expect("the pinned same-run method facts analyze");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("canonical CFG is present");
    let ssa = ir.ssa().expect("same-run SSA is present");
    let code = ir.code().expect("complete Code facts are present");
    let pool = ir.constant_pool();
    let operations = Operations::of(code, pool);
    let mut flow_budget = Budget::new(generous_limits());
    let view = NormalFlowView::build(canonical, &mut flow_budget)
        .expect("the complete normal-flow projection builds");
    let (header_node, natural) = (0..view.len())
        .find_map(|node| {
            view.loop_entered_at(node)
                .filter(|natural| natural.header() == node && !natural.is_irreducible())
                .map(|natural| (node, natural))
        })
        .expect("the frozen method has its natural loop");
    let header = view
        .id_of(header_node)
        .expect("natural header identity exists")
        .clone();
    let blocks = natural.blocks().clone();
    let exit = view
        .successor_ids(&header)
        .into_iter()
        .find(|successor| {
            view.index_of(successor)
                .is_some_and(|node| !blocks.contains(&node))
        })
        .or_else(|| {
            view.immediate_post_dominator(header_node)
                .and_then(|node| view.id_of(node))
                .filter(|id| {
                    view.index_of(id)
                        .is_some_and(|node| !blocks.contains(&node))
                })
                .cloned()
        })
        .expect("the complete loop has an external exit or post-dominator");
    let later_candidate = view
        .successor_ids(&header)
        .into_iter()
        .find(|successor| {
            view.index_of(successor)
                .is_some_and(|node| blocks.contains(&node) && node != header_node)
        })
        .expect("header has a structurally eligible later AND candidate");
    let chains = crate::concat::plan(ssa, &operations, &crate::build::FieldCopies::default());
    let sites = crate::init::Sites::empty();
    let excluded_edge_nodes = canonical
        .edges()
        .iter()
        .filter(|edge| {
            matches!(
                edge.kind(),
                CanonicalEdgeKind::Exception { .. } | CanonicalEdgeKind::Call { .. }
            )
        })
        .flat_map(|edge| [edge.from().clone(), edge.to().clone()])
        .collect();
    let mut walker = Walker {
        canonical,
        view: &view,
        ssa,
        operations: &operations,
        pool,
        chains: &chains,
        sites: &sites,
        code,
        method_synchronized: Some(false),
        has_reachable_explicit_monitor: false,
        return_is_boolean: false,
        handlers: &code.exception_handlers,
        profile: &crate::pass::JAVA_8,
        budget,
        catch_joins: BTreeSet::new(),
        loop_catch_terminal_owners: Vec::new(),
        fragmented: None,
        excluded_edge_nodes,
        visited: BTreeSet::new(),
        unclosed_tail_at: None,
        own_monitor: None,
        depth: 0,
    };
    check(
        &mut walker,
        &header,
        header_node,
        &blocks,
        &later_candidate,
        &exit,
    )
}

fn contains_if_at(region: &Region, bci: u32) -> bool {
    match region {
        Region::Sequence { regions } => regions.iter().any(|region| contains_if_at(region, bci)),
        Region::If {
            branch_bci,
            then_arm,
            else_arm,
            ..
        } => *branch_bci == bci || contains_if_at(then_arm, bci) || contains_if_at(else_arm, bci),
        Region::CompoundIf {
            tests,
            then_arm,
            else_arm,
            ..
        } => {
            tests.iter().any(|test| test.branch_bci == bci)
                || contains_if_at(then_arm, bci)
                || contains_if_at(else_arm, bci)
        }
        Region::Loop { body, .. } => body.iter().any(|region| contains_if_at(region, bci)),
        _ => false,
    }
}

fn contains_loop_break_for(region: &Region, header: &CanonicalBlockId) -> bool {
    match region {
        Region::Sequence { regions } => regions
            .iter()
            .any(|region| contains_loop_break_for(region, header)),
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => contains_loop_break_for(then_arm, header) || contains_loop_break_for(else_arm, header),
        Region::Loop { body, .. } => body
            .iter()
            .any(|region| contains_loop_break_for(region, header)),
        Region::LoopBreak { loop_header, .. } => loop_header == header,
        _ => false,
    }
}

fn contains_fallback(region: &Region) -> bool {
    match region {
        Region::Fallback { .. } => true,
        Region::Sequence { regions } => regions.iter().any(contains_fallback),
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => contains_fallback(then_arm) || contains_fallback(else_arm),
        Region::Loop { body, .. } => body.iter().any(contains_fallback),
        _ => false,
    }
}

#[test]
fn one_pure_header_keeps_the_existing_loop_route_and_owns_effectful_body() {
    for (label, bytes) in [("debug", DEBUG), ("no-debug", NO_DEBUG)] {
        let mut budget = Budget::new(generous_limits());
        with_real_loop(
            bytes,
            &mut budget,
            |walker, header, header_node, blocks, candidate, exit| {
                assert_eq!(header.bci(), 7, "{label}: pinned natural header");
                assert_eq!(candidate.bci(), 14, "{label}: later candidate");
                assert_eq!(exit.bci(), 72, "{label}: shared exit");
                let candidate_node = walker.view.index_of(candidate).expect("candidate node");
                let reason = match walker.proved_header_test(header, candidate, candidate_node) {
                    Ok(Err(
                        reason @ FallbackReason::UnmetPrecondition {
                            pass,
                            requirement: crate::pass::Precondition::StatementFree,
                            block_bci,
                            ..
                        },
                    )) if *pass == crate::pass::LOOP && block_bci == candidate.bci() => reason,
                    other => {
                        panic!("{label}: expected the later StatementFree refusal, got {other:?}")
                    }
                };
                assert!(
                    walker
                        .later_and_body_boundary(
                            header_node,
                            candidate,
                            candidate_node,
                            exit,
                            &reason
                        )
                        .expect("the exact local successor check completes")
                );

                assert!(
                    matches!(
                        walker.header_test_chain(header, header_node, blocks),
                        Ok(Ok(None))
                    ),
                    "one pure test remains on the existing single-header route"
                );
                let (regions, recovered_exit) = walker
                    .header_tested_loop(header, header_node, blocks, &Frame::default())
                    .expect("the existing loop route completes")
                    .expect("the original loop remains structured");
                assert_eq!(recovered_exit.as_ref(), Some(exit));
                assert!(regions.iter().all(|region| !contains_fallback(region)));
                let [
                    Region::Loop {
                        tests,
                        body,
                        exit: loop_exit,
                        ..
                    },
                ] = regions.as_slice()
                else {
                    panic!("{label}: expected one ordinary header-tested loop, got {regions:?}");
                };
                assert_eq!(tests.len(), 1, "only the pure header remains a test");
                assert_eq!(tests[0].0, *header);
                assert_eq!(loop_exit.as_ref(), Some(exit));
                let owned = regions.iter().flat_map(Region::blocks).collect::<Vec<_>>();
                assert!(owned.contains(&candidate), "body entry is owned");
                assert_eq!(owned.len(), owned.iter().collect::<BTreeSet<_>>().len());
                assert!(
                    body.iter().any(|region| contains_if_at(region, 47)),
                    "null guard remains an If"
                );
                assert!(
                    body.iter()
                        .any(|region| contains_loop_break_for(region, header)),
                    "break remains owned by this loop"
                );
            },
        );
    }
}

#[test]
fn a_longer_pure_and_prefix_stops_before_the_effectful_body() {
    let mut budget = Budget::new(generous_limits());
    with_loop_method(
        CONTROLS,
        b"andPrefix",
        CONTROL_DESCRIPTOR,
        &mut budget,
        |walker, header, header_node, blocks, _, exit| {
            let chain = match walker.header_test_chain(header, header_node, blocks) {
                Ok(Ok(Some(chain))) => chain,
                _ => panic!("the two-predicate AND prefix must be retained"),
            };
            assert_eq!(chain.operator, crate::ast::BinaryOp::LogicalAnd);
            assert!(chain.tests.len() >= 2, "both pure header predicates remain");
            assert_eq!(chain.exit, *exit);
            let body_node = walker.view.index_of(&chain.body).expect("body entry node");
            let body_test = match walker.proved_header_test(header, &chain.body, body_node) {
                Ok(Err(
                    reason @ FallbackReason::UnmetPrecondition {
                        pass,
                        requirement: crate::pass::Precondition::StatementFree,
                        block_bci,
                        ..
                    },
                )) if *pass == crate::pass::LOOP && block_bci == chain.body.bci() => reason,
                other => panic!("the body candidate stops only at StatementFree: {other:?}"),
            };
            assert!(
                walker
                    .later_and_body_boundary(header_node, &chain.body, body_node, exit, &body_test)
                    .expect("the effectful body boundary remains exact")
            );
            let (regions, _) = walker
                .header_tested_loop(header, header_node, blocks, &Frame::default())
                .expect("complete loop traversal finishes")
                .expect("the complete loop is structurally recoverable");
            let [Region::Loop { tests, body, .. }] = regions.as_slice() else {
                panic!("expected a recovered loop, got {regions:?}");
            };
            assert_eq!(tests.len(), chain.tests.len());
            let owned = regions.iter().flat_map(Region::blocks).collect::<Vec<_>>();
            assert!(owned.contains(&&chain.body));
            assert_eq!(owned.len(), owned.iter().collect::<BTreeSet<_>>().len());
            assert!(
                !body
                    .iter()
                    .any(|region| matches!(region, Region::Fallback { .. }))
            );
        },
    );
}

#[test]
fn the_existing_or_route_and_effectful_initial_tests_remain_unchanged() {
    let mut budget = Budget::new(generous_limits());
    with_loop_method(
        CONTROLS,
        b"orBody",
        CONTROL_DESCRIPTOR,
        &mut budget,
        |walker, header, header_node, blocks, _, _| {
            match walker.header_test_chain(header, header_node, blocks) {
                Ok(Ok(Some(chain))) => assert_eq!(chain.operator, crate::ast::BinaryOp::LogicalOr),
                Ok(Ok(None)) | Ok(Err(FallbackReason::LoopShape { .. })) => {}
                _ => panic!("the OR route must keep its prior proof decision"),
            }
            let (regions, _) = walker
                .loop_region(header, header_node, &Frame::default())
                .expect("the existing complete OR loop route finishes");
            assert!(regions.iter().all(|region| !contains_fallback(region)));
        },
    );

    for method in [b"initialEffect".as_slice(), b"initialStatements".as_slice()] {
        let mut budget = Budget::new(generous_limits());
        with_loop_method(
            CONTROLS,
            method,
            CONTROL_DESCRIPTOR,
            &mut budget,
            |walker, header, header_node, blocks, _, _| {
                let result = walker.header_test_chain(header, header_node, blocks);
                match result {
                    Ok(Err(FallbackReason::UnmetPrecondition {
                        pass,
                        requirement: crate::pass::Precondition::StatementFree,
                        block_bci,
                        ..
                    })) => {
                        assert_eq!(*pass, crate::pass::LOOP);
                        assert_eq!(block_bci, header.bci());
                    }
                    Ok(Ok(None)) => {}
                    Ok(Ok(Some(_))) => panic!(
                        "initial effect unexpectedly claimed a header chain: {}",
                        String::from_utf8_lossy(method)
                    ),
                    Ok(Err(reason)) => panic!(
                        "unexpected initial-header refusal for {}: {reason:?}",
                        String::from_utf8_lossy(method)
                    ),
                    Err(stop) => panic!(
                        "unexpected initial-header Stop for {}: {stop:?}",
                        String::from_utf8_lossy(method)
                    ),
                }
                assert!(walker.visited.is_empty());
                let (regions, _) = walker
                    .loop_region(header, header_node, &Frame::default())
                    .expect("the complete existing loop path finishes");
                assert!(
                    regions.iter().any(|region| matches!(
                        region,
                        Region::Fallback {
                            reason: FallbackReason::UnmetPrecondition {
                                requirement: crate::pass::Precondition::StatementFree,
                                block_bci,
                                ..
                            },
                            ..
                        } if *block_bci == header.bci()
                    )),
                    "the complete loop preserves its observed initial-header refusal: {} {regions:?}",
                    String::from_utf8_lossy(method)
                );
            },
        );
    }
}

#[test]
fn an_extra_real_bytecode_entry_does_not_become_a_body_boundary() {
    let mut proof_budget = Budget::new(generous_limits());
    let (changed, header_bci, candidate_bci) = with_real_loop(
        DEBUG,
        &mut proof_budget,
        |walker, header, _header_node, _blocks, candidate, exit| {
            let exit_edge = walker
                .canonical
                .edges()
                .iter()
                .find(|edge| {
                    edge.kind() == CanonicalEdgeKind::Normal
                        && edge.to() == exit
                        && edge.from() != header
                        && walker
                            .terminal_bci(edge.from())
                            .and_then(|bci| walker.code.instructions.iter().find(|i| i.bci == bci))
                            .is_some_and(|instruction| matches!(instruction.opcode, 0xa7 | 0xc8))
                })
                .expect("the actual loop has a non-header normal exit edge");
            let source_bci = walker
                .terminal_bci(exit_edge.from())
                .expect("exit edge source has a physical terminal");
            let instruction = walker
                .code
                .instructions
                .iter()
                .find(|instruction| instruction.bci == source_bci)
                .expect("edge source instruction is present");
            assert!(
                matches!(instruction.opcode, 0xa7 | 0xc8),
                "the selected exit is a goto"
            );
            let operand = walker
                .code
                .instructions
                .iter()
                .zip(walker.code.operands())
                .find(|(instruction, _)| instruction.bci == source_bci)
                .map(|(_, operand)| operand)
                .expect("goto operands are present");
            assert_eq!(
                i64::from(source_bci) + i64::from(operand.branch_offset.expect("goto target")),
                i64::from(exit.bci()),
                "the source goto originally reaches this loop exit"
            );
            let relative = i64::from(candidate.bci()) - i64::from(source_bci);
            let code_start =
                usize::try_from(walker.code.code_span.start).expect("class offset fits");
            let operand_start = code_start + usize::try_from(source_bci).expect("BCI fits") + 1;
            let mut changed = DEBUG.to_vec();
            match instruction.opcode {
                0xa7 => {
                    let relative = i16::try_from(relative).expect("short branch displacement fits");
                    changed[operand_start..operand_start + 2]
                        .copy_from_slice(&relative.to_be_bytes());
                }
                0xc8 => {
                    let relative = i32::try_from(relative).expect("wide branch displacement fits");
                    changed[operand_start..operand_start + 4]
                        .copy_from_slice(&relative.to_be_bytes());
                }
                _ => unreachable!(),
            }
            (changed, header.bci(), candidate.bci())
        },
    );

    // Parser/IR-only mutation: this adds a normal predecessor to the body entry while retaining
    // the same physical code and a decodable target. The unchanged StackMapTable is not a JVM claim.
    let mut budget = Budget::new(generous_limits());
    with_real_loop(
        &changed,
        &mut budget,
        |walker, header, header_node, blocks, _, _| {
            assert_eq!(
                header.bci(),
                header_bci,
                "the same outer natural loop remains selected"
            );
            let candidate_node = (0..walker.view.len())
                .find(|node| {
                    walker
                        .view
                        .id_of(*node)
                        .is_some_and(|id| id.bci() == candidate_bci)
                })
                .expect("mutated CFG retains the original body entry");
            assert!(
                walker.view.predecessors(candidate_node).len() > 1,
                "the goto mutation adds an entry"
            );
            assert!(!walker.header_test_candidate(header_node, candidate_node, blocks));
            let result = walker.header_test_chain(header, header_node, blocks);
            match result {
                Ok(Ok(Some(chain))) => assert_ne!(chain.body.bci(), candidate_bci),
                Ok(Ok(None)) | Ok(Err(_)) => {}
                Err(stop) => panic!("ample budget does not stop the extra-entry refusal: {stop:?}"),
            }
            assert!(
                walker.visited.is_empty(),
                "the refusal does not publish partial ownership"
            );
        },
    );
}

#[test]
fn only_the_exact_later_loop_statement_free_reason_can_select_the_body_boundary() {
    let mut budget = Budget::new(generous_limits());
    with_real_loop(
        DEBUG,
        &mut budget,
        |walker, header, header_node, _, candidate, exit| {
            let candidate_node = walker.view.index_of(candidate).expect("candidate node");
            let statement_free = FallbackReason::unmet(
                &crate::pass::LOOP,
                crate::pass::Precondition::StatementFree,
                candidate.bci(),
                candidate.bci(),
            );
            assert!(
                walker
                    .later_and_body_boundary(
                        header_node,
                        candidate,
                        candidate_node,
                        exit,
                        &statement_free
                    )
                    .expect("matching evidence completes")
            );
            assert!(
                !walker
                    .later_and_body_boundary(
                        header_node,
                        candidate,
                        candidate_node,
                        header,
                        &statement_free
                    )
                    .expect("a mismatched common exit is a local refusal")
            );
            let wrong_block = FallbackReason::unmet(
                &crate::pass::LOOP,
                crate::pass::Precondition::StatementFree,
                candidate.bci().saturating_add(1),
                candidate.bci(),
            );
            assert!(
                !walker
                    .later_and_body_boundary(
                        header_node,
                        candidate,
                        candidate_node,
                        exit,
                        &wrong_block
                    )
                    .expect("a different refusal owner is a local refusal")
            );
            assert!(
                !walker
                    .later_and_body_boundary(
                        header_node,
                        candidate,
                        candidate_node,
                        exit,
                        &FallbackReason::LoopShape {
                            block_bci: candidate.bci()
                        },
                    )
                    .expect("a topology failure is never a body boundary")
            );
            assert!(
                !walker
                    .later_and_body_boundary(
                        header_node,
                        candidate,
                        candidate_node,
                        exit,
                        &FallbackReason::ExceptionEdge {
                            block_bci: candidate.bci(),
                            handler_ordinal: 0,
                        },
                    )
                    .expect("an exception-edge failure is never a body boundary")
            );
            let initial_header_refusal = FallbackReason::unmet(
                &crate::pass::LOOP,
                crate::pass::Precondition::StatementFree,
                header.bci(),
                header.bci(),
            );
            assert!(
                !walker
                    .later_and_body_boundary(
                        header_node,
                        header,
                        header_node,
                        exit,
                        &initial_header_refusal
                    )
                    .expect("initial-header effects retain the old path")
            );
        },
    );
}

#[test]
fn stop_at_later_candidate_does_not_publish_partial_loop_ownership() {
    let mut limits = generous_limits();
    limits.analysis_steps = 1;
    let mut budget = Budget::new(limits);
    with_real_loop(
        DEBUG,
        &mut budget,
        |walker, header, header_node, blocks, _, _| {
            let result = walker.header_test_chain(header, header_node, blocks);
            assert!(
                matches!(
                    &result,
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        ..
                    })
                ),
                "the later candidate remains a stop when its proof budget is exhausted"
            );
            assert!(
                walker.visited.is_empty(),
                "chain proof owns no blocks before commit"
            );
        },
    );
}

#[test]
fn budget_stop_at_body_boundary_does_not_publish_partial_ownership() {
    let mut measured_budget = Budget::new(generous_limits());
    let proved_steps = with_real_loop(
        DEBUG,
        &mut measured_budget,
        |walker, header, _, _, candidate, _| {
            let candidate_node = walker.view.index_of(candidate).expect("candidate node");
            let before = walker.budget.usage().analysis_steps;
            let result = walker.proved_header_test(header, candidate, candidate_node);
            let after = walker.budget.usage().analysis_steps;
            assert!(matches!(
                result,
                Ok(Err(FallbackReason::UnmetPrecondition {
                    requirement: crate::pass::Precondition::StatementFree,
                    ..
                }))
            ));
            after - before
        },
    );
    assert!(proved_steps > 0);

    let mut limits = generous_limits();
    limits.analysis_steps = proved_steps;
    let mut budget = Budget::new(limits);
    with_real_loop(
        DEBUG,
        &mut budget,
        |walker, header, header_node, _, candidate, exit| {
            let candidate_node = walker.view.index_of(candidate).expect("candidate node");
            let reason = match walker.proved_header_test(header, candidate, candidate_node) {
                Ok(Err(reason)) => reason,
                other => panic!("the candidate proof has the measured refusal: {other:?}"),
            };
            assert!(matches!(
                walker.later_and_body_boundary(
                    header_node,
                    candidate,
                    candidate_node,
                    exit,
                    &reason
                ),
                Err(StopReason::Budget {
                    dimension: CountedBudgetDimension::AnalysisSteps,
                    ..
                })
            ));
            assert!(
                walker.visited.is_empty(),
                "the stopped proof publishes no ownership"
            );
        },
    );
}

#[test]
fn pre_cancelled_later_chain_remains_a_stop_without_partial_ownership() {
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(generous_limits(), token);
    with_real_loop(
        DEBUG,
        &mut budget,
        |walker, header, header_node, blocks, _, _| {
            let result = walker.header_test_chain(header, header_node, blocks);
            assert!(matches!(result, Err(StopReason::Cancelled { .. })));
            assert!(walker.visited.is_empty());
        },
    );
}

#[test]
fn cancellation_during_later_boundary_check_remains_a_stop() {
    let token = CancellationToken::new();
    let mut budget = Budget::with_cancellation_token(generous_limits(), token.clone());
    with_real_loop(
        DEBUG,
        &mut budget,
        |walker, header, header_node, _, candidate, exit| {
            let candidate_node = walker.view.index_of(candidate).expect("candidate node");
            let reason = match walker.proved_header_test(header, candidate, candidate_node) {
                Ok(Err(
                    reason @ FallbackReason::UnmetPrecondition {
                        requirement: crate::pass::Precondition::StatementFree,
                        ..
                    },
                )) => reason,
                other => panic!(
                    "the real later candidate must retain its precondition refusal: {other:?}"
                ),
            };
            token.cancel();
            assert!(matches!(
                walker.later_and_body_boundary(
                    header_node,
                    candidate,
                    candidate_node,
                    exit,
                    &reason
                ),
                Err(StopReason::Cancelled { .. })
            ));
            assert!(
                walker.visited.is_empty(),
                "the stopped boundary proof owns no blocks"
            );
        },
    );
}
