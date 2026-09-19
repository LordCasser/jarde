//! P4 1.1 acceptance: the release-bound feature registry on the public entry points.
//!
//! The registry is a table keyed by release, and this file pins it from outside the crates that own
//! it: `feature_registry()` answers what a release legally allows, and the header report states the
//! four planes a caller must not confuse — the structure was read (`structural_read`), the dialect
//! band is the record's (`version_dialect_support`), no verifier ran (`verification`), and no output
//! level was evaluated (`output_level`).
//!
//! The version variants come from patching the two version fields of a committed class file
//! (`fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class`), which is the same bytes the
//! reader's own unit tests use: what changes is only the release the artifact claims to be, so a
//! readable structure must survive versions the registry does not validate.
//!
//! The **boundary** this slice draws: the registry states and answers the tag, attribute, flag and
//! opcode rules, and the passes that read those facts off an artifact report them (1.2/1.3). No
//! attribute is read here, and no header diagnostic is an attribute verdict.

use jarde::{
    AttributePlacement, Budget, ClassfileLocation, HeaderStructuralRead, InspectionMode, Limits,
    OutputLevelStatus, PreviewMarker, ReleaseLookup, ReleaseRegistration, VerificationStatus,
    VersionDialectSupport, feature_registry, inspect_header,
};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

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
        nested_depth: u64::MAX,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn with_version(major: u16, minor: u16) -> Vec<u8> {
    let mut bytes = FIXTURE.to_vec();
    bytes[4..6].copy_from_slice(&minor.to_be_bytes());
    bytes[6..8].copy_from_slice(&major.to_be_bytes());
    bytes
}

#[test]
fn preview_and_unregistered_releases_keep_a_readable_structure_and_claim_nothing() {
    for (major, minor) in [(72u16, 0u16), (99, 0), (72, u16::MAX)] {
        let bytes = with_version(major, minor);
        let inspection = inspect_header(
            &bytes,
            &mut Budget::new(unlimited()),
            InspectionMode::Forensic,
        )
        .unwrap();
        // Readable structure, and that is the only plane that succeeded.
        assert_eq!(inspection.structural_read, HeaderStructuralRead::Complete);
        assert_eq!(inspection.verification, VerificationStatus::NotPerformed);
        assert_eq!(inspection.output_level, OutputLevelStatus::NotEvaluated);
        // The dialect plane states non-support explicitly and never a supported band.
        assert_eq!(
            inspection.version_capability.version_dialect_support,
            VersionDialectSupport::FutureRelease,
            "{major}.{minor}"
        );
        assert_ne!(
            inspection.version_capability.version_dialect_support,
            VersionDialectSupport::Supported
        );
        assert_eq!(
            inspection.version_capability.release_registration,
            ReleaseRegistration::UnregisteredFutureRelease,
            "{major}.{minor}"
        );
        // The registry makes no claim about an unregistered release, on any axis.
        let registry = feature_registry();
        assert_eq!(
            registry.release(major),
            ReleaseLookup::UnregisteredFutureRelease
        );
        assert_eq!(registry.release_record(major), None);
        assert_eq!(registry.attribute("Record", major), None);
        assert_eq!(registry.attributes(major).count(), 0);
        assert_eq!(registry.flags(major).count(), 0);
        assert!(registry.unregistered_notes(major).is_empty());
        assert_eq!(
            registry.attribute_diagnostic("Record", ClassfileLocation::ClassFile, major),
            None
        );
        assert_eq!(
            registry.flag_diagnostic("ACC_MODULE", ClassfileLocation::ClassFile, major),
            None
        );
        assert_eq!(registry.opcode_diagnostic(0xba, major), None);
        assert_eq!(registry.opcode_diagnostic(0xca, major), None);
        // Strict mode refuses the same input the forensic read kept readable.
        assert!(
            inspect_header(
                &bytes,
                &mut Budget::new(unlimited()),
                InspectionMode::Strict
            )
            .is_err()
        );
    }
}

#[test]
fn a_preview_marker_is_reported_without_becoming_dialect_support() {
    let bytes = with_version(56, u16::MAX);
    let inspection = inspect_header(
        &bytes,
        &mut Budget::new(unlimited()),
        InspectionMode::Forensic,
    )
    .unwrap();
    assert_eq!(
        inspection.version_capability.preview_marker,
        PreviewMarker::Present
    );
    assert_eq!(
        inspection.version_capability.release_registration,
        ReleaseRegistration::Registered
    );
    assert_eq!(
        inspection.version_capability.version_dialect_support,
        VersionDialectSupport::UnsupportedPreview
    );
    assert_eq!(inspection.structural_read, HeaderStructuralRead::Complete);
    assert_eq!(inspection.verification, VerificationStatus::NotPerformed);
    assert_eq!(inspection.output_level, OutputLevelStatus::NotEvaluated);
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "classfile_preview_unsupported")
    );
}

#[test]
fn attribute_rules_are_bound_to_a_release_and_a_location() {
    let registry = feature_registry();
    let cases = [
        ("Module", 52u16, 53u16),
        ("NestHost", 54, 55),
        ("NestMembers", 54, 55),
        ("Record", 55, 60),
        ("PermittedSubclasses", 60, 61),
    ];
    for (name, before, since) in cases {
        let diagnostic = registry
            .attribute_diagnostic(name, ClassfileLocation::ClassFile, before)
            .expect("a version rule is stated");
        assert_eq!(
            diagnostic.code,
            "classfile_attribute_version_not_applicable"
        );
        assert!(
            diagnostic.message.starts_with(&format!(
                "attribute \"{name}\" is registered from major {since} (JVMS 4.7."
            )),
            "{}",
            diagnostic.message
        );
        // The diagnostic is a release rule, not evidence taken from an artifact.
        assert!(diagnostic.provenance.is_none());
        // The same name is legal from its release on, and at the registry ceiling.
        assert_eq!(
            registry.attribute_diagnostic(name, ClassfileLocation::ClassFile, since),
            None
        );
        assert_eq!(
            registry.attribute_placement(name, ClassfileLocation::ClassFile, 71),
            AttributePlacement::Legal {
                rule: registry.attribute(name, 71).unwrap()
            }
        );
    }

    let location = registry
        .attribute_diagnostic("NestMembers", ClassfileLocation::MethodInfo, 55)
        .expect("a location rule is stated");
    assert_eq!(location.code, "classfile_attribute_location_not_applicable");
    assert_eq!(
        location.message,
        "attribute \"NestMembers\" is registered only for ClassFile (JVMS 4.7.29); it is not valid in a method_info structure"
    );

    let flag = registry
        .flag_diagnostic("ACC_MODULE", ClassfileLocation::ClassFile, 52)
        .expect("a flag version rule is stated");
    assert_eq!(flag.code, "classfile_flag_version_not_applicable");
    assert_eq!(
        registry
            .flag_diagnostic("ACC_MODULE", ClassfileLocation::MethodInfo, 53)
            .expect("a flag location rule is stated")
            .code,
        "classfile_flag_location_not_applicable"
    );

    // A name the registry does not hold is no claim: neither legal nor a violation.
    assert_eq!(
        registry.attribute_diagnostic("MyProjectAttribute", ClassfileLocation::ClassFile, 55),
        None
    );
    assert_eq!(
        registry.attribute_placement("MyProjectAttribute", ClassfileLocation::ClassFile, 55),
        AttributePlacement::NotRegistered
    );
    assert_eq!(
        registry.flag_placement("ACC_STRICT", ClassfileLocation::MethodInfo, 71),
        jarde::FlagPlacement::NotRegistered
    );
}

#[test]
fn every_registered_release_is_readable_and_the_ceiling_is_where_it_claims_to_be() {
    let registry = feature_registry();
    let majors: Vec<u16> = registry.releases().map(|record| record.major).collect();
    assert_eq!(majors, (45..=71).collect::<Vec<u16>>());
    assert_eq!(registry.dialect_validated_ceiling(), 52);
    assert_eq!(
        registry.release(71).registration(),
        ReleaseRegistration::Registered
    );
    assert_eq!(
        registry.release(72),
        ReleaseLookup::UnregisteredFutureRelease
    );
    assert_eq!(registry.attribute("Record", 59), None);
    assert!(registry.attribute("Record", 60).is_some());
    for record in registry.releases() {
        assert!(
            !record.unregistered.is_empty(),
            "major {} states what it does not register",
            record.major
        );
    }
}
