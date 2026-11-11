//! CHECKCAST collector tests on actual analyzed method IR and selected physical headers.
use jarde_java::{
    report::ReferenceRelation,
    source_proof::{
        DependencyResult, RecoveryHeaders, ReferenceClassHeader, prove_reference_relations,
    },
};
use jarde_jvm::{
    engine::analyze_method_ir,
    environment::ResolutionEnvironment,
    ir::{AnalysisStage, MethodAnalysisRequest},
};
use jarde_reader::{
    artifact::{ArtifactInput, ArtifactSnapshot},
    budget::{Budget, BudgetDimension, CancellationToken, Limits},
    classfile::class_member_facts,
    error::Error,
    model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    },
    view::*,
};
#[path = "../../../tests/support/reverse_cast_fixture.rs"]
mod fixture;
fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 22,
        archive_entries: 100,
        entry_bytes: 1 << 22,
        read_bytes: 1 << 22,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 64,
        method_bodies: 16,
        ir_items: 1 << 18,
        ir_edges: 1 << 18,
        analysis_steps: 1 << 18,
        normalization_clones: 0,
        nested_depth: 32,
        dependency_depth: 32,
        elapsed_millis: u64::MAX,
    }
}

fn definition(bytes: &[u8], snapshot: &ArtifactSnapshot) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: bytes.len() as u64,
        },
        variant: PhysicalVariant::Base,
    }
}
struct Headers {
    loader: LoaderId,
    entries: Vec<(Vec<u8>, ReferenceClassHeader)>,
}
impl RecoveryHeaders for Headers {
    type Error = Error;
    fn caller_loader(&self) -> &LoaderId {
        &self.loader
    }
    fn selected_from(
        &mut self,
        _loader: &LoaderId,
        owner: &[u8],
        budget: &mut Budget,
    ) -> Result<DependencyResult<ReferenceClassHeader>, Error> {
        budget.poll()?;
        Ok(self
            .entries
            .iter()
            .find(|(n, _)| n == owner)
            .map(|(_, h)| DependencyResult::Found(h.clone()))
            .unwrap_or(DependencyResult::Missing))
    }
    fn platform_unprovided(&mut self, _owner: &[u8], budget: &mut Budget) -> Result<bool, Error> {
        budget.poll()?;
        Ok(true)
    }
}
#[test]
fn checkcast_collects_both_ordered_pairs_at_the_physical_instruction() {
    for ((parent, child), wide) in [("RootFace", "LeafFace"), ("RenamedRoot", "RenamedLeaf")]
        .into_iter()
        .flat_map(|case| [false, true].map(|wide| (case, wide)))
    {
        let cast_bci = fixture::cast_bci(wide);
        let bytes = fixture::subject("CastProbe", parent, child, wide);
        let mut b = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.clone()), &mut b).unwrap();
        let domain = LoadDomain {
            loader: LoaderId("app".into()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        let request = MethodAnalysisRequest {
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
                providers: vec![],
            },
            method: PhysicalMethodId {
                owner: definition(&bytes, &snapshot),
                name: JvmBytes(b"run".to_vec()),
                descriptor: JvmBytes(format!("(L{parent};)L{child};").into_bytes()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        };
        let analyzed = analyze_method_ir(&[snapshot], &request, &mut b).unwrap();
        assert!(analyzed.ir().ssa().is_some());
        let mut headers = Headers {
            loader: LoaderId("app".into()),
            entries: vec![],
        };
        for (name, p) in [(parent, None), (child, Some(parent))] {
            let bytes = fixture::interface(name, p, false);
            let snap = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.clone()), &mut b).unwrap();
            let facts = class_member_facts(&bytes, &mut b).unwrap();
            let header = ReferenceClassHeader::from_facts(
                &facts,
                headers.loader.clone(),
                definition(&bytes, &snap),
            )
            .unwrap();
            headers.entries.push((name.as_bytes().to_vec(), header));
        }
        let relations =
            prove_reference_relations(&request, analyzed.ir(), &mut headers, &mut b).unwrap();
        let pair = |source: &str, target: &str| {
            relations
                .iter()
                .filter(|p| p.bci == cast_bci && p.source == source && p.target == target)
                .collect::<Vec<_>>()
        };
        let forward = pair(parent, child);
        let reverse = pair(child, parent);
        assert_eq!(forward.len(), 1);
        assert_eq!(reverse.len(), 1);
        assert!(!matches!(
            forward[0].relation,
            ReferenceRelation::SelectedHeaders | ReferenceRelation::PlatformContract { .. }
        ));
        assert_eq!(reverse[0].relation, ReferenceRelation::SelectedHeaders);
        assert!(
            relations.iter().all(|p| p.source != p.target),
            "identity pairs are not materialized"
        );
        headers.entries.retain(|(name, _)| name != child.as_bytes());
        let missing =
            prove_reference_relations(&request, analyzed.ir(), &mut headers, &mut b).unwrap();
        assert!(missing.iter().any(|p| p.bci == cast_bci
            && p.source == child
            && p.target == parent
            && p.relation == ReferenceRelation::Missing));
        let mut exhausted = limits();
        exhausted.analysis_steps = 0;
        assert!(
            matches!(
                prove_reference_relations(
                    &request,
                    analyzed.ir(),
                    &mut headers,
                    &mut Budget::new(exhausted)
                ),
                Err(Error::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps,
                    ..
                })
            ),
            "the exact analysis-step stop must propagate"
        );
        let token = CancellationToken::new();
        token.cancel();
        assert!(
            matches!(
                prove_reference_relations(
                    &request,
                    analyzed.ir(),
                    &mut headers,
                    &mut Budget::with_cancellation_token(limits(), token)
                ),
                Err(Error::Cancelled { .. })
            ),
            "cancellation must remain a cancellation"
        );
    }
}
