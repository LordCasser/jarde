//! Bulk task 2.3 (stream E′): the driver read and the same-class callee read consume a **prepared**
//! class.
//!
//! The single-method path (`Engine::recover_method`) reads one class per request: one header read by
//! identity, one body decode. A bulk operation cannot afford that shape — a class with `N` methods
//! would be read `N` times — so a worker prepares the class once (`jarde_reader::prepared`) and
//! analyses every method of that class against that one preparation. This file pins the second half
//! of that lifecycle, through the entries a worker calls:
//!
//! * [`jarde_jvm::analyze_prepared_method_ir`] publishes **the same report** as
//!   [`jarde_jvm::analyze_method_ir`] for the same request — field by field, with the two things a
//!   different budget must move taken out (`elapsed_millis` and the usage counts) — and hands over a
//!   payload of the same shape: the same declaration, the same decoded body, the same constant pool,
//!   the same `BootstrapMethods` table, and the same presence of the canonical graph, the frames and
//!   the names;
//! * the presentation (`jarde_java::recover`) writes the same text from the prepared payload as the
//!   facade writes for the direct one — on a body compiled without debug metadata and on one that
//!   carries its `LocalVariableTable`;
//! * the class is paid for **once**: N prepared analyses charge no `ClassHeaders` for the definition
//!   itself (the preparation read the class once) — while the direct path charges one per request and
//!   N of them for N methods — and the prepared path still charges one `MethodBodies` attempt per
//!   decoded body. Both paths run the same binding search over the declared order, so a position
//!   that search has to examine outside this definition is charged on both;
//! * [`jarde_jvm::callee::read_prepared_callees`] answers the same members, bodies, refusals and read
//!   record as [`jarde_jvm::callee::read_callees`] for the same candidates, without a second class
//!   read;
//! * the same holds for a class stored in an archive, which is the shape a bulk scope really walks:
//!   the coordinate the request names, the prepared class and the read record all refer to that one
//!   entry.
//!
//! What this file does **not** prove: that a bulk operation is wired to these entries (the facade's
//! `recover_method` keeps calling the direct path, and the bulk scheduler is a later task), that the
//! prepared path is faster (nothing here measures time), and that a class whose *member table* is
//! damaged behaves the same on both paths — the prepared view refuses such a class with the stop's
//! own code (`the_prepared_path_refuses_a_class_whose_member_table_stopped`), which is the reader's
//! stated behaviour rather than a per-member verdict this file could compare.

use jarde::*;
use jarde_java::{
    DebugLocal, DeclaringClass, MethodFacts, RecoveryFacts, RecoveryReport, RecoveryRequest,
};
use jarde_jvm::callee::{CalleeCandidate, CalleeReadRequest, read_callees, read_prepared_callees};
use jarde_jvm::engine::{analyze_method_ir, analyze_prepared_method_ir};
use jarde_jvm::method_ir::{MethodIr, MethodIrAnalysis};
use jarde_reader::classfile::{BootstrapMethodFacts, CpEntryFacts, MethodCodeFacts};
use jarde_reader::prepared::{MethodCodeAttribute, PreparedClass};
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;

/// The 8-method sample compiled without debug metadata (`javac --release 8 -g:none`).
const SCOPE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");
/// The same source with its debug evidence (`javac --release 8 -g`).
const SCOPE_DEBUG: &[u8] = include_bytes!("fixtures/p3-scope/v8-debug/Scope.class");
/// An ordinary class: a constructor, an instance method, a static method and a `<clinit>`.
const HOLDER: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Holder.class");
/// An interface: `default`/`static` members beside one member that declares no body at all.
const SHAPE: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Shape.class");
/// A Java 8 sample with one `invokedynamic` site and the `BootstrapMethods` table it resolves
/// against: the attribute the payload hands on and the presentation reads back.
const LAMBDA: &[u8] = include_bytes!("fixtures/b2-bootstrap-descriptor/v8/LambdaSample.class");

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
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

/// The one load root a standalone sample is: the snapshot's own class file.
fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
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

/// The same runtime view over an archive: its root container, searched under `prefix` — the path
/// the class entry this sample names really sits at.
fn archive_environment(snapshot: &ArtifactSnapshot, prefix: &[u8]) -> ResolutionEnvironment {
    let mut environment = environment(snapshot);
    environment.runtime.load_domain.roots = vec![LoadRoot::Container {
        origin: ContainerOrigin {
            snapshot: snapshot.id().clone(),
            root_container: ContainerId("root".into()),
            steps: Vec::new(),
        },
        prefix: ArchiveNameBytes(prefix.to_vec()),
    }];
    environment.domains = vec![environment.runtime.load_domain.clone()];
    environment
}

/// One stored entry per name, as a ZIP no class read has to decompress.
fn stored_archive(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
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

/// One opened sample: the snapshot, the identity of its class file, and the members **its own
/// header** declares (name, descriptor, whether the record declares a `Code` shell).
struct Sample {
    snapshot: ArtifactSnapshot,
    definition: PhysicalDefinitionId,
    members: Vec<(Vec<u8>, Vec<u8>, bool)>,
    /// The prefix the class's entry sits under inside the snapshot's root container; empty for a
    /// standalone CLASS file, whose whole content is the definition.
    root_prefix: Vec<u8>,
}

impl Sample {
    /// One standalone CLASS file, with the members its own header read states.
    fn standalone(bytes: &[u8]) -> Self {
        let engine = Engine::new();
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
            .expect("the fixture opens as a standalone CLASS");
        let inspected = engine
            .inspect_header(
                &snapshot,
                ClassTarget::Root,
                &mut budget,
                InspectionMode::Strict,
            )
            .expect("the fixture's own header is readable");
        let members = member_summary(&inspected.inspection.header.methods);
        Self {
            definition: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            snapshot,
            members,
            root_prefix: Vec::new(),
        }
    }

    /// One class of an archive, as the declaration listing confirms it.
    ///
    /// The listing's declaration record states the class-level facts only, so the member summary
    /// comes from the class's own prepared member table — the same records a method request is
    /// answered from.
    fn archived(bytes: &[u8], name: &[u8]) -> Self {
        let engine = Engine::new();
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
            .expect("the fixture opens as a ZIP");
        let listing = engine
            .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
            .expect("the archive's declarations are listable");
        let item = listing
            .items
            .iter()
            .find(|item| {
                item.definition
                    .entry()
                    .is_some_and(|entry| entry.raw_name.0 == name)
            })
            .unwrap_or_else(|| {
                panic!(
                    "{} holds no class entry named `{}`: {:?}",
                    snapshot.id().0,
                    String::from_utf8_lossy(name),
                    listing
                        .items
                        .iter()
                        .map(|item| item
                            .definition
                            .entry()
                            .map(|entry| entry.raw_name.0.clone()))
                        .collect::<Vec<_>>()
                )
            });
        let prefix = name
            .iter()
            .rposition(|byte| *byte == b'/')
            .map_or_else(Vec::new, |slash| name[..=slash].to_vec());
        let mut sample = Self {
            definition: item.definition.clone(),
            snapshot,
            members: Vec::new(),
            root_prefix: prefix,
        };
        let mut budget = Budget::new(limits());
        let members = sample.with_prepared(&mut budget, |prepared, _budget| {
            prepared
                .method_slots()
                .iter()
                .map(|slot| {
                    (
                        slot.name.raw().0.clone(),
                        slot.descriptor.raw().0.clone(),
                        slot.code != MethodCodeAttribute::Absent,
                    )
                })
                .collect()
        });
        sample.members = members;
        sample
    }

    fn content(&self) -> &[ArtifactSnapshot] {
        slice::from_ref(&self.snapshot)
    }

    fn environment(&self) -> ResolutionEnvironment {
        match &self.definition.location {
            PhysicalClassLocation::StandaloneRoot { .. } => environment(&self.snapshot),
            PhysicalClassLocation::ArchiveEntry { .. } => {
                archive_environment(&self.snapshot, &self.root_prefix)
            }
        }
    }

    fn request(&self, name: &[u8], descriptor: &[u8]) -> MethodAnalysisRequest {
        MethodAnalysisRequest {
            environment: self.environment(),
            method: PhysicalMethodId {
                owner: self.definition.clone(),
                name: JvmBytes(name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        }
    }

    /// The members that really declare a body, which are the ones a worker decodes.
    fn bodies(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.members
            .iter()
            .filter(|(_, _, has_code)| *has_code)
            .map(|(name, descriptor, _)| (name.clone(), descriptor.clone()))
            .collect()
    }

    /// Prepares this sample's class under `budget` and runs `body` with it.
    ///
    /// The prepared class borrows the read this function owns, so it cannot outlive the closure —
    /// exactly the lifecycle the contract states (a borrow owned by the caller for the duration of
    /// the call), and the reason nothing in these tests stores one.
    fn with_prepared<T>(
        &self,
        budget: &mut Budget,
        body: impl FnOnce(&PreparedClass<'_>, &mut Budget) -> T,
    ) -> T {
        let read = match self.definition.entry() {
            Some(entry) => self
                .snapshot
                .prepared_class(entry, budget)
                .expect("the listed class entry prepares"),
            None => self
                .snapshot
                .prepared_root_class(budget)
                .expect("a standalone CLASS root prepares"),
        };
        let prepared = PreparedClass::prepare(&read, budget).expect("the fixture's class prepares");
        body(&prepared, budget)
    }
}

fn member_summary(
    members: &[jarde_reader::classfile::MemberHeader],
) -> Vec<(Vec<u8>, Vec<u8>, bool)> {
    members
        .iter()
        .map(|member| {
            (
                member.name.raw().0.clone(),
                member.descriptor.raw().0.clone(),
                member
                    .attributes
                    .iter()
                    .any(|shell| shell.name.raw().0.as_slice() == b"Code"),
            )
        })
        .collect()
}

// -----------------------------------------------------------------------------------------------
// The report and the payload
// -----------------------------------------------------------------------------------------------

/// One execution report under a zeroed usage snapshot.
///
/// The prepared run is a **different request** over the same bytes — it read no class — so its
/// `execution` carries a different usage snapshot and a different elapsed reading. Everything else
/// in a report is what the run decided about the same input.
fn execution_without_usage(execution: &ExecutionReport) -> ExecutionReport {
    let usage = UsageSnapshot::default();
    match execution {
        ExecutionReport::Complete { .. } => ExecutionReport::Complete { usage },
        ExecutionReport::Partial { reason, .. } => ExecutionReport::Partial {
            reason: reason.clone(),
            usage,
        },
        ExecutionReport::Cancelled { .. } => ExecutionReport::Cancelled { usage },
        ExecutionReport::Failed { reason, .. } => ExecutionReport::Failed {
            reason: reason.clone(),
            usage,
        },
    }
}

/// The report of one run, with the two things a budget decides taken out.
fn report_without_usage(analysis: &MethodIrAnalysis) -> MethodAnalysisReport {
    let mut report = analysis.report().clone();
    report.execution = execution_without_usage(&report.execution);
    report
}

/// The declaration one run's read stated about the member, in comparable form.
#[derive(Debug, Eq, PartialEq)]
struct DeclarationShape {
    access_flags: u16,
    name: Vec<u8>,
    descriptor: Vec<u8>,
    parameter_slots: u16,
    class_name: Vec<u8>,
    class_access_flags: u16,
    identity: PhysicalMethodId,
}

/// What a payload states about one method, in the shape the two paths must agree on.
///
/// The canonical graph, the frames and the names are not restated here: the report above pins every
/// stage's state for the same run, and those tables are the passes' own artifacts over the body
/// facts this value does compare. What this is, is the *read*: the declaration, the decoded body,
/// the class's constant pool, its bootstrap table, and whether each later table was published at all.
#[derive(Debug, Eq, PartialEq)]
struct PayloadShape {
    canonical: bool,
    frames: bool,
    ssa: bool,
    code: Option<MethodCodeFacts>,
    constant_pool: Vec<CpEntryFacts>,
    bootstrap_methods: Vec<BootstrapMethodFacts>,
    declaration: Option<DeclarationShape>,
    member: Option<PhysicalMethodId>,
}

fn payload_shape(analysis: &MethodIrAnalysis) -> PayloadShape {
    let ir: &MethodIr = analysis.ir();
    let mut code = ir.code().cloned();
    if let Some(code) = code.as_mut() {
        code.execution = execution_without_usage(&code.execution);
    }
    PayloadShape {
        canonical: ir.canonical().is_some(),
        frames: ir.frames().is_some(),
        ssa: ir.ssa().is_some(),
        code,
        constant_pool: ir.constant_pool().to_vec(),
        bootstrap_methods: ir.bootstrap_methods().to_vec(),
        declaration: ir.declaration().map(|declaration| DeclarationShape {
            access_flags: declaration.access_flags(),
            name: declaration.name().0.clone(),
            descriptor: declaration.descriptor().0.clone(),
            parameter_slots: declaration.parameter_slots(),
            class_name: declaration.class_name().0.clone(),
            class_access_flags: declaration.class_access_flags(),
            identity: declaration.identity().clone(),
        }),
        member: ir.declaration().map(|_| analysis.report().method.clone()),
    }
}

/// Usage of one completed run, or a panic naming the report that did not complete.
fn complete_usage(analysis: &MethodIrAnalysis) -> UsageSnapshot {
    match &analysis.report().execution {
        ExecutionReport::Complete { usage } => usage.clone(),
        other => panic!("a legal request completes: {other:?}"),
    }
}

#[test]
fn every_member_of_one_class_is_the_same_report_from_a_prepared_class() {
    // Four samples, every member each one declares: bodies (a constructor, a `<clinit>`, instance
    // and static methods, an interface `default`, a lambda body), a member without a body (an
    // interface's `abstract` method), bodies with and without debug evidence, and a class that
    // declares a `BootstrapMethods` table.
    for (label, bytes) in [
        ("Scope", SCOPE),
        ("Scope (debug)", SCOPE_DEBUG),
        ("Holder", HOLDER),
        ("Shape", SHAPE),
        ("LambdaSample", LAMBDA),
    ] {
        let sample = Sample::standalone(bytes);
        assert!(!sample.members.is_empty(), "{label} declares members");
        let mut budget = Budget::new(limits());
        sample.with_prepared(&mut budget, |prepared, budget| {
            for (name, descriptor, _) in &sample.members {
                let request = sample.request(name, descriptor);
                let direct =
                    analyze_method_ir(sample.content(), &request, &mut Budget::new(limits()))
                        .expect("a legal request is answered, not raised");
                let from_prepared =
                    analyze_prepared_method_ir(sample.content(), prepared, &request, budget)
                        .expect("a prepared input is a legal request too");
                let member = format!(
                    "{label}.{}{}",
                    String::from_utf8_lossy(name),
                    String::from_utf8_lossy(descriptor)
                );
                assert_eq!(
                    report_without_usage(&from_prepared),
                    report_without_usage(&direct),
                    "{member}: the prepared run publishes the direct report"
                );
                assert_eq!(
                    payload_shape(&from_prepared),
                    payload_shape(&direct),
                    "{member}: and a payload of the same shape"
                );
                if label == "LambdaSample" {
                    assert!(
                        !direct.ir().bootstrap_methods().is_empty(),
                        "{member}: this sample declares a bootstrap table, so the comparison above \
                         is about a non-empty one"
                    );
                }
            }
        });
    }
}

// -----------------------------------------------------------------------------------------------
// What the class read costs
// -----------------------------------------------------------------------------------------------

#[test]
fn the_class_read_is_paid_once_for_the_whole_class() {
    // The same class, member by member: the direct path reads it once per request, the prepared path
    // reads it once — in the preparation — and charges nothing of the kind per method.
    let sample = Sample::standalone(SCOPE);
    let bodies = sample.bodies();
    let count = u64::try_from(bodies.len()).expect("a fixture's member count fits u64");
    assert_eq!(count, 8, "the sample declares eight bodies");

    let mut direct_headers = 0_u64;
    let mut direct_bodies = 0_u64;
    for (name, descriptor) in &bodies {
        let analysis = analyze_method_ir(
            sample.content(),
            &sample.request(name, descriptor),
            &mut Budget::new(limits()),
        )
        .expect("a legal request is answered, not raised");
        let usage = complete_usage(&analysis);
        direct_headers += usage.class_headers;
        direct_bodies += usage.method_bodies;
    }
    assert_eq!(
        direct_headers, count,
        "the direct path reads the class once per request"
    );
    assert_eq!(direct_bodies, count, "and decodes one body per request");

    let mut budget = Budget::new(limits());
    let (after_preparation, after_methods) =
        sample.with_prepared(&mut budget, |prepared, budget| {
            let after_preparation = budget.usage();
            for (name, descriptor) in &bodies {
                let analysis = analyze_prepared_method_ir(
                    sample.content(),
                    prepared,
                    &sample.request(name, descriptor),
                    budget,
                )
                .expect("a prepared input is a legal request too");
                assert!(
                    matches!(
                        analysis.report().execution,
                        ExecutionReport::Complete { .. }
                    ),
                    "{}.{}{}: {:?}",
                    sample.definition.snapshot().0,
                    String::from_utf8_lossy(name),
                    String::from_utf8_lossy(descriptor),
                    analysis.report().execution
                );
            }
            (after_preparation, budget.usage())
        });
    assert!(
        after_preparation.class_bytes > 0,
        "the preparation really read the class: {after_preparation:?}"
    );
    assert_eq!(
        after_preparation.class_headers, 0,
        "a prepared read is not a header read: {after_preparation:?}"
    );
    assert_eq!(
        after_preparation.method_bodies, 0,
        "and the preparation decoded no body nobody asked for"
    );
    assert_eq!(
        after_methods.class_headers, after_preparation.class_headers,
        "the class's methods charge no class header at all"
    );
    assert_eq!(
        after_methods.method_bodies,
        after_preparation.method_bodies + count,
        "while each decoded body is still charged once"
    );
}

// -----------------------------------------------------------------------------------------------
// The presentation the payload produces
// -----------------------------------------------------------------------------------------------

/// The facts the facade derives for one run, written here the way `Engine::recover_method` writes
/// them (P3 3.1/3.3).
///
/// This is the one helper of this file that repeats production logic: `recover_method` derives these
/// facts from the payload's declaration, its decoded body and the request's own member, and it does
/// so *inside* the facade — which is neither this stream's file nor an entry that takes a prepared
/// class. The equalities below hold the helper to that derivation: the facts it derives from the
/// prepared payload must be the facade's own facts for the direct run over the same member, and only
/// then is the text the prepared payload produces comparable with the text the facade printed.
fn derived_facts(analysis: &MethodIrAnalysis, method: &PhysicalMethodId) -> RecoveryFacts {
    let debug = match analysis.ir().code().map(MethodCodeFacts::debug) {
        Some(jarde_reader::classfile::LocalDebugTable::Read(records)) => records
            .iter()
            .map(|record| {
                DebugLocal::over(
                    record.slot,
                    record.name_lossy(),
                    record.start_bci,
                    record.end_bci,
                )
            })
            .collect(),
        _ => Vec::new(),
    };
    let Some(declaration) = analysis.ir().declaration() else {
        let name = String::from_utf8_lossy(&method.name.0).into_owned();
        let descriptor = String::from_utf8_lossy(&method.descriptor.0).into_owned();
        return RecoveryFacts::new(MethodFacts::new(name, descriptor, 0)).with_debug_locals(debug);
    };
    RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(&declaration.name().0).into_owned(),
            String::from_utf8_lossy(&declaration.descriptor().0).into_owned(),
            declaration.parameter_slots(),
        )
        .with_access_flags(declaration.access_flags())
        .with_declaring_class(DeclaringClass::new(
            String::from_utf8_lossy(&declaration.class_name().0).into_owned(),
            declaration.class_access_flags(),
        )),
    )
    .with_debug_locals(debug)
}

/// One presentation report with the usage of its own run taken out, and the **artifact statement**
/// beside it.
///
/// The direct route below goes through the facade, which states what the artifact it committed is
/// *of* (change `add-demand-driven-core-results`, D3'), while this file builds the prepared request
/// itself and states no subject: the two halves of [`RecoveryReport::artifact`] are the *entry's*
/// statement about the artifact, and every other field is what the presentation of the same run
/// produced. The identity half is asserted where it belongs — the case below checks that the direct
/// entry really publishes one — so this helper keeps the comparison this file is about exact.
fn recovery_without_usage(report: &RecoveryReport) -> RecoveryReport {
    let mut report = report.clone();
    report.execution = execution_without_usage(&report.execution);
    report.artifact = RecoveryArtifact::default();
    report
}

#[test]
fn the_prepared_payload_prints_the_text_the_direct_run_prints() {
    // The facade's own entry for the direct path — it hands over the facts it derived for that run,
    // which is what holds the helper above to the facade's own rule — and the prepared entry for the
    // same member, presented under the very same profile.
    let engine = Engine::new();
    for (label, bytes, name, descriptor) in [
        (
            "Scope.simple",
            SCOPE,
            b"simple".as_slice(),
            b"()I".as_slice(),
        ),
        (
            "Scope.scope",
            SCOPE,
            b"scope".as_slice(),
            b"(Z)I".as_slice(),
        ),
        (
            "Scope-debug.reuse",
            SCOPE_DEBUG,
            b"reuse".as_slice(),
            b"(ZI)I".as_slice(),
        ),
        (
            "Scope-debug.receiver",
            SCOPE_DEBUG,
            b"receiver".as_slice(),
            b"(J)J".as_slice(),
        ),
        (
            "Holder.<init>",
            HOLDER,
            b"<init>".as_slice(),
            b"(I)V".as_slice(),
        ),
        (
            "Holder.value",
            HOLDER,
            b"value".as_slice(),
            b"()I".as_slice(),
        ),
        (
            "Holder.of",
            HOLDER,
            b"of".as_slice(),
            b"(I)LHolder;".as_slice(),
        ),
        (
            "Shape.scaled",
            SHAPE,
            b"scaled".as_slice(),
            b"(I)I".as_slice(),
        ),
        (
            "LambdaSample.run",
            LAMBDA,
            b"run".as_slice(),
            b"()V".as_slice(),
        ),
    ] {
        let sample = Sample::standalone(bytes);
        let request = sample.request(name, descriptor);
        let direct = engine
            .recover_method(sample.content(), &request, &mut Budget::new(limits()))
            .expect("a legal request is answered, not raised");
        assert!(
            direct.callees().is_none(),
            "{label}: the body names no accessor call site, so both runs present the same input"
        );
        let profile = request.environment.runtime.profile.clone();
        let mut budget = Budget::new(limits());
        let (prepared_report, prepared_facts) =
            sample.with_prepared(&mut budget, |prepared, budget| {
                let analysis =
                    analyze_prepared_method_ir(sample.content(), prepared, &request, budget)
                        .expect("a prepared input is a legal request too");
                let facts = derived_facts(&analysis, &request.method);
                let report = jarde_java::recover(
                    &RecoveryRequest::new(analysis.ir(), &facts, profile.clone()),
                    budget,
                );
                (report, facts)
            });
        assert_eq!(
            prepared_facts,
            direct.facts().clone(),
            "{label}: the prepared payload derives the facts the facade derived from the direct one"
        );
        assert_eq!(
            recovery_without_usage(&prepared_report),
            recovery_without_usage(direct.recovery()),
            "{label}: the prepared payload prints the direct report"
        );
        assert!(
            direct.recovery().artifact.binding().is_some(),
            "{label}: the facade's own entry states the artifact it was bound to"
        );
        assert_eq!(
            prepared_report.text,
            direct.recovery().text,
            "{label}: and the text itself, byte for byte"
        );
        assert!(
            !prepared_report.text.is_empty(),
            "{label}: something was printed"
        );
    }
}

// -----------------------------------------------------------------------------------------------
// The same-class callee read
// -----------------------------------------------------------------------------------------------

/// One callee report in comparable form: what it read, what it refused and what it recorded.
#[derive(Debug, Eq, PartialEq)]
struct CalleeShape {
    class: String,
    members: Vec<(PhysicalMethodId, u16, Option<MethodCodeFacts>)>,
    refusals: Vec<(String, String, CalleeCandidate)>,
    reads: Vec<(ReadReason, PhysicalDefinitionId)>,
}

fn callee_shape(report: &jarde_jvm::callee::CalleeReadReport) -> CalleeShape {
    CalleeShape {
        class: report.class().to_string(),
        members: report
            .members()
            .iter()
            .map(|member| {
                let body = member.body().map(|body| {
                    let mut facts = body.facts().clone();
                    facts.execution = execution_without_usage(&facts.execution);
                    facts
                });
                (member.identity().clone(), member.access_flags(), body)
            })
            .collect(),
        refusals: report
            .refusals()
            .iter()
            .map(|refusal| {
                (
                    refusal.code().to_string(),
                    refusal.message().to_string(),
                    refusal.candidate().clone(),
                )
            })
            .collect(),
        reads: report
            .reads()
            .iter()
            .map(|read| (read.reason, read.definition.clone()))
            .collect(),
    }
}

#[test]
fn the_prepared_callee_read_answers_what_the_direct_one_answers() {
    // The candidates one presented body's call sites would name, plus the two shapes the read
    // refuses: a member this class does not declare, and a call site naming another class. The list
    // is the caller's own input — the accessor rule derives it from a body — which is what makes the
    // two entries comparable on one request.
    let sample = Sample::standalone(HOLDER);
    let candidates = vec![
        CalleeCandidate::new(1, b"Holder".to_vec(), b"value".to_vec(), b"()I".to_vec()),
        // The same member again, from another call site: answered by the member the first one read.
        CalleeCandidate::new(9, b"Holder".to_vec(), b"value".to_vec(), b"()I".to_vec()),
        CalleeCandidate::new(3, b"Holder".to_vec(), b"<init>".to_vec(), b"(I)V".to_vec()),
        CalleeCandidate::new(5, b"Holder".to_vec(), b"missing".to_vec(), b"()V".to_vec()),
        CalleeCandidate::new(7, b"Other".to_vec(), b"value".to_vec(), b"()I".to_vec()),
    ];

    let mut direct_budget = Budget::new(limits());
    let direct = read_callees(
        sample.content(),
        &CalleeReadRequest::new(&sample.environment(), &sample.definition, &candidates),
        &mut direct_budget,
    )
    .expect("a legal request is answered, not raised");
    let direct_usage = direct_budget.usage();

    let mut budget = Budget::new(limits());
    let (after_preparation, prepared_report, after_callees) =
        sample.with_prepared(&mut budget, |prepared, budget| {
            let after_preparation = budget.usage();
            let report = read_prepared_callees(
                sample.content(),
                prepared,
                &CalleeReadRequest::new(&sample.environment(), &sample.definition, &candidates),
                budget,
            )
            .expect("the prepared class answers the same request");
            let after_callees = budget.usage();
            (after_preparation, report, after_callees)
        });

    assert_eq!(
        callee_shape(&prepared_report),
        callee_shape(&direct),
        "the same class, the same members, the same bodies, the same refusals and the same read"
    );
    assert_eq!(
        direct
            .members()
            .iter()
            .map(|member| member.identity().name.0.clone())
            .collect::<Vec<_>>(),
        vec![b"value".to_vec(), b"<init>".to_vec()],
        "the distinct members the candidates named, in candidate order"
    );
    assert_eq!(
        direct
            .refusals()
            .iter()
            .map(|refusal| refusal.code())
            .collect::<Vec<_>>(),
        vec!["callee_not_declared", "callee_not_this_class"],
        "the undeclared member and the other class's member: {:?}",
        direct.refusals()
    );

    // The charges: the direct read reads the class definition for the members it is asked about, the
    // prepared read read it when the class was prepared — and both charge one body attempt per
    // distinct member the class declares with a body.
    assert_eq!(
        direct_usage.class_headers, 1,
        "the direct read reads the class definition once"
    );
    assert_eq!(
        after_preparation.class_headers, 0,
        "the preparation is a class read, not a header read: {after_preparation:?}"
    );
    assert_eq!(
        after_callees.class_headers, after_preparation.class_headers,
        "and the prepared callee read reads no class at all"
    );
    assert_eq!(
        direct_usage.method_bodies, 2,
        "one attempt per distinct member the class declares with a body"
    );
    assert_eq!(
        after_callees.method_bodies,
        after_preparation.method_bodies + direct_usage.method_bodies,
        "the prepared read charges exactly the same bodies"
    );
}

#[test]
fn an_archived_class_reads_the_same_on_both_paths() {
    // The shape a bulk scope really walks: a class inside an archive. The identity the request
    // names, the prepared read and the published read record all have to refer to that one entry.
    let holder = stored_archive(&[
        (b"p/Holder.class", HOLDER),
        (b"p/resources.txt", b"not a class"),
    ]);
    let sample = Sample::archived(&holder, b"p/Holder.class");
    assert!(
        matches!(
            sample.definition.location,
            PhysicalClassLocation::ArchiveEntry { .. }
        ),
        "the sample's class is one archive entry"
    );

    let request = sample.request(b"value", b"()I");
    let direct = analyze_method_ir(sample.content(), &request, &mut Budget::new(limits()))
        .expect("a legal request is answered, not raised");
    let mut budget = Budget::new(limits());
    let (after_preparation, from_prepared) =
        sample.with_prepared(&mut budget, |prepared, budget| {
            let after_preparation = budget.usage();
            let analysis = analyze_prepared_method_ir(sample.content(), prepared, &request, budget)
                .expect("a prepared input is a legal request too");
            (after_preparation, analysis)
        });
    assert_eq!(
        report_without_usage(&from_prepared),
        report_without_usage(&direct),
        "the archived class's report is the direct one"
    );
    assert_eq!(
        payload_shape(&from_prepared),
        payload_shape(&direct),
        "and its payload is of the same shape"
    );
    assert_eq!(
        from_prepared
            .report()
            .reads
            .iter()
            .map(|read| (read.reason, read.definition.clone()))
            .collect::<Vec<_>>(),
        direct
            .report()
            .reads
            .iter()
            .map(|read| (read.reason, read.definition.clone()))
            .collect::<Vec<_>>(),
        "the read record names the same definition, under the same reason"
    );
    assert_eq!(
        direct.report().reads.len(),
        1,
        "the entry's own definition, read once: {:?}",
        direct.report().reads
    );
    assert_eq!(
        after_preparation.class_headers, 0,
        "the entry's class read was the preparation's: {after_preparation:?}"
    );

    // And the same-class accessor members come out of that entry too, without a second class read.
    let candidates = vec![CalleeCandidate::new(
        1,
        b"Holder".to_vec(),
        b"value".to_vec(),
        b"()I".to_vec(),
    )];
    let expected = read_callees(
        sample.content(),
        &CalleeReadRequest::new(&sample.environment(), &sample.definition, &candidates),
        &mut Budget::new(limits()),
    )
    .expect("the archived class declares the member");
    assert_eq!(
        expected
            .reads()
            .iter()
            .map(|read| read.definition.clone())
            .collect::<Vec<_>>(),
        vec![sample.definition.clone()],
        "the direct callee read bound the same entry"
    );
    let from_prepared = sample.with_prepared(&mut Budget::new(limits()), |prepared, budget| {
        read_prepared_callees(
            sample.content(),
            prepared,
            &CalleeReadRequest::new(&sample.environment(), &sample.definition, &candidates),
            budget,
        )
        .expect("the prepared entry answers the same request")
    });
    assert_eq!(
        callee_shape(&from_prepared),
        callee_shape(&expected),
        "the member of the entry the prepared class holds, answered as the direct read answers it"
    );
}

// -----------------------------------------------------------------------------------------------
// A prepared class that cannot be a member verdict
// -----------------------------------------------------------------------------------------------

#[test]
fn the_prepared_path_refuses_a_class_whose_member_table_stopped() {
    // Cutting the sample's last bytes leaves the last method record's name and descriptor readable
    // while its attribute list cannot be read, which is a member table that stopped. The exact cut is
    // found by asking the reader, never assumed: the first truncation whose prepared class publishes
    // a stop **and** still locates a member this test asks for.
    let full = SCOPE.to_vec();
    let cut = (1..full.len())
        .rev()
        .find(|length| {
            let truncated = full[..*length].to_vec();
            let mut budget = Budget::new(limits());
            let Ok(snapshot) = ArtifactSnapshot::open(ArtifactInput::bytes(truncated), &mut budget)
            else {
                return false;
            };
            let Ok(read) = snapshot.prepared_root_class(&mut budget) else {
                return false;
            };
            PreparedClass::prepare(&read, &mut budget).is_ok_and(|prepared| {
                prepared.member_table_stop().is_some()
                    && !prepared.locate_method(b"simple", b"()I").is_empty()
            })
        })
        .expect("a truncation of the sample stops its member table inside a readable declaration");

    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(full[..cut].to_vec()), &mut budget)
        .expect("the truncated sample still opens");
    let read = snapshot
        .prepared_root_class(&mut budget)
        .expect("a damaged member table is not a damaged declaration");
    let prepared = match PreparedClass::prepare(&read, &mut budget) {
        Ok(prepared) => prepared,
        Err(error) => panic!("a damaged member table publishes a stop, got {error}"),
    };
    let stop = prepared
        .member_table_stop()
        .expect("the stop is published on the class")
        .clone();
    let sample = Sample {
        definition: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: prepared.class_bytes().digest.clone(),
                length: prepared.class_bytes().length,
            },
            variant: PhysicalVariant::Base,
        },
        snapshot,
        members: Vec::new(),
        root_prefix: Vec::new(),
    };

    // The member this test asks for is in the readable prefix — the locator answered for it — and it
    // is still refused: an incomplete table cannot state that a class declares no member, and it
    // states no body verdict either. The refusal is the stop's own code, not "not found".
    let request = sample.request(b"simple", b"()I");
    let from_prepared = analyze_prepared_method_ir(
        sample.content(),
        &prepared,
        &request,
        &mut Budget::new(limits()),
    )
    .expect("a refusal of the class is a report, not a raised error");
    match &from_prepared.report().execution {
        ExecutionReport::Failed {
            reason: TerminationReason::Error { code },
            ..
        } => assert_eq!(
            code, &stop.code,
            "the refusal is the member-table stop's own code"
        ),
        other => panic!("a stopped member table is a failed run: {other:?}"),
    }
    assert!(
        from_prepared
            .report()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == stop.code),
        "and the diagnostic names it: {:?}",
        from_prepared.report().diagnostics
    );
    assert!(
        !from_prepared.report().reads.is_empty(),
        "the read record survives the refusal: the class was read, wherever it was read"
    );

    // The direct read of the same bytes refuses them as well: a class file whose member table does
    // not read to its end is an error on that path too, and neither path invents a body verdict.
    let direct = analyze_method_ir(sample.content(), &request, &mut Budget::new(limits()))
        .expect("the direct read of the same bytes is a report too");
    assert!(
        !matches!(direct.report().execution, ExecutionReport::Complete { .. }),
        "the direct read refuses these bytes as well: {:?}",
        direct.report().execution
    );
}
