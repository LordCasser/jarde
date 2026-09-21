//! P3 acceptance: the driver class's own declaration facts, handed over from the read that already
//! happened (the `carry-declaring-class-evidence` change).
//!
//! `Engine::recover_method` runs one method-analysis request and presents the payload of that same
//! run. The run's `raw_facts` pass locates the driver member in **one** header read — the read whose
//! bytes also yield the body, the constant pool, the class file's version and its bootstrap table —
//! and that read holds two facts the payload used to drop: the class's own internal name
//! (`this_class`) and the class's own access flags. Without them the recovery layer could not tell a
//! `public` method that is not `abstract` in an **interface** (a `default` method) from the same
//! member in a **class** (an ordinary one), so `declaration@1` refused the fact as missing
//! (`jre_declaration_class_not_in_run`) even though the bytes had already been read.
//!
//! What this file checks, all of it through the entry points a caller has:
//!
//! * the payload's two getters state what an **independent header read of the same committed bytes**
//!   states — the class's internal name and its flags — and the member's own flags are still the
//!   member's, never the class's (a class-level `ACC_INTERFACE`/`ACC_ABSTRACT` may not appear as if
//!   they were the member's);
//! * two physical definitions **of the same internal name** with different class flags do not
//!   cross-use facts: the same committed member, read from a definition whose flags say `interface`
//!   and from one whose flags say `class`, is a `default` method in the first and an instance method
//!   in the second, each bound to its own definition's digest;
//! * the declaration forms the new fixture's own class files declare — an interface's `default` and
//!   `static` method, an `abstract` interface member with no `Code`, a class's constructor, instance
//!   method, static method and `<clinit>` — are all decided through the **public** entry, on the
//!   committed fixtures whose flags say so (the mutation this file is aimed at is the removal of the
//!   facade's adaptation: with it gone, every member below falls back to the missing-fact refusal);
//! * a class name no Java identifier grammar accepts is *displayed* through the existing lossy read
//!   and moves nothing else: the same body, the same quality, the same planes, and the payload's
//!   class-name bytes are still the class file's own;
//! * the paths that publish no declaration fabricate nothing: an `abstract` member with no `Code`, a
//!   header read a budget refused, and a cancelled run all publish no class facts, and the low-level
//!   recovery entry keeps its existing refusal when the facts really are missing;
//! * an ordinary member still charges exactly the reads and bytes it charged before the handoff: one
//!   header attempt, one body attempt and no second scan, checked both by the numbers the run reports
//!   and at source level (the class facts are taken from the read that was already there);
//! * a body that **still** has fallbacks keeps its quality: the declaration diagnostic disappears
//!   from that body's envelope, and its fallbacks, quality, representation and content do not move.
//!
//! The numbers this file pins were measured on the pre-handoff tree as well as after it, member by
//! member; the only field they differ in is `elapsed_millis`, the wall-clock reading every usage
//! snapshot carries and the one field the repository's own fingerprint comparison normalizes away.
//!
//! The same read is what a **prepared** class hands to this pass, so the guards below are extended
//! (never weakened) to cover the two entries that consume one: `read_prepared_driver_method` (bulk
//! task 2.3, driver half) reads no class definition of its own, decodes through the prepared class's
//! own decoder and charges no `ClassHeaders`, and `callee::read_prepared_callees` (the callee half)
//! answers the members it is asked about without a second class read — while both keep the loader
//! binding check and the per-body charges the direct paths make.
//!
//! What this file does **not** prove: that the class facts are correct in general (they are the
//! reader's `this_class` and `access_flags` of one read, and the cross-check below is a second read by
//! the same reader, not an independent parser); that the text the recovery layer is handed can spell
//! every JVM name (the facade's boundary spells the payload's bytes with the lossy Modified-UTF-8 read
//! this layer already uses, so a name it cannot spell shows as the replacement character while the
//! payload's bytes stay exact); or that any of this raises the recovered statement count, which is a
//! property of the bodies and not of the envelope.

use jarde::*;
use jarde_java::{DeclarationForm, DeclarationRecord};
use jarde_jvm::engine::analyze_method_ir;
use std::path::{Path, PathBuf};
use std::slice;

/// The no-debug sample, compiled by javac 23.0.1 `--release 8 -g:none` (see the fixture's README).
const SCOPE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");
/// The handled-sample fixture, whose members carry the guarded shapes and a `<clinit>`.
const GUARDED: &[u8] = include_bytes!("fixtures/p3-handlers/v8/Guarded.class");
/// The declaration fixture's interface: `default`/`static`/`abstract` members side by side.
const SHAPE: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Shape.class");
/// The declaration fixture's class: a constructor, an instance method, a static method, a `<clinit>`.
const HOLDER: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Holder.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
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

/// One opened sample and what an **independent** header read of its bytes states.
///
/// The header read here is a second, separate read through the reader's own entry point
/// ([`Engine::inspect_header`]), under its own budget: it is what the payload's facts are compared
/// against, so "the getters reflect the real Header" is a comparison between the handoff and the
/// bytes rather than a restatement of the payload. It is the same reader, so what it proves is that
/// the *handoff* carried the read's facts — not that the reader's parse of `this_class` is right
/// (that is P1's acceptance, over its own fixtures).
struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
    /// The class's own access flags, as that independent read states them.
    class_flags: u16,
    /// The class's own internal name (`this_class`), as that independent read states it.
    class_name: Vec<u8>,
    /// The class's declared members: name, descriptor, access flags and whether a `Code` shell is
    /// present, in declaration order.
    members: Vec<(Vec<u8>, Vec<u8>, u16, bool)>,
}

impl Fixture {
    /// One declared member as the independent header read states it.
    fn member(&self, name: &[u8], descriptor: &[u8]) -> (u16, bool) {
        self.members
            .iter()
            .find(|(declared, declared_descriptor, _, _)| {
                declared.as_slice() == name && declared_descriptor.as_slice() == descriptor
            })
            .map(|(_, _, flags, has_code)| (*flags, *has_code))
            .unwrap_or_else(|| {
                panic!(
                    "the fixture declares `{}` `{}`: {:?}",
                    String::from_utf8_lossy(name),
                    String::from_utf8_lossy(descriptor),
                    self.members
                        .iter()
                        .map(|(name, descriptor, _, _)| format!(
                            "{}{}",
                            String::from_utf8_lossy(name),
                            String::from_utf8_lossy(descriptor)
                        ))
                        .collect::<Vec<_>>()
                )
            })
    }
}

fn fixture(engine: &Engine, bytes: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let header = &inspected.inspection.header;
    Fixture {
        snapshot,
        class_bytes: inspected.source.class_bytes.clone(),
        class_flags: header.access_flags,
        class_name: header.this_class.raw().0.clone(),
        members: header
            .methods
            .iter()
            .map(|member| {
                (
                    member.name.raw().0.clone(),
                    member.descriptor.raw().0.clone(),
                    member.access_flags,
                    member
                        .attributes
                        .iter()
                        .any(|shell| shell.name.raw().0.as_slice() == b"Code"),
                )
            })
            .collect(),
    }
}

fn recover(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> RecoveredMethod {
    let request = request_of(fixture, name, descriptor);
    let mut budget = Budget::new(limits());
    engine
        .recover_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised")
}

fn request_of(fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(&fixture.snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: fixture.snapshot.id().clone(),
                },
                class_bytes: fixture.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

fn usage(recovered: &RecoveredMethod) -> UsageSnapshot {
    match &recovered.analysis().execution {
        ExecutionReport::Complete { usage } => usage.clone(),
        other => panic!("a legal request completes: {other:?}"),
    }
}

/// One recovery run under a caller-supplied budget: the cancellation and bound cases below need the
/// run to be the one that stops, not a raised error.
fn recover_with(
    engine: &Engine,
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
    stages: Vec<AnalysisStage>,
    budget: &mut Budget,
) -> RecoveredMethod {
    let mut request = request_of(fixture, name, descriptor);
    request.stages = stages;
    engine
        .recover_method(slice::from_ref(&fixture.snapshot), &request, budget)
        .expect("a stopped run is an answer, not a raised error")
}

/// The payload of one request, through the IR entry the facade's own recovery call composes.
///
/// This is the one place this file reads a payload directly: the low-level entry is what the
/// getters' own contract is written for, and the public-entry cases below go through
/// [`Engine::recover_method`] instead.
fn ir_of(
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
) -> jarde_jvm::method_ir::MethodIrAnalysis {
    let request = request_of(fixture, name, descriptor);
    let mut budget = Budget::new(limits());
    analyze_method_ir(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("the fixture's own member is readable")
}

/// The declaration record of a produced run, which the report always writes.
fn declaration(report: &RecoveryReport) -> &DeclarationRecord {
    report
        .declaration
        .as_ref()
        .expect("a run that reached a payload states what it read of the declaration")
}

/// The rules one run named, in the order it named them.
fn named_rules(report: &RecoveryReport) -> Vec<String> {
    report.rules.iter().map(|rule| rule.citation()).collect()
}

/// Every diagnostic code one run published, in order.
fn diagnostic_codes(report: &RecoveryReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// The offset of the class file's own `access_flags` (JVMS 4.1).
///
/// They sit *after* the constant pool, so the offset is found by walking the pool's entries by their
/// own tag sizes — not by a fixed guess: `magic`, `minor_version`, `major_version` and
/// `constant_pool_count` are the first ten bytes, and every entry states its own length (a
/// `long`/`double` entry also takes two pool slots, and no other tag does).
///
/// This is deliberately not a second parser: it reads no name, no descriptor and no value, and every
/// case that patches the flags first checks that the two bytes this walk lands on equal the flags the
/// reader's own header read states for the same bytes ([`Fixture::class_flags`]), so a wrong walk is a
/// failing assertion rather than a silent corruption.
fn access_flags_offset(bytes: &[u8]) -> usize {
    let count = usize::from(u16::from_be_bytes([bytes[8], bytes[9]]));
    let mut at = 10usize;
    let mut index = 1usize;
    while index < count {
        let tag = bytes[at];
        at += 1;
        let width = match tag {
            1 => {
                let length = usize::from(u16::from_be_bytes([bytes[at], bytes[at + 1]]));
                at += 2 + length;
                index += 1;
                continue;
            }
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => 4,
            7 | 8 | 16 | 19 | 20 => 2,
            15 => 3,
            5 | 6 => {
                // `CONSTANT_Long`/`CONSTANT_Double` take two pool slots (JVMS 4.4.5).
                index += 1;
                8
            }
            other => {
                panic!("constant pool tag {other} at entry {index} is not one this walk knows")
            }
        };
        at += width;
        index += 1;
    }
    at
}

/// The same class file with its own `access_flags` replaced; every other byte, and therefore every
/// offset, is the sample's.
fn with_class_flags(bytes: &[u8], flags: u16) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    let at = access_flags_offset(bytes);
    patched[at..at + 2].copy_from_slice(&flags.to_be_bytes());
    patched
}

/// One byte sequence replaced by another of the same length, with the number of sites rewritten.
///
/// The length is part of the contract: a patch that moved a byte would move every constant-pool index
/// and every BCI in the file, and the case would no longer be about one name.
fn patched(bytes: &[u8], needle: &[u8], replacement: &[u8]) -> (Vec<u8>, usize) {
    assert_eq!(
        needle.len(),
        replacement.len(),
        "a patch keeps every offset: {needle:?} -> {replacement:?}"
    );
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0usize;
    let mut sites = 0usize;
    while at < bytes.len() {
        if bytes[at..].starts_with(needle) {
            out.extend_from_slice(replacement);
            at += needle.len();
            sites += 1;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    (out, sites)
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read_repository_file(relative: &str) -> String {
    let path = repository_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{relative} is unreadable at {}: {error}", path.display()))
}

// -------------------------------------------------------------------------------------------
// 1.1: the payload's class facts are the header read's own, and they are the class's, not the
// member's
// -------------------------------------------------------------------------------------------

#[test]
fn the_payload_states_the_declaring_class_facts_the_same_header_read_holds() {
    let engine = Engine::new();
    // Two class kinds and two member kinds, so "class flags are not member flags" is checked in both
    // directions: an interface's `static` method and a class's instance method.
    for (label, bytes, name, descriptor, member_static) in [
        (
            "interface",
            SHAPE,
            b"sum".as_slice(),
            b"(II)I".as_slice(),
            true,
        ),
        (
            "class",
            HOLDER,
            b"value".as_slice(),
            b"()I".as_slice(),
            false,
        ),
    ] {
        let fixture = fixture(&engine, bytes);
        let (member_flags, member_has_code) = fixture.member(name, descriptor);
        assert!(member_has_code, "{label}: the member read here has a body");
        assert_eq!(
            member_flags & ACC_STATIC != 0,
            member_static,
            "{label}: the member's own flags are the premise of this case"
        );

        let analyzed = ir_of(&fixture, name, descriptor);
        let declaration = analyzed
            .ir()
            .declaration()
            .expect("the run located the member in its own header read");

        // The two class facts are what an independent read of the same bytes states.
        assert_eq!(
            declaration.class_name().0.as_slice(),
            fixture.class_name.as_slice(),
            "{label}: the payload's class name is the read's own `this_class`"
        );
        assert_eq!(
            declaration.class_access_flags(),
            fixture.class_flags,
            "{label}: the payload's class flags are the read's own class flags"
        );
        // And the member's declaration is still the member's.
        assert_eq!(
            declaration.access_flags(),
            member_flags,
            "{label}: the member's flags are the member's"
        );
        assert_eq!(
            declaration.name().0.as_slice(),
            name,
            "{label}: the member's name is the member's"
        );
        assert_eq!(
            declaration.descriptor().0.as_slice(),
            descriptor,
            "{label}: the member's descriptor is the member's"
        );

        // The class facts are bound to the physical definition the request read, by digest.
        assert_eq!(
            declaration.identity().owner.class_bytes,
            fixture.class_bytes,
            "{label}: the declaration is bound to the definition this request read"
        );

        // Class flags and member flags are never confused: the class's interface/abstract bits are on
        // the class, the member's own `static`/`abstract` bits are on the member, and neither side
        // carries the other's.
        let class_is_interface = fixture.class_flags & ACC_INTERFACE != 0;
        assert_eq!(
            declaration.class_access_flags() & ACC_INTERFACE != 0,
            class_is_interface,
            "{label}: `ACC_INTERFACE` is read off the class"
        );
        assert_eq!(
            declaration.access_flags() & ACC_INTERFACE,
            0,
            "{label}: and it is not copied onto the member"
        );
        assert_eq!(
            declaration.access_flags() & ACC_ABSTRACT,
            member_flags & ACC_ABSTRACT,
            "{label}: the member's `abstract` bit is the member's own"
        );
        assert_eq!(
            declaration.class_access_flags() & ACC_STATIC,
            0,
            "{label}: a class is never `static`; only the member may be"
        );
        if class_is_interface {
            assert_eq!(
                declaration.class_access_flags() & ACC_ABSTRACT,
                ACC_ABSTRACT,
                "{label}: JVMS 4.1 makes an interface `ACC_ABSTRACT` in the class's own flags"
            );
            assert_eq!(
                declaration.access_flags() & ACC_ABSTRACT,
                0,
                "{label}: while this member is not (`sum` declares a body)"
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// 1.1: two physical definitions of one internal name do not cross-use their class facts
// -------------------------------------------------------------------------------------------

#[test]
fn two_definitions_of_the_same_internal_name_keep_their_own_class_facts() {
    let engine = Engine::new();
    // The same committed class file twice: once as javac wrote it, once with its own `access_flags`
    // replaced by an interface's. Both declare the internal name `Holder` and the same member
    // `value()I` with the same bytes — only the class's own kind differs.
    let class = fixture(&engine, HOLDER);
    let at = access_flags_offset(HOLDER);
    assert_eq!(
        u16::from_be_bytes([HOLDER[at], HOLDER[at + 1]]),
        class.class_flags,
        "the walk lands on the class's own `access_flags`, as the reader's header read states them"
    );
    assert_ne!(
        at,
        access_flags_offset(SHAPE),
        "the walk really is a walk: the two samples hold different pools, so their flags do not sit \
         at one fixed offset"
    );
    let interface_bytes = with_class_flags(HOLDER, ACC_PUBLIC | ACC_INTERFACE | ACC_ABSTRACT);
    let interface = fixture(&engine, &interface_bytes);

    assert_eq!(
        interface.class_name, class.class_name,
        "the two definitions declare the same internal name, which is the trap this case is about"
    );
    assert_ne!(
        interface.class_flags, class.class_flags,
        "and different class flags"
    );
    assert_ne!(
        interface.class_bytes.digest, class.class_bytes.digest,
        "they are two physical definitions, not one definition read twice"
    );

    let in_class = recover(&engine, &class, b"value", b"()I");
    let in_interface = recover(&engine, &interface, b"value", b"()I");
    let class_declaration = declaration(in_class.recovery());
    let interface_declaration = declaration(in_interface.recovery());

    assert_eq!(
        class_declaration.declaring_class.as_deref(),
        Some("Holder"),
        "both definitions spell the same name"
    );
    assert_eq!(
        interface_declaration.declaring_class.as_deref(),
        Some("Holder")
    );
    assert_eq!(
        class_declaration.interface,
        Some(false),
        "the class's own flags say it is a class"
    );
    assert_eq!(
        interface_declaration.interface,
        Some(true),
        "and the other definition's flags say it is an interface"
    );
    assert_eq!(
        class_declaration.form,
        Some(DeclarationForm::InstanceMethod),
        "`public`, not `static`: an ordinary instance method in a class"
    );
    assert_eq!(
        interface_declaration.form,
        Some(DeclarationForm::DefaultMethod),
        "and an interface's `default` method — the same member, the same bytes, the other definition"
    );

    // Each request's facts come from its own definition, by digest: a name-keyed shortcut (or a
    // cache keyed by the name the two share) would give both runs the same answer.
    for (fixture, name, descriptor) in [
        (&class, b"value".as_slice(), b"()I".as_slice()),
        (&interface, b"value".as_slice(), b"()I".as_slice()),
    ] {
        let analyzed = ir_of(fixture, name, descriptor);
        let payload = analyzed
            .ir()
            .declaration()
            .expect("the member is declared by its own definition");
        assert_eq!(
            payload.identity().owner.class_bytes,
            fixture.class_bytes,
            "the payload's definition is the one this request read"
        );
        assert_eq!(
            payload.class_access_flags(),
            fixture.class_flags,
            "and its class flags are that definition's"
        );
    }
}

// -------------------------------------------------------------------------------------------
// 1.2: every member form, through the public entry (the mutation this file is aimed at)
// -------------------------------------------------------------------------------------------

#[test]
fn the_public_entry_decides_every_member_form_from_the_runs_own_class_facts() {
    let engine = Engine::new();
    let scope = fixture(&engine, SCOPE);
    let guarded = fixture(&engine, GUARDED);
    let shape = fixture(&engine, SHAPE);
    let holder = fixture(&engine, HOLDER);

    // The four forms the acceptance names, plus the ordinary instance/static pair taken from a
    // fixture that is not about declarations at all. Every row is the *public* entry: the run's
    // report and the presentation of the same run.
    for (label, fixture, name, descriptor, declaring_class, is_interface, form, phrase) in [
        (
            "a class's instance method",
            &scope,
            b"receiver".as_slice(),
            b"(J)J".as_slice(),
            "Scope",
            false,
            DeclarationForm::InstanceMethod,
            "an instance method",
        ),
        (
            "a class's static method",
            &scope,
            b"simple".as_slice(),
            b"()I".as_slice(),
            "Scope",
            false,
            DeclarationForm::StaticMethod,
            "a static method",
        ),
        (
            "a class's constructor",
            &scope,
            b"<init>".as_slice(),
            b"()V".as_slice(),
            "Scope",
            false,
            DeclarationForm::Constructor,
            "a constructor",
        ),
        (
            "a class's `<clinit>`",
            &guarded,
            b"<clinit>".as_slice(),
            b"()V".as_slice(),
            "Guarded",
            false,
            DeclarationForm::StaticInitializer,
            "a static initializer",
        ),
        (
            "an interface's `default` method",
            &shape,
            b"scaled".as_slice(),
            b"(I)I".as_slice(),
            "Shape",
            true,
            DeclarationForm::DefaultMethod,
            "an interface's default method",
        ),
        (
            "an interface's `static` method",
            &shape,
            b"sum".as_slice(),
            b"(II)I".as_slice(),
            "Shape",
            true,
            DeclarationForm::StaticInterfaceMethod,
            "an interface's static method",
        ),
        (
            "a class's second constructor",
            &holder,
            b"<init>".as_slice(),
            b"(I)V".as_slice(),
            "Holder",
            false,
            DeclarationForm::Constructor,
            "a constructor",
        ),
        (
            "a class's instance method",
            &holder,
            b"value".as_slice(),
            b"()I".as_slice(),
            "Holder",
            false,
            DeclarationForm::InstanceMethod,
            "an instance method",
        ),
        (
            "a class's static method",
            &holder,
            b"of".as_slice(),
            b"(I)LHolder;".as_slice(),
            "Holder",
            false,
            DeclarationForm::StaticMethod,
            "a static method",
        ),
        (
            "a class's `<clinit>`",
            &holder,
            b"<clinit>".as_slice(),
            b"()V".as_slice(),
            "Holder",
            false,
            DeclarationForm::StaticInitializer,
            "a static initializer",
        ),
    ] {
        let recovered = recover(&engine, fixture, name, descriptor);
        let report = recovered.recovery();
        assert!(report.produced(), "{label}: {:?}", report.outcome);
        let record = declaration(report);
        assert_eq!(
            record.declaring_class.as_deref(),
            Some(declaring_class),
            "{label}: the class the run's own read declares"
        );
        assert_eq!(
            record.interface,
            Some(is_interface),
            "{label}: and whether it is an interface"
        );
        assert_eq!(record.form, Some(form), "{label}");
        assert!(
            record.presented(),
            "{label}: the envelope states the declaration"
        );
        assert_eq!(
            record.member_flags,
            Some(fixture.member(name, descriptor).0),
            "{label}: the member flags are the ones the class declares"
        );
        assert!(
            record.refusal.is_none(),
            "{label}: nothing was refused: {:?}",
            record.refusal
        );
        assert!(
            !diagnostic_codes(report).contains(&"jre_declaration_class_not_in_run"),
            "{label}: the missing-class diagnostic is gone: {:?}",
            diagnostic_codes(report)
        );
        assert!(
            report
                .text
                .contains(&format!("// @declaration {phrase} of `{declaring_class}`")),
            "{label}: the artifact's envelope states it: {}",
            report.text
        );
        assert!(
            named_rules(report).contains(&"declaration@1".to_string()),
            "{label}: the rule that wrote the line is named: {:?}",
            named_rules(report)
        );
    }
}

// -------------------------------------------------------------------------------------------
// 1.1/2.1: a name no Java grammar accepts is displayed, and moves nothing else
// -------------------------------------------------------------------------------------------

#[test]
fn a_mangled_class_name_is_displayed_and_moves_nothing_but_the_display() {
    let engine = Engine::new();
    let control = recover(&engine, &fixture(&engine, SHAPE), b"sum", b"(II)I");

    // The class's own name, replaced in the one constant-pool UTF-8 entry `this_class` resolves
    // through: a leading digit and a `/` that is not a package separator a source could declare.
    let (mangled_bytes, sites) = patched(SHAPE, b"\x01\x00\x05Shape", b"\x01\x00\x051/hap");
    assert_eq!(sites, 1, "the class's own name is one pool entry");
    let mangled = fixture(&engine, &mangled_bytes);
    assert_eq!(mangled.class_name, b"1/hap".to_vec());
    assert_eq!(
        mangled.class_flags,
        fixture(&engine, SHAPE).class_flags,
        "the patch moves the name and nothing else"
    );

    let analyzed = ir_of(&mangled, b"sum", b"(II)I");
    let payload = analyzed
        .ir()
        .declaration()
        .expect("the member is still located in the same read");
    assert_eq!(
        payload.class_name().0,
        b"1/hap".to_vec(),
        "the payload keeps the class file's own bytes, not a spelling of them"
    );
    assert_eq!(
        payload.class_access_flags(),
        mangled.class_flags,
        "and the flags are unaffected by the name"
    );

    let recovered = recover(&engine, &mangled, b"sum", b"(II)I");
    let report = recovered.recovery();
    let record = declaration(report);
    assert_eq!(
        record.declaring_class.as_deref(),
        Some("1/hap"),
        "the record states the name as the lossy text this boundary spells"
    );
    assert_eq!(
        record.form,
        Some(DeclarationForm::StaticInterfaceMethod),
        "the form is read from the flags, not from the name"
    );
    assert!(
        report
            .text
            .contains("an interface's static method of `1.hap`"),
        "the artifact displays it, with `/` spelled the way this layer writes a class name: {}",
        report.text
    );

    // What the name may not do: move the body's planes, or make the run claim the text is legal Java
    // because the name reached an envelope. The two runs differ in the spelling and in nothing else.
    let control_report = control.recovery();
    assert_eq!(report.quality, control_report.quality, "quality");
    assert_eq!(
        report.representation, control_report.representation,
        "representation"
    );
    assert_eq!(
        report.syntax_status, control_report.syntax_status,
        "syntax status"
    );
    assert_eq!(report.content, control_report.content, "content");
    assert_eq!(
        report.compile_status, control_report.compile_status,
        "compile status"
    );
    assert_eq!(
        report.semantic_validation, control_report.semantic_validation,
        "semantic validation"
    );
    assert_eq!(
        report.verification, control_report.verification,
        "verification"
    );
    assert_eq!(
        diagnostic_codes(report),
        diagnostic_codes(control_report),
        "no diagnostic is invented for the name"
    );
    assert_eq!(
        report.text.replace("1.hap", "Shape"),
        control_report.text,
        "the text is the control's with the class's spelling replaced, and nothing else"
    );
}

// -------------------------------------------------------------------------------------------
// 2.1: the paths that publish no declaration fabricate nothing
// -------------------------------------------------------------------------------------------

#[test]
fn a_stopped_run_publishes_no_declaration_and_no_class_facts() {
    let engine = Engine::new();
    let shape = fixture(&engine, SHAPE);
    let holder = fixture(&engine, HOLDER);

    // (a) An `abstract` interface member: the class file declares no `Code` for it, so there is no
    // body to present. The run still read the header — and still states no declaration from it, nor
    // any class, because `raw_facts` publishes a declaration only for a member it located with a body.
    let analyzed = ir_of(&shape, b"sides", b"()I");
    assert!(
        matches!(
            analyzed.report().body,
            MethodBodyState::DeclaredWithoutBody { .. }
        ),
        "the member's own declaration says it has no body: {:?}",
        analyzed.report().body
    );
    assert!(
        analyzed.ir().declaration().is_none(),
        "a run that located no member with a body states no declaration"
    );
    let recovered = recover(&engine, &shape, b"sides", b"()I");
    let report = recovered.recovery();
    assert!(
        matches!(
            report.outcome,
            RecoveryOutcome::Stopped(StopReason::IrTableMissing { table: "canonical" })
        ),
        "{:?}",
        report.outcome
    );
    assert!(
        report.declaration.is_none(),
        "and the recovery layer states no declaration of its own: {:?}",
        report.declaration
    );
    assert_eq!(report.text, "");
    assert_eq!(report.content, RecoveryContent::NotProduced);
    let stopped_usage = usage(&recovered);
    assert_eq!(stopped_usage.class_headers, 1, "the header really was read");
    assert_eq!(stopped_usage.method_bodies, 0, "and no body was attempted");
    assert_eq!(
        stopped_usage.code_bytes, 0,
        "no instruction was decoded, because there is no body"
    );
    assert_eq!(stopped_usage.ir_items, 0, "and no IR was built from one");
    assert!(
        stopped_usage.read_bytes > 0,
        "the read is charged: {stopped_usage:?}"
    );

    // (a2) The same member declared `native` instead of `abstract`: the other flag JVMS 4.6 lets a
    // member carry without a `Code` attribute. The needle is the member's own flags, name index,
    // descriptor index and attribute count, so the case only applies to a member that really is the
    // `public abstract` one this sample declares — and the patched member is the other kind of
    // no-body declaration, with no class facts published for it either.
    let (native_bytes, sites) = patched(
        SHAPE,
        b"\x04\x01\x00\x05\x00\x06\x00\x00",
        b"\x01\x01\x00\x05\x00\x06\x00\x00",
    );
    assert_eq!(sites, 1, "the member this case patches is declared once");
    let native = fixture(&engine, &native_bytes);
    assert_eq!(
        native.member(b"sides", b"()I"),
        (0x0101, false),
        "the patched member is `public native` and still declares no `Code` shell"
    );
    let analyzed = ir_of(&native, b"sides", b"()I");
    assert!(
        matches!(
            analyzed.report().body,
            MethodBodyState::DeclaredWithoutBody {
                no_body_kind: NoBodyKind::Native
            }
        ),
        "the run states the kind the patched flags declare: {:?}",
        analyzed.report().body
    );
    assert!(
        analyzed.ir().declaration().is_none(),
        "a native member states no declaration either"
    );
    let recovered = recover(&engine, &native, b"sides", b"()I");
    assert_eq!(recovered.recovery().declaration, None);
    assert_eq!(recovered.recovery().text, "");
    assert_eq!(
        usage(&recovered).method_bodies,
        0,
        "and no body was attempted for it"
    );

    // (b) A budget that refuses the header read: nothing was read, so nothing is stated.
    let mut refused = Budget::new(Limits {
        class_headers: 0,
        ..limits()
    });
    let recovered = recover_with(
        &engine,
        &holder,
        b"value",
        b"()I",
        AnalysisStage::ALL.to_vec(),
        &mut refused,
    );
    assert!(
        matches!(
            recovered.analysis().execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ClassHeaders,
                },
                ..
            }
        ),
        "the run stopped at the header attempt: {:?}",
        recovered.analysis().execution
    );
    assert!(
        recovered.analysis().reads.is_empty(),
        "a refused charge records no read: {:?}",
        recovered.analysis().reads
    );
    assert!(recovered.recovery().declaration.is_none());
    assert_eq!(recovered.recovery().text, "");
    assert_eq!(RecoveryContent::NotProduced, recovered.recovery().content);

    // (c) A cancelled run: same shape, and nothing is invented from the request's own member.
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token);
    let recovered = recover_with(
        &engine,
        &holder,
        b"value",
        b"()I",
        AnalysisStage::ALL.to_vec(),
        &mut cancelled,
    );
    assert!(
        matches!(
            recovered.analysis().execution,
            ExecutionReport::Cancelled { .. }
        ),
        "{:?}",
        recovered.analysis().execution
    );
    assert!(recovered.recovery().declaration.is_none());
    assert_eq!(recovered.recovery().text, "");
    assert_eq!(
        recovered.analysis().reads.len(),
        0,
        "a cancelled run published no read"
    );

    // (d) The low-level entry with facts that state no class: the payload below *does* hold the class
    // facts, and the recovery layer still refuses them as missing when its caller states none — the
    // refusal this change removes from the facade path is untouched for a caller that really has no
    // evidence.
    let analyzed = ir_of(&holder, b"value", b"()I");
    assert!(
        analyzed.ir().declaration().is_some(),
        "the payload holds them"
    );
    let request = request_of(&holder, b"value", b"()I");
    let facts =
        RecoveryFacts::new(MethodFacts::new("value", "()I", 1).with_access_flags(ACC_PUBLIC));
    let mut budget = Budget::new(limits());
    let low = jarde_java::recover(
        &jarde_java::RecoveryRequest::new(
            analyzed.ir(),
            &facts,
            request.environment.runtime.profile.clone(),
        ),
        &mut budget,
    );
    let record = declaration(&low);
    assert!(
        !record.presented(),
        "the low-level run states no declaration"
    );
    assert_eq!(record.form, None);
    assert_eq!(
        record.refusal.as_ref().map(|refusal| refusal.code),
        Some("jre_declaration_class_not_in_run"),
        "the existing refusal, with the requirement it fell short of: {:?}",
        record.refusal
    );
    assert!(
        !low.text.contains("@declaration"),
        "and no envelope line is written: {}",
        low.text
    );
}

// -------------------------------------------------------------------------------------------
// 2.1: an ordinary member still charges the same reads, and no second scan happens
// -------------------------------------------------------------------------------------------

#[test]
fn an_ordinary_member_charges_the_same_reads_and_bytes_as_before_the_handoff() {
    // The two members of the `p3-scope` sample that name no callee at all, so nothing but the driver
    // request itself can account for a read. The numbers below are the *deterministic* usage fields of
    // the run, and they are the numbers the pre-handoff tree charged for the same requests, verbatim:
    // the handoff adds two fields to a value that was already read, and no read, decode, pass or item.
    // (`elapsed_millis` is the one field left out: it is the wall-clock reading every usage snapshot
    // carries, and the repository's fingerprint comparison normalizes it away for the same reason.)
    let engine = Engine::new();
    let scope = fixture(&engine, SCOPE);
    for (label, name, descriptor, expected) in [
        (
            "an instance member",
            b"receiver".as_slice(),
            b"(J)J".as_slice(),
            (532, 532, 275, 2, 51, 2, 16),
        ),
        (
            "a static member",
            b"simple".as_slice(),
            b"()I".as_slice(),
            (532, 532, 277, 4, 53, 3, 24),
        ),
    ] {
        let recovered = recover(&engine, &scope, name, descriptor);
        let report = recovered.recovery();
        assert!(report.produced(), "{label}: {:?}", report.outcome);
        assert!(declaration(report).presented(), "{label}");
        let usage = usage(&recovered);
        let (
            expected_read_bytes,
            expected_class_bytes,
            expected_attribute_bytes,
            expected_code_bytes,
            expected_ir_items,
            expected_ir_edges,
            expected_steps,
        ) = expected;
        assert_eq!(
            (
                usage.read_bytes,
                usage.class_bytes,
                usage.attribute_bytes,
                usage.code_bytes,
                usage.ir_items,
                usage.ir_edges,
                usage.analysis_steps,
            ),
            (
                expected_read_bytes,
                expected_class_bytes,
                expected_attribute_bytes,
                expected_code_bytes,
                expected_ir_items,
                expected_ir_edges,
                expected_steps,
            ),
            "{label}: the run's own usage, as the pre-handoff tree charged it"
        );
        // A16's shape: one header attempt, one body attempt, one read — whatever the class declares.
        assert_eq!(usage.class_headers, 1, "{label}");
        assert_eq!(usage.method_bodies, 1, "{label}");
        assert_eq!(usage.entry_bytes, 0, "{label}: no entry was materialized");
        assert_eq!(usage.result_items, 0, "{label}");
        assert_eq!(
            recovered
                .analysis()
                .reads
                .iter()
                .map(|read| read.reason)
                .collect::<Vec<_>>(),
            vec![ReadReason::DriverMethodBody],
            "{label}: the request read one thing, the driver member's own body"
        );
        assert!(
            usage.output_bytes > 0,
            "{label}: and the artifact was written"
        );
    }
}

#[test]
fn the_class_facts_are_taken_from_the_read_that_was_already_there() {
    // The numbers above can only show that this change added no charge on the members they run. This
    // is the other half: where the two fields come from in the pass's source, and that the pass still
    // performs one header read and one body decode — a second read, or a value derived from the
    // request's owner spelling, is a change this guard fails on.
    let source = read_repository_file("crates/jarde-jvm/src/engine.rs");
    let start = source
        .find("fn read_driver_method(")
        .expect("`raw_facts`'s one read is in this file");
    let rest = &source[start..];
    let end = rest
        .find("\n/// Whether the member declares a `Code` attribute")
        .expect("the pass ends where the next helper begins");
    let pass = &rest[..end];
    assert_eq!(
        pass.matches("HeaderDemand::DriverMethodBody").count(),
        1,
        "the pass performs exactly one header read"
    );
    assert_eq!(
        pass.matches("method_code_facts(").count(),
        1,
        "and decodes exactly one body"
    );
    assert_eq!(
        pass.matches("class_facts(").count(),
        0,
        "it parses no second header of its own"
    );
    assert!(
        pass.contains("read.header.facts.this_class.raw().clone(),")
            && pass.contains("read.header.facts.access_flags,"),
        "the class's name and flags are read off the header facts this pass already holds"
    );
    assert!(
        pass.contains("request.method.clone(),"),
        "while the physical identity stays the request's own definition"
    );
}

#[test]
fn the_prepared_driver_read_consumes_the_class_the_caller_already_read() {
    // The other half of the same pass (bulk task 2.3): a class the caller prepared
    // (`jarde_reader::prepared`) is read once, and this entry must consume that read instead of
    // opening a second one. The guard is at source level for the same reason the one above is: the
    // numbers of a run cannot tell "one class read" from "two reads of the same class", because both
    // report the same physical facts.
    let source = read_repository_file("crates/jarde-jvm/src/engine.rs");
    let start = source
        .find("fn read_prepared_driver_method(")
        .expect("the prepared driver pass is in this file");
    let rest = &source[start..];
    let end = rest
        .find("\n/// The termination and diagnostic of a reader")
        .expect("the prepared pass ends where the next helper begins");
    let pass = &rest[..end];
    assert_eq!(
        pass.matches("read_own_definition").count()
            + pass.matches("read_definition_content").count(),
        0,
        "the prepared pass reads no class definition: the caller's preparation is the read"
    );
    assert_eq!(
        pass.matches("method_code_facts(").count(),
        0,
        "and builds no second decoder"
    );
    assert_eq!(
        pass.matches("prepared.method_code(").count(),
        1,
        "it decodes each body through the prepared class's own decoder, the one the single-method \
         path delegates to"
    );
    assert_eq!(
        pass.matches("CountedBudgetDimension::ClassHeaders").count(),
        0,
        "the class read the preparation paid for is not charged again per method"
    );
    assert_eq!(
        pass.matches("CountedBudgetDimension::MethodBodies").count(),
        1,
        "while the one body attempt of the request is still charged, before its decode"
    );
    assert!(
        pass.contains("require_prepared_definition(prepared, &request.method.owner)"),
        "the prepared class has to be the definition the request names"
    );
    assert!(
        pass.contains("bind_definition("),
        "and the loader binding check of the direct read is not skipped"
    );
    assert!(
        pass.contains("finish_driver_read("),
        "the tail of the read — the declaration, the pool, the bootstrap table and the member's own \
         statement — is the same one the direct read uses"
    );
    // And the prepared lifecycle stays the caller's: the crate that consumes a prepared class never
    // prepares one and never reaches the reader's own prepared accessors, so no prepared class is
    // built, stored, memoized or handed on inside the consuming layer — it can only be borrowed from
    // its caller for the duration of the call.
    for path in [
        "crates/jarde-jvm/src/engine.rs",
        "crates/jarde-jvm/src/callee.rs",
    ] {
        let consuming = read_repository_file(path);
        assert_eq!(
            consuming.matches("PreparedClass::prepare").count()
                + consuming.matches("prepared_root_class").count()
                + consuming.matches("prepared_class(").count(),
            0,
            "{path} builds or re-reads a prepared class: the class lifecycle is the caller's"
        );
    }
}

#[test]
fn the_prepared_callee_read_needs_no_second_class_read() {
    // The callee half (bulk task 2.3): the class's own members a presented body's accessor call
    // sites named, answered from the prepared class the same run was presented from — no
    // `HeaderClosure::read_own_definition`, no `ClassHeaders`, and the same candidate loop, refusals
    // and per-body charge the direct entry has.
    let source = read_repository_file("crates/jarde-jvm/src/callee.rs");
    let start = source
        .find("pub fn read_prepared_callees(")
        .expect("the prepared callee read is in this file");
    let rest = &source[start..];
    let end = rest
        .find("\n/// The class one callee read answers from")
        .expect("the prepared read ends where the shared candidate loop begins");
    let pass = &rest[..end];
    assert_eq!(
        pass.matches("read_own_definition").count(),
        0,
        "the prepared callee read opens no header read of its own"
    );
    assert_eq!(
        pass.matches("CountedBudgetDimension::ClassHeaders").count(),
        0,
        "and charges no class header for the definition the preparation read"
    );
    assert_eq!(
        pass.matches("require_prepared_member_table(").count(),
        1,
        "a member table that stopped refuses the read instead of answering from its prefix"
    );
    assert_eq!(
        pass.matches("bind_definition(").count(),
        1,
        "and the loader binding check of the direct read is not skipped"
    );
    assert_eq!(
        pass.matches("callee_members(").count(),
        1,
        "the candidate loop, its refusals and its charges are the shared ones"
    );
    // One charge and one decoder per source, in the whole file: the shared loop cannot grow a second
    // one without failing here.
    assert_eq!(
        source
            .matches("budget.charge(CountedBudgetDimension::MethodBodies, 1)")
            .count(),
        1,
        "one pre-decode body attempt for both sources of a callee read"
    );
    assert_eq!(
        source.matches("classfile::method_code_facts(").count(),
        1,
        "one decode for a class this entry read itself"
    );
    assert_eq!(
        source.matches("prepared.method_code(").count(),
        1,
        "and one for a class the caller prepared, which is the same implementation"
    );
}

// -------------------------------------------------------------------------------------------
// 2.1: a body that still has fallbacks keeps its quality
// -------------------------------------------------------------------------------------------

#[test]
fn a_body_that_still_has_fallbacks_keeps_its_quality() {
    // `Guarded.fin()V` is the two-`finally`-copies shape the region rules refuse. Before the handoff
    // its envelope carried the missing-class diagnostic; after it, the envelope states the member and
    // the body's own verdict is exactly what it was — this is the property that keeps the change from
    // being a quality claim about the recovery itself.
    let engine = Engine::new();
    let guarded = fixture(&engine, GUARDED);
    let recovered = recover(&engine, &guarded, b"fin", b"()V");
    let report = recovered.recovery();
    assert!(report.produced(), "{:?}", report.outcome);
    let record = declaration(report);
    assert!(
        record.presented(),
        "the declaration is what the handoff added: {:?}",
        record.refusal
    );
    assert_eq!(record.form, Some(DeclarationForm::StaticMethod));

    assert_eq!(report.quality, Quality::Fallback, "quality, unchanged");
    assert_eq!(
        report.representation,
        Representation::Mixed,
        "representation, unchanged"
    );
    assert_eq!(
        report.content,
        RecoveryContent::ExplanationOnly,
        "content, unchanged"
    );
    assert_eq!(
        report.syntax_status,
        SyntaxStatus::NotJava,
        "syntax status, unchanged"
    );
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
    assert_eq!(report.verification, VerificationStatus::NotPerformed);
    assert!(
        !report.fallbacks.is_empty(),
        "the body still has fallbacks: {:?}",
        report.fallbacks
    );
    assert_eq!(
        report.fallbacks,
        vec!["jre_guard_finally_copy", "jre_region_uncovered_blocks"],
        "and they are the ones it had before the handoff"
    );
    assert_eq!(
        diagnostic_codes(report),
        vec![
            "jre_guard_finally_copy",
            "jre_region_uncovered_blocks",
            "jre_recovery_produced",
            "jre_declaration",
        ],
        "the missing-class diagnostic left, and the body's own two stayed"
    );
    assert!(
        report.text.contains("// @bytecode 0"),
        "the refused instructions are still quoted: {}",
        report.text
    );
    assert!(
        report.text.contains("// @declaration a static method"),
        "beside the declaration the run could now read: {}",
        report.text
    );
}

// -------------------------------------------------------------------------------------------
// The class flags this file reads
// -------------------------------------------------------------------------------------------

/// `ACC_PUBLIC`, `ACC_STATIC`, `ACC_INTERFACE` and `ACC_ABSTRACT` (JVMS 4.1/4.6).
///
/// Written here rather than imported because these are the class file's own bits, and the layer
/// above states only the three it reads for its own decisions.
const ACC_PUBLIC: u16 = 0x0001;
const ACC_STATIC: u16 = 0x0008;
const ACC_INTERFACE: u16 = 0x0200;
const ACC_ABSTRACT: u16 = 0x0400;
