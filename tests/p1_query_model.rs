use jarde::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn origin(snapshot: &str, outer_ordinal: u64) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: SnapshotId(snapshot.into()),
        root_container: ContainerId("root".into()),
        steps: vec![
            ContainerOriginStep {
                via_ordinal: outer_ordinal,
                via_raw_name: ArchiveNameBytes(b"lib/child.jar".to_vec()),
                child_container: ContainerId(format!("child-{outer_ordinal}")),
            },
            ContainerOriginStep {
                via_ordinal: 4,
                via_raw_name: ArchiveNameBytes(b"nested/deeper.jar".to_vec()),
                child_container: ContainerId(format!("deep-{outer_ordinal}")),
            },
        ],
    }
}

fn archive_definition(origin: ContainerOrigin, variant: PhysicalVariant) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::ArchiveEntry {
            entry: PhysicalEntryId {
                origin,
                ordinal: 9,
                raw_name: ArchiveNameBytes(b"pkg/A.class".to_vec()),
            },
        },
        class_bytes: ClassBytesId {
            digest: Digest("same-bytes".into()),
            length: 123,
        },
        variant,
    }
}

fn load_domain(delegation: DelegationPolicy) -> LoadDomain {
    LoadDomain {
        loader: LoaderId("app-loader".into()),
        parent_loader: Some(LoaderId("platform-loader".into())),
        delegation,
        roots: vec![
            LoadRoot::External {
                id: "caller-root-b".into(),
            },
            LoadRoot::External {
                id: "caller-root-a".into(),
            },
        ],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::Possible,
        runtime_transformation: RuntimeUncertainty::Unknown,
    }
}

#[test]
fn standalone_and_archive_locations_round_trip_without_synthetic_entries() {
    let standalone = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: SnapshotId("standalone".into()),
        },
        class_bytes: ClassBytesId {
            digest: Digest("class".into()),
            length: 8,
        },
        variant: PhysicalVariant::Base,
    };
    let json = serde_json::to_value(&standalone).unwrap();
    let location = json
        .get("location")
        .and_then(serde_json::Value::as_object)
        .unwrap();
    assert_eq!(
        location.get("kind"),
        Some(&serde_json::json!("standalone_root"))
    );
    assert_eq!(
        location.get("snapshot"),
        Some(&serde_json::json!("standalone"))
    );
    for synthetic_field in ["entry", "ordinal", "raw_name", "origin", "root_container"] {
        assert!(!location.contains_key(synthetic_field));
    }
    let round_trip: PhysicalDefinitionId = serde_json::from_value(json).unwrap();
    assert_eq!(round_trip, standalone);
    assert_eq!(round_trip.snapshot(), &SnapshotId("standalone".into()));
    assert_eq!(round_trip.entry(), None);

    let archive = archive_definition(origin("archive", 1), PhysicalVariant::Base);
    assert!(archive.entry().is_some());
    assert_eq!(
        serde_json::from_str::<PhysicalDefinitionId>(&serde_json::to_string(&archive).unwrap())
            .unwrap(),
        archive
    );
}

#[test]
fn directed_nested_origins_preserve_edges_and_distinguish_duplicate_parents() {
    let left = archive_definition(origin("snapshot", 1), PhysicalVariant::Base);
    let right = archive_definition(origin("snapshot", 2), PhysicalVariant::Base);
    assert_eq!(left.class_bytes, right.class_bytes);
    assert_ne!(left, right);

    let left_entry = left.entry().unwrap();
    assert_eq!(left_entry.origin.steps.len(), 2);
    assert_eq!(left_entry.container(), &ContainerId("deep-1".into()));
    let json = serde_json::to_string(left_entry).unwrap();
    let round_trip: PhysicalEntryId = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, *left_entry);
    assert_eq!(hash(&round_trip), hash(left_entry));
}

#[test]
fn physical_variants_coexist_while_runtime_views_only_declare_requests() {
    let base = archive_definition(origin("mr", 1), PhysicalVariant::Base);
    let v11 = archive_definition(
        origin("mr", 1),
        PhysicalVariant::MultiRelease { version: 11 },
    );
    let v17 = archive_definition(
        origin("mr", 1),
        PhysicalVariant::MultiRelease { version: 17 },
    );
    assert_ne!(base, v11);
    assert_ne!(v11, v17);
    assert_ne!(base, v17);

    let physical = PhysicalView {
        snapshot: SnapshotId("mr".into()),
        scope: PhysicalScope::SnapshotAll,
    };
    let runtime = |java_release| RuntimeView {
        physical: physical.clone(),
        profile: RuntimeProfile {
            java_release,
            multi_release: MultiReleasePolicy::Enabled,
            layout: LayoutMode::Generic,
        },
        load_domain: load_domain(DelegationPolicy::ParentFirst),
    };
    let java8 = runtime(8);
    let java11 = runtime(11);
    let java17 = runtime(17);
    assert_eq!(java8.physical, java11.physical);
    assert_eq!(java11.physical, java17.physical);
    assert_ne!(java8, java11);
    assert_ne!(java11, java17);
}

#[test]
fn unknown_runtime_modes_round_trip_without_degrading_to_known_values() {
    let multi_release = MultiReleasePolicy::Unknown;
    assert_eq!(
        serde_json::from_value::<MultiReleasePolicy>(serde_json::to_value(&multi_release).unwrap())
            .unwrap(),
        multi_release
    );
    assert_ne!(multi_release, MultiReleasePolicy::Disabled);
    assert_ne!(hash(&multi_release), hash(&MultiReleasePolicy::Disabled));

    let layout = LayoutMode::Unknown;
    assert_eq!(
        serde_json::from_value::<LayoutMode>(serde_json::to_value(&layout).unwrap()).unwrap(),
        layout
    );
    assert_ne!(layout, LayoutMode::Generic);
    assert_ne!(hash(&layout), hash(&LayoutMode::Generic));

    let module = ModuleMode::Unknown;
    assert_eq!(
        serde_json::from_value::<ModuleMode>(serde_json::to_value(&module).unwrap()).unwrap(),
        module
    );
    assert_ne!(module, ModuleMode::ClassPath);
    assert_ne!(hash(&module), hash(&ModuleMode::ClassPath));
}

#[test]
fn load_domain_keeps_policy_root_order_and_uncertainty_in_identity() {
    let parent_first = load_domain(DelegationPolicy::ParentFirst);
    let child_first = load_domain(DelegationPolicy::ChildFirst);
    let unknown = load_domain(DelegationPolicy::Unknown);
    assert_ne!(parent_first, child_first);
    assert_ne!(parent_first, unknown);
    assert_ne!(child_first, unknown);
    assert_ne!(parent_first.roots[0], parent_first.roots[1]);

    let mut reversed = parent_first.clone();
    reversed.roots.reverse();
    assert_ne!(parent_first, reversed);
    let mut certain = parent_first.clone();
    certain.external_override = RuntimeUncertainty::None;
    certain.runtime_transformation = RuntimeUncertainty::None;
    assert_ne!(parent_first, certain);

    let json = serde_json::to_string(&unknown).unwrap();
    assert_eq!(serde_json::from_str::<LoadDomain>(&json).unwrap(), unknown);
}

#[test]
fn consumer_schema_is_canonical_and_all_query_relations_round_trip() {
    let left = ConsumerSchema::new(
        1,
        [
            ConsumerKind::Resource,
            ConsumerKind::Invocation,
            ConsumerKind::Resource,
            ConsumerKind::Annotation,
        ],
    );
    let right = ConsumerSchema::new(
        1,
        [
            ConsumerKind::Annotation,
            ConsumerKind::Resource,
            ConsumerKind::Invocation,
        ],
    );
    assert_eq!(left, right);
    assert_eq!(hash(&left), hash(&right));
    assert_eq!(
        serde_json::to_string(&left).unwrap(),
        serde_json::to_string(&right).unwrap()
    );

    let relations = [
        QueryRelation::ConstantPoolContains,
        QueryRelation::MentionsSymbol,
        QueryRelation::LiteralValue,
        QueryRelation::ReferencesDefinition,
        QueryRelation::MayDispatchTo,
    ];
    for relation in relations {
        let json = serde_json::to_string(&relation).unwrap();
        assert_eq!(
            serde_json::from_str::<QueryRelation>(&json).unwrap(),
            relation
        );
    }
}
