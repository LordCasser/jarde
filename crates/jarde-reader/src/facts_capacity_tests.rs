use super::*;
use crate::artifact::{ArtifactInput, ArtifactSnapshot, root_origin};
use crate::budget::Limits;
use crate::classfile::{class_facts, inspect_header};
use crate::model::{PhysicalClassLocation, PhysicalDefinitionId, PhysicalVariant};

fn budget() -> Budget {
    Budget::new(Limits {
        input_bytes: u64::MAX,
        archive_entries: u64::MAX,
        entry_bytes: u64::MAX,
        read_bytes: u64::MAX,
        class_bytes: u64::MAX,
        attribute_bytes: u64::MAX,
        code_bytes: u64::MAX,
        result_items: u64::MAX,
        output_bytes: u64::MAX,
        class_headers: u64::MAX,
        method_bodies: u64::MAX,
        ir_items: u64::MAX,
        ir_edges: u64::MAX,
        analysis_steps: u64::MAX,
        normalization_clones: u64::MAX,
        nested_depth: u64::MAX,
        dependency_depth: u64::MAX,
        elapsed_millis: u64::MAX,
    })
}

struct Products {
    container: Arc<ContainerFacts>,
    definition: PhysicalDefinitionId,
    bytes: Vec<u8>,
    payload: FactsPayload,
}

impl Products {
    fn read() -> Self {
        let archive = include_bytes!("../../../fuzz/corpus/query/minimal-jar");
        let snapshot = ArtifactSnapshot::open(
            ArtifactInput::Bytes(Arc::from(archive.as_slice())),
            &mut budget(),
        )
        .expect("fixture snapshot");
        let container = snapshot
            .container_facts(&root_origin(snapshot.id()), &mut budget())
            .expect("complete verified directory");
        let entry = container
            .entries()
            .iter()
            .find(|entry| entry.id.raw_name.0.ends_with(b".class"))
            .expect("fixture class");
        let read = snapshot
            .read_entry(entry, &mut budget())
            .expect("verified read");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::ArchiveEntry {
                entry: entry.id.clone(),
            },
            class_bytes: ClassBytesId {
                digest: read.content_digest,
                length: read.bytes.len() as u64,
            },
            variant: PhysicalVariant::Base,
        };
        let facts = class_facts(&read.bytes, &mut budget()).expect("complete class facts");
        Self {
            container,
            definition,
            bytes: read.bytes,
            payload: FactsPayload::Structure(Arc::new(facts)),
        }
    }

    fn insert(&self, cache: &FactsCache, layer: usize) {
        match layer {
            0 => cache.remember_container(&self.container),
            1 => cache.remember_definition_read(
                &self.definition,
                &self.bytes,
                &self.definition.class_bytes.digest,
            ),
            2 => cache.keep(
                FactsKey::of(&self.bytes, ParsePolicy::Structure),
                self.payload.clone(),
            ),
            _ => unreachable!(),
        }
    }

    fn weight(&self, layer: usize) -> u64 {
        if layer == 0 {
            self.container.weight()
        } else {
            self.bytes.len() as u64
        }
    }
}

#[test]
fn all_insertion_orders_share_entry_bounds_and_refusal_reasons() {
    let products = Products::read();
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for order in orders {
        for capacity in [0, 2, 3] {
            let cache = FactsCache::current(FactsCapacity::new(capacity, u64::MAX));
            for (index, layer) in order.into_iter().enumerate() {
                products.insert(&cache, layer);
                let report = cache.report();
                let retained = (index + 1).min(capacity);
                assert_eq!(
                    report.entries + report.containers + report.definition_reads,
                    retained
                );
                assert_eq!(
                    report.retained_bytes,
                    order[..retained]
                        .iter()
                        .map(|layer| products.weight(*layer))
                        .sum::<u64>()
                );
                assert_eq!(report.refused_capacity, (index + 1 - retained) as u64);
                assert_eq!(report.refused_capacity_bytes, 0);
            }
            // Rewriting each retained key at the full boundary consumes no new allowance.
            let before = cache.report();
            for layer in &order[..capacity.min(3)] {
                products.insert(&cache, *layer);
            }
            let after = cache.report();
            assert_eq!(after.retained_bytes, before.retained_bytes);
            assert_eq!(after.refused_capacity, before.refused_capacity);
            assert_eq!(
                after.entries + after.containers + after.definition_reads,
                capacity
            );
        }
        let cache = FactsCache::current(FactsCapacity::new(3, 0));
        for layer in order {
            products.insert(&cache, layer);
        }
        let report = cache.report();
        assert_eq!(
            report.entries + report.containers + report.definition_reads,
            0
        );
        assert_eq!(report.retained_bytes, 0);
        assert_eq!(report.refused_capacity, 0);
        assert_eq!(report.refused_capacity_bytes, 3);
    }
}

fn payload(bytes: &[u8], policy: ParsePolicy) -> FactsPayload {
    match policy {
        ParsePolicy::Structure => FactsPayload::Structure(Arc::new(
            class_facts(bytes, &mut budget()).expect("structure"),
        )),
        ParsePolicy::Strict | ParsePolicy::Forensic => {
            let mode = if policy == ParsePolicy::Strict {
                InspectionMode::Strict
            } else {
                InspectionMode::Forensic
            };
            FactsPayload::Header(Arc::new(
                inspect_header(bytes, &mut budget(), mode).expect("header"),
            ))
        }
    }
}

#[test]
fn each_policy_releases_discarded_weight_and_preserves_live_payload() {
    let products = Products::read();
    for policy in [
        ParsePolicy::Structure,
        ParsePolicy::Strict,
        ParsePolicy::Forensic,
    ] {
        for format in [true, false] {
            let cache = FactsCache::current(FactsCapacity::new(1, products.bytes.len() as u64));
            let key = FactsKey::of(&products.bytes, policy);
            cache.keep(key.clone(), payload(&products.bytes, policy));
            let live = cache
                .take(key.clone(), &mut budget())
                .expect("lookup")
                .expect("hit");
            let identity = cache.identity();
            let changed = if format {
                FactsIdentity {
                    format: identity.format + 1,
                    ..identity
                }
            } else {
                FactsIdentity {
                    registry: identity.registry + 1,
                    ..identity
                }
            };
            let reader = cache.over(changed);
            assert!(
                reader
                    .take(key.clone(), &mut budget())
                    .expect("discard")
                    .is_none()
            );
            let report = cache.report();
            assert_eq!(report.entries, 0);
            assert_eq!(report.retained_bytes, 0);
            assert_eq!(report.discarded_format, u64::from(format));
            assert_eq!(report.discarded_registry, u64::from(!format));
            assert!(policy.holds(&live));
            reader.keep(key.clone(), live.clone());
            let restored = reader
                .take(key, &mut budget())
                .expect("lookup")
                .expect("retained again without clear");
            match (&live, &restored) {
                (FactsPayload::Structure(a), FactsPayload::Structure(b)) => {
                    assert!(Arc::ptr_eq(a, b))
                }
                (FactsPayload::Header(a), FactsPayload::Header(b)) => assert!(Arc::ptr_eq(a, b)),
                _ => panic!("product changed"),
            }
            assert_eq!(reader.report().retained_bytes, products.bytes.len() as u64);
            assert_eq!(reader.report().refused_capacity_bytes, 0);
        }
    }
}

#[test]
fn mixed_store_discards_only_the_incompatible_cp_weight() {
    let products = Products::read();
    let total = products.container.weight() + 2 * products.bytes.len() as u64;
    for policy in [
        ParsePolicy::Structure,
        ParsePolicy::Strict,
        ParsePolicy::Forensic,
    ] {
        for reason in [Unusable::Format, Unusable::Registry, Unusable::Product] {
            let cache = FactsCache::current(FactsCapacity::new(3, total));
            products.insert(&cache, 0);
            products.insert(&cache, 1);
            let key = FactsKey::of(&products.bytes, policy);
            cache.keep(key.clone(), payload(&products.bytes, policy));
            let live = cache
                .take(key.clone(), &mut budget())
                .expect("lookup")
                .expect("live payload");
            // Same-key rewrite at full entry and byte capacity must keep the same weight.
            cache.keep(key.clone(), live.clone());
            assert_eq!(cache.report().retained_bytes, total);
            {
                let mut shared = cache.shared();
                let entry = shared.entries.get_mut(&key).expect("retained CP entry");
                match reason {
                    Unusable::Format => entry.identity.format += 1,
                    Unusable::Registry => entry.identity.registry += 1,
                    Unusable::Product => {
                        let other = if policy == ParsePolicy::Structure {
                            ParsePolicy::Strict
                        } else {
                            ParsePolicy::Structure
                        };
                        entry.payload = payload(&products.bytes, other);
                    }
                }
            }
            assert!(
                cache
                    .take(key.clone(), &mut budget())
                    .expect("discard")
                    .is_none()
            );
            let report = cache.report();
            assert_eq!(report.entries, 0);
            assert_eq!(report.retained_bytes, total - products.bytes.len() as u64);
            assert_eq!(
                report.discarded_format,
                u64::from(matches!(reason, Unusable::Format))
            );
            assert_eq!(
                report.discarded_registry,
                u64::from(matches!(reason, Unusable::Registry))
            );
            assert_eq!(
                report.discarded_product,
                u64::from(matches!(reason, Unusable::Product))
            );
            let container = cache
                .container(products.container.origin(), &mut budget())
                .expect("lookup")
                .expect("other layer survives");
            assert!(Arc::ptr_eq(&container, &products.container));
            let (bytes, identity) = cache
                .definition_read(&products.definition, &mut budget())
                .expect("lookup")
                .expect("read survives");
            assert_eq!(bytes.as_slice(), products.bytes);
            assert_eq!(identity, products.definition.class_bytes);
            cache.keep(key.clone(), live.clone());
            let restored = cache
                .take(key, &mut budget())
                .expect("lookup")
                .expect("reinserted");
            match (&live, &restored) {
                (FactsPayload::Structure(a), FactsPayload::Structure(b)) => {
                    assert!(Arc::ptr_eq(a, b))
                }
                (FactsPayload::Header(a), FactsPayload::Header(b)) => assert!(Arc::ptr_eq(a, b)),
                _ => panic!("product changed"),
            }
            let report = cache.report();
            assert_eq!(report.retained_bytes, total);
            assert_eq!(
                report.entries + report.containers + report.definition_reads,
                3
            );
            assert_eq!(report.refused_capacity, 0);
            assert_eq!(report.refused_capacity_bytes, 0);
            assert_eq!(report.container_hits, 1);
            assert_eq!(report.definition_read_hits, 1);
        }
    }
}
