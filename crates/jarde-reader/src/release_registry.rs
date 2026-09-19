//! The release-bound feature registry: what each class-file release legally allows.
//!
//! P0 classified a version by comparing its `major_version` against literal ranges. A band answers
//! "how far does this build go", which is a different statement from "what does this release
//! define". The second statement is a table keyed by *release* — the constant-pool tags, attribute
//! locations and minimum versions, flags and opcode rules that release legally allows — and that
//! table is what the `Version and feature registry` requirement asks this crate to hold.
//!
//! The registry is that table and nothing more:
//!
//! - One [`ReleaseRecord`] per registered major, from [`MINIMUM_MAJOR`] to
//!   [`HIGHEST_REGISTERED_MAJOR`]. A record lists the constraints its release **introduces**; a
//!   query for a release applies every record at or below it, so `Record` is legal at major 60 and
//!   at every later release without being restated per release, and the release that established a
//!   rule stays readable in the rule itself.
//! - Every registered entry cites the JVMS section it was read from. An entry this registry has
//!   **not** established is not invented: it is listed in the record's `unregistered` notes, and a
//!   lookup for it answers "not registered" ([`AttributePlacement`], [`FlagPlacement`],
//!   [`OpcodeStatus`]) instead of claiming the constraint is legal or that it is illegal.
//! - A release the registry does not hold — above [`HIGHEST_REGISTERED_MAJOR`], or below
//!   [`MINIMUM_MAJOR`] — makes **no** claim at all. [`FeatureRegistry::release`] reports it as
//!   unregistered, every lookup answers [`Placement::UnregisteredRelease`] (or the equivalent
//!   status), and no answer borrows a neighbouring release's capabilities. The conservative path
//!   is a state of its own, so a consumer cannot read a future release as validated.
//!
//! The registry states legality; it does not read an artifact. The placement diagnostics name the
//! attribute, flag or opcode and the release rule that decides them, never a class name, method
//! name or path taken from the input, and they carry no provenance because no input byte backs
//! them: an artifact that happens to contain a misplaced `Record` attribute is diagnosed from the
//! release rule, not from the bytes.
//!
//! The seven codes those diagnostics use, for the passes that read the facts and report them:
//!
//! | code | decided by |
//! | --- | --- |
//! | `classfile_attribute_version_not_applicable` | the release registered the attribute later |
//! | `classfile_attribute_location_not_applicable` | the release registers the attribute elsewhere |
//! | `classfile_flag_version_not_applicable` | the release registered the flag later |
//! | `classfile_flag_location_not_applicable` | the release registers the flag elsewhere |
//! | `classfile_opcode_version_not_applicable` | the opcode exists only from a later release on |
//! | `classfile_opcode_forbidden` | the release forbids an opcode earlier ones allowed |
//! | `classfile_opcode_reserved` | the JVMS reserves the opcode; it is never valid |

use crate::classfile::{Java8RuntimeCompatibility, VersionDialectSupport};
use crate::model::{Diagnostic, DiagnosticSeverity};
use serde::{Deserialize, Serialize};

/// The `minor_version` a preview class file carries (JVMS 4.1).
pub const PREVIEW_MARKER: u16 = u16::MAX;

/// The lowest `major_version` the class-file format defines (JVMS 4.1).
pub const MINIMUM_MAJOR: u16 = 45;

/// The highest release this registry holds: major 71, the newest release P0 read structurally.
///
/// The number is this build's declaration of how far its table goes, not a JVMS fact. A version
/// above it is an *unregistered release*: the registry holds no record for it, and every lookup for
/// one answers "no claim" rather than the ceiling's capabilities.
pub const HIGHEST_REGISTERED_MAJOR: u16 = 71;

/// Where a constraint applies: a structure an attribute or an access flag can appear in.
///
/// The vocabulary is JVMS's own structure names (4.1, 4.5, 4.6, 4.7), shared by attributes and
/// flags because the question "is this legal *here*" has the same answer space for both.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ClassfileLocation {
    /// The `attributes` table of the `ClassFile` structure.
    ClassFile,
    /// The `attributes` table of a `field_info` structure.
    FieldInfo,
    /// The `attributes` table of a `method_info` structure.
    MethodInfo,
    /// The `attributes` table of a `Code` attribute (JVMS 4.7.3).
    Code,
    /// One `parameter` of a `MethodParameters` attribute (JVMS 4.7.24).
    MethodParameter,
}

impl ClassfileLocation {
    /// The name JVMS uses for this structure, for diagnostics and notes.
    pub const fn jvms_name(self) -> &'static str {
        match self {
            Self::ClassFile => "ClassFile",
            Self::FieldInfo => "field_info",
            Self::MethodInfo => "method_info",
            Self::Code => "Code",
            Self::MethodParameter => "MethodParameters",
        }
    }
}

/// One attribute's legality, as the release that introduces it states it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttributeRule {
    /// The attribute name a `CONSTANT_Utf8` in `attribute_name_index` spells.
    pub name: &'static str,
    /// Every structure JVMS places this attribute in.
    pub locations: &'static [ClassfileLocation],
    /// The lowest `major_version` from which JVMS places the attribute in those locations.
    pub since: u16,
    /// The JVMS section this entry was read from.
    pub source: &'static str,
}

/// One constant-pool tag's availability, as the release that introduces it states it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConstantPoolTagRule {
    pub tag: u8,
    pub name: &'static str,
    /// The lowest `major_version` whose constant pool may hold this tag.
    pub since: u16,
    pub source: &'static str,
}

/// One access flag's legality at one or more structures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlagRule {
    /// The JVMS name of the flag, e.g. `ACC_MODULE`.
    pub name: &'static str,
    /// The bit the flag occupies in the structure's `access_flags`.
    pub bits: u16,
    /// Every structure this flag is declared for.
    pub locations: &'static [ClassfileLocation],
    /// The lowest `major_version` that declares this flag for those locations.
    pub since: u16,
    pub source: &'static str,
}

/// What a release says about one opcode, when it says anything.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpcodeConstraint {
    /// The opcode exists from this `major_version` on.
    RegisteredFrom(u16),
    /// Releases from this `major_version` on must not contain the opcode, though earlier ones could.
    ForbiddenFrom(u16),
    /// Reserved by the JVMS and never valid in a class file.
    Reserved,
}

/// One opcode's rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpcodeRule {
    pub opcode: u8,
    pub name: &'static str,
    pub constraint: OpcodeConstraint,
    pub source: &'static str,
}

/// The answer to "is this attribute or flag legal here, at this release?".
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Placement<Rule> {
    /// Registered for this release and legal at the queried location.
    Legal { rule: Rule },
    /// Registered, but introduced by a later release than the queried one.
    VersionNotApplicable { rule: Rule },
    /// Registered, but not for the queried location.
    LocationNotApplicable {
        rule: Rule,
        /// Every location the registry declares for this name, in registry order.
        allowed: Vec<ClassfileLocation>,
    },
    /// The registry holds no rule for this name at any release it covers. The answer is "no rule",
    /// not "illegal": a name this registry does not hold is a name it makes no claim about.
    NotRegistered,
    /// The queried release itself is not registered ([`ReleaseLookup::UnregisteredFutureRelease`]
    /// or [`ReleaseLookup::UnregisteredBelowMinimum`]), so no name is reported legal for it.
    UnregisteredRelease,
}

/// The placement answer for an attribute name.
pub type AttributePlacement = Placement<&'static AttributeRule>;

/// The placement answer for an access flag name.
pub type FlagPlacement = Placement<&'static FlagRule>;

/// The answer to "does this release's constant pool hold this tag?".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstantPoolTagStatus {
    /// Registered for this release.
    Registered { rule: &'static ConstantPoolTagRule },
    /// Registered, but introduced by a later release than the queried one.
    VersionNotApplicable { rule: &'static ConstantPoolTagRule },
    /// No rule for this tag at any release the registry covers.
    Unregistered,
    /// The queried release is not registered, so no tag is reported legal for it.
    UnregisteredRelease,
}

/// The answer to "is this opcode legal at this release?".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpcodeStatus {
    /// Registered and legal for this release.
    Legal { rule: &'static OpcodeRule },
    /// Registered, but the opcode only exists from a later release on.
    VersionNotApplicable { rule: &'static OpcodeRule },
    /// The release forbids an opcode that earlier releases allowed.
    Forbidden { rule: &'static OpcodeRule },
    /// Reserved by the JVMS and never valid in a class file.
    Reserved { rule: &'static OpcodeRule },
    /// No rule for this opcode at any release the registry covers.
    Unregistered,
    /// The queried release is not registered, so no opcode is reported legal for it.
    UnregisteredRelease,
}

/// How the `minor_version` of one release must look (JVMS 4.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinorForm {
    /// Releases below 56: the format places no constraint on `minor_version`.
    Unconstrained,
    /// Releases 56 and above: `minor_version` is `0` or the preview marker.
    ZeroOrPreviewMarker,
}

/// What the preview marker means in one release (JVMS 4.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewRule {
    /// The `minor_version` that marks a preview class file of this release.
    pub marker: u16,
    /// The dialect support this build declares for a preview class file of this release.
    pub dialect_support: VersionDialectSupport,
    pub source: &'static str,
}

/// The Java 8 runtime profile rule of one release.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Java8RuntimeRule {
    /// Every minor of this release is the release the profile was declared for.
    Accepted,
    /// Only minor 0 is Java SE 8; a later minor of the same major is not that release.
    AcceptedAtMinorZero,
    /// The release is newer than the profile.
    Rejected,
}

/// The bands the registered releases fall into, as this build declares them.
///
/// The bands carry the support decisions — how far the dialect is validated, whether the preview
/// marker is registered, what the Java 8 profile says — while the constraint tables above carry the
/// release-bound legality. The three bands are the ranges P0 hard-coded, now read from records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseBand {
    /// Releases 45–52: the dialect is validated, and the Java 8 runtime profile is the rule.
    DialectValidated { java8_runtime: Java8RuntimeRule },
    /// Releases 53–55: structural probe only, and no preview marker.
    StructuralProbe,
    /// Releases 56–71: structural probe only, with the preview marker registered as unsupported.
    StructuralProbeWithPreview { preview: PreviewRule },
}

impl ReleaseBand {
    const fn preview(self) -> Option<PreviewRule> {
        match self {
            Self::StructuralProbeWithPreview { preview } => Some(preview),
            Self::DialectValidated { .. } | Self::StructuralProbe => None,
        }
    }

    const fn minor_form(self) -> MinorForm {
        match self {
            Self::StructuralProbeWithPreview { .. } => MinorForm::ZeroOrPreviewMarker,
            Self::DialectValidated { .. } | Self::StructuralProbe => MinorForm::Unconstrained,
        }
    }

    /// The dialect support this build declares for a *non-preview* class file of the release.
    const fn dialect_support(self) -> VersionDialectSupport {
        match self {
            Self::DialectValidated { .. } => VersionDialectSupport::Supported,
            Self::StructuralProbe | Self::StructuralProbeWithPreview { .. } => {
                VersionDialectSupport::StructuralProbeOnly
            }
        }
    }
}

/// The constraints one release introduces, each citing its JVMS source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntroducedConstraints {
    pub constant_pool_tags: &'static [ConstantPoolTagRule],
    pub attributes: &'static [AttributeRule],
    pub flags: &'static [FlagRule],
    pub opcodes: &'static [OpcodeRule],
}

impl IntroducedConstraints {
    /// A release that introduces no constraint of its own; the earlier records still apply.
    pub const NONE: Self = Self {
        constant_pool_tags: &[],
        attributes: &[],
        flags: &[],
        opcodes: &[],
    };
}

/// One registered release: its minor/preview shape, its band, what it introduces, and what this
/// registry deliberately does not record for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseRecord {
    pub major: u16,
    pub band: ReleaseBand,
    pub introduced: IntroducedConstraints,
    /// The constraint classes this registry does **not** record for this release, so a consumer
    /// cannot read the registered entries as a complete table. Never empty: every release states
    /// at least one unregistered class.
    pub unregistered: &'static [&'static str],
}

impl ReleaseRecord {
    /// The preview rule of this release, if it registers one.
    pub const fn preview(&self) -> Option<PreviewRule> {
        self.band.preview()
    }

    pub const fn minor_form(&self) -> MinorForm {
        self.band.minor_form()
    }

    /// The dialect support for a non-preview class file of this release.
    pub const fn dialect_support(&self) -> VersionDialectSupport {
        self.band.dialect_support()
    }

    /// The Java 8 runtime profile verdict for one version of this release.
    pub const fn java8_runtime(&self, minor: u16) -> Java8RuntimeCompatibility {
        let accepted = match self.band {
            ReleaseBand::DialectValidated { java8_runtime } => match java8_runtime {
                Java8RuntimeRule::Accepted => true,
                Java8RuntimeRule::AcceptedAtMinorZero => minor == 0,
                Java8RuntimeRule::Rejected => false,
            },
            ReleaseBand::StructuralProbe | ReleaseBand::StructuralProbeWithPreview { .. } => false,
        };
        if accepted {
            Java8RuntimeCompatibility::Accepted
        } else {
            Java8RuntimeCompatibility::Rejected
        }
    }
}

/// What the registry holds for the major being classified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseLookup {
    /// The registry holds a record for this major.
    Registered(&'static ReleaseRecord),
    /// The major is above [`HIGHEST_REGISTERED_MAJOR`]: an unregistered release, about which the
    /// registry makes no claim.
    UnregisteredFutureRelease,
    /// The major is below [`MINIMUM_MAJOR`], which the format itself does not define.
    UnregisteredBelowMinimum,
}

impl ReleaseLookup {
    /// The lookup in the report's own vocabulary, for callers that publish a status.
    pub const fn registration(self) -> ReleaseRegistration {
        match self {
            Self::Registered(_) => ReleaseRegistration::Registered,
            Self::UnregisteredFutureRelease => ReleaseRegistration::UnregisteredFutureRelease,
            Self::UnregisteredBelowMinimum => ReleaseRegistration::UnregisteredBelowMinimum,
        }
    }
}

/// Whether the version being classified names a release this registry holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseRegistration {
    Registered,
    UnregisteredFutureRelease,
    UnregisteredBelowMinimum,
}

/// The registry itself: one table of [`ReleaseRecord`]s, queried per release.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FeatureRegistry;

/// The one registry this build ships.
///
/// A free function, like `classfile::version_capability`, so a consumer reads the same table the
/// reader classified a version with instead of restating any rule.
pub const fn feature_registry() -> FeatureRegistry {
    FeatureRegistry
}

impl FeatureRegistry {
    /// What the registry holds for `major`.
    pub fn release(&self, major: u16) -> ReleaseLookup {
        if major < MINIMUM_MAJOR {
            return ReleaseLookup::UnregisteredBelowMinimum;
        }
        if major > HIGHEST_REGISTERED_MAJOR {
            return ReleaseLookup::UnregisteredFutureRelease;
        }
        match RELEASES.iter().find(|record| record.major == major) {
            Some(record) => ReleaseLookup::Registered(record),
            // Unreachable while every major from MINIMUM_MAJOR to HIGHEST_REGISTERED_MAJOR has a
            // record (the table's own test asserts that); answering "unregistered" keeps the
            // conservative path if the table ever gains a gap.
            None => ReleaseLookup::UnregisteredFutureRelease,
        }
    }

    /// The record of `major`, if the registry holds one.
    pub fn release_record(&self, major: u16) -> Option<&'static ReleaseRecord> {
        match self.release(major) {
            ReleaseLookup::Registered(record) => Some(record),
            ReleaseLookup::UnregisteredFutureRelease | ReleaseLookup::UnregisteredBelowMinimum => {
                None
            }
        }
    }

    /// Every registered release, in ascending major order.
    pub fn releases(&self) -> impl Iterator<Item = &'static ReleaseRecord> {
        RELEASES.iter()
    }

    /// The lowest registered release whose `minor_version` the format constrains (JVMS 4.1).
    ///
    /// The rule applies to every release at or above it, including releases this registry does not
    /// hold, because the constraint the *format* places on `minor_version` does not depend on which
    /// releases a build happens to record.
    pub fn modern_minor_since(&self) -> u16 {
        RELEASES
            .iter()
            .find(|record| record.minor_form() == MinorForm::ZeroOrPreviewMarker)
            .map_or(HIGHEST_REGISTERED_MAJOR, |record| record.major)
    }

    /// The highest registered release whose dialect this build validates.
    ///
    /// The number is read from the records rather than restated, so the diagnostics that name the
    /// end of dialect validation cannot drift from the band that decides it.
    pub fn dialect_validated_ceiling(&self) -> u16 {
        RELEASES
            .iter()
            .filter(|record| record.dialect_support() == VersionDialectSupport::Supported)
            .map(|record| record.major)
            .max()
            .unwrap_or(MINIMUM_MAJOR)
    }

    /// The constraint classes this registry does not record for `major`; empty for an unregistered
    /// release, whose whole record is absent rather than partially filled.
    pub fn unregistered_notes(&self, major: u16) -> &'static [&'static str] {
        match self.release(major) {
            ReleaseLookup::Registered(record) => record.unregistered,
            ReleaseLookup::UnregisteredFutureRelease | ReleaseLookup::UnregisteredBelowMinimum => {
                &[]
            }
        }
    }

    /// Every constant-pool tag rule applicable at `major`: the records at or below it.
    pub fn constant_pool_tags(
        &self,
        major: u16,
    ) -> impl Iterator<Item = &'static ConstantPoolTagRule> {
        self.applicable_records(major)
            .flat_map(|record| record.introduced.constant_pool_tags.iter())
    }

    /// Whether the constant pool of `major` may hold `tag`.
    pub fn constant_pool_tag(&self, tag: u8, major: u16) -> ConstantPoolTagStatus {
        if !matches!(self.release(major), ReleaseLookup::Registered(_)) {
            return ConstantPoolTagStatus::UnregisteredRelease;
        }
        if let Some(rule) = self.constant_pool_tags(major).find(|rule| rule.tag == tag) {
            return ConstantPoolTagStatus::Registered { rule };
        }
        if let Some(rule) = self
            .later_records(major)
            .flat_map(|later| later.introduced.constant_pool_tags.iter())
            .find(|rule| rule.tag == tag)
        {
            return ConstantPoolTagStatus::VersionNotApplicable { rule };
        }
        ConstantPoolTagStatus::Unregistered
    }

    /// Every attribute rule applicable at `major`: the records at or below it.
    pub fn attributes(&self, major: u16) -> impl Iterator<Item = &'static AttributeRule> {
        self.applicable_records(major)
            .flat_map(|record| record.introduced.attributes.iter())
    }

    /// The rule the registry holds for `name` at `major`, if any: the newest applicable entry.
    pub fn attribute(&self, name: &str, major: u16) -> Option<&'static AttributeRule> {
        self.attributes(major)
            .filter(|rule| rule.name == name)
            .last()
    }

    /// Whether `name` is a legal attribute of the `location` structure at `major`.
    pub fn attribute_placement(
        &self,
        name: &str,
        location: ClassfileLocation,
        major: u16,
    ) -> AttributePlacement {
        self.placement(
            name,
            location,
            major,
            self.attributes(major),
            self.later_attributes(major),
        )
    }

    /// The version or location diagnostic for a misplaced attribute name, if the registry holds a
    /// rule that makes the placement illegal.
    ///
    /// `None` for a legal placement, for a name the registry does not hold, and for an unregistered
    /// release: a name this registry makes no claim about is not reported as a violation.
    pub fn attribute_diagnostic(
        &self,
        name: &str,
        location: ClassfileLocation,
        major: u16,
    ) -> Option<Diagnostic> {
        match self.attribute_placement(name, location, major) {
            AttributePlacement::VersionNotApplicable { rule } => Some(registry_diagnostic(
                "classfile_attribute_version_not_applicable",
                format!(
                    "attribute \"{}\" is registered from major {} ({}); it is not valid at major {}",
                    rule.name, rule.since, rule.source, major
                ),
            )),
            AttributePlacement::LocationNotApplicable { rule, allowed } => {
                Some(registry_diagnostic(
                    "classfile_attribute_location_not_applicable",
                    format!(
                        "attribute \"{}\" is registered only for {} ({}); it is not valid in a {} structure",
                        rule.name,
                        location_list(&allowed),
                        rule.source,
                        location.jvms_name()
                    ),
                ))
            }
            AttributePlacement::Legal { .. }
            | AttributePlacement::NotRegistered
            | AttributePlacement::UnregisteredRelease => None,
        }
    }

    /// Every access flag rule applicable at `major`: the records at or below it.
    pub fn flags(&self, major: u16) -> impl Iterator<Item = &'static FlagRule> {
        self.applicable_records(major)
            .flat_map(|record| record.introduced.flags.iter())
    }

    /// Whether `name` is a legal access flag of the `location` structure at `major`.
    pub fn flag_placement(
        &self,
        name: &str,
        location: ClassfileLocation,
        major: u16,
    ) -> FlagPlacement {
        self.placement(
            name,
            location,
            major,
            self.flags(major),
            self.later_flags(major),
        )
    }

    /// The version or location diagnostic for a misplaced access flag, if the registry holds a rule
    /// that makes the placement illegal.
    pub fn flag_diagnostic(
        &self,
        name: &str,
        location: ClassfileLocation,
        major: u16,
    ) -> Option<Diagnostic> {
        match self.flag_placement(name, location, major) {
            FlagPlacement::VersionNotApplicable { rule } => Some(registry_diagnostic(
                "classfile_flag_version_not_applicable",
                format!(
                    "access flag {} (0x{:04x}) is registered from major {} ({}); it is not valid at major {}",
                    rule.name, rule.bits, rule.since, rule.source, major
                ),
            )),
            FlagPlacement::LocationNotApplicable { rule, allowed } => Some(registry_diagnostic(
                "classfile_flag_location_not_applicable",
                format!(
                    "access flag {} (0x{:04x}) is registered only for {} ({}); it is not valid in a {} structure",
                    rule.name,
                    rule.bits,
                    location_list(&allowed),
                    rule.source,
                    location.jvms_name()
                ),
            )),
            FlagPlacement::Legal { .. }
            | FlagPlacement::NotRegistered
            | FlagPlacement::UnregisteredRelease => None,
        }
    }

    /// The opcode rules registered at `major`: the records at or below it.
    ///
    /// This is the registration view. [`FeatureRegistry::opcode_status`] is the legality view: an
    /// opcode rule states the *range* of releases it gates, so a release below the record that
    /// introduced the rule is still described by it.
    pub fn opcodes(&self, major: u16) -> impl Iterator<Item = &'static OpcodeRule> {
        self.applicable_records(major)
            .flat_map(|record| record.introduced.opcodes.iter())
    }

    /// Every opcode rule the registry holds, from every record, in ascending release order.
    ///
    /// No opcode is registered twice: the table's own test asserts that, which is what makes the
    /// first matching entry the rule for an opcode at every release.
    pub fn opcode_rules(&self) -> impl Iterator<Item = &'static OpcodeRule> {
        RELEASES
            .iter()
            .flat_map(|record| record.introduced.opcodes.iter())
    }

    /// Whether `opcode` is legal at `major`.
    ///
    /// The rule's own gate decides, not the release that introduced it: `invokedynamic` is
    /// `RegisteredFrom(51)`, so it is a version violation at 50 even though the rule is registered
    /// by the record of 51, and `jsr` is `ForbiddenFrom(51)`, so it is legal below 51 and forbidden
    /// from 51 on.
    pub fn opcode_status(&self, opcode: u8, major: u16) -> OpcodeStatus {
        if !matches!(self.release(major), ReleaseLookup::Registered(_)) {
            return OpcodeStatus::UnregisteredRelease;
        }
        let Some(rule) = self.opcode_rules().find(|rule| rule.opcode == opcode) else {
            return OpcodeStatus::Unregistered;
        };
        match rule.constraint {
            OpcodeConstraint::RegisteredFrom(since) if major < since => {
                OpcodeStatus::VersionNotApplicable { rule }
            }
            OpcodeConstraint::ForbiddenFrom(since) if major >= since => {
                OpcodeStatus::Forbidden { rule }
            }
            OpcodeConstraint::Reserved => OpcodeStatus::Reserved { rule },
            OpcodeConstraint::RegisteredFrom(_) | OpcodeConstraint::ForbiddenFrom(_) => {
                OpcodeStatus::Legal { rule }
            }
        }
    }

    /// The version diagnostic for an opcode the release does not allow, if the registry holds a
    /// rule that forbids it.
    pub fn opcode_diagnostic(&self, opcode: u8, major: u16) -> Option<Diagnostic> {
        match self.opcode_status(opcode, major) {
            OpcodeStatus::VersionNotApplicable { rule } => {
                let OpcodeConstraint::RegisteredFrom(since) = rule.constraint else {
                    return None;
                };
                Some(registry_diagnostic(
                    "classfile_opcode_version_not_applicable",
                    format!(
                        "opcode 0x{:02x} ({}) is registered from major {} ({}); it is not valid at major {}",
                        rule.opcode, rule.name, since, rule.source, major
                    ),
                ))
            }
            OpcodeStatus::Forbidden { rule } => {
                let OpcodeConstraint::ForbiddenFrom(since) = rule.constraint else {
                    return None;
                };
                Some(registry_diagnostic(
                    "classfile_opcode_forbidden",
                    format!(
                        "opcode 0x{:02x} ({}) is forbidden from major {} ({}); it must not appear at major {}",
                        rule.opcode, rule.name, since, rule.source, major
                    ),
                ))
            }
            OpcodeStatus::Reserved { rule } => Some(registry_diagnostic(
                "classfile_opcode_reserved",
                format!(
                    "opcode 0x{:02x} ({}) is reserved by the JVMS ({}); it must not appear in a class file",
                    rule.opcode, rule.name, rule.source
                ),
            )),
            OpcodeStatus::Legal { .. }
            | OpcodeStatus::Unregistered
            | OpcodeStatus::UnregisteredRelease => None,
        }
    }

    /// The records whose constraints apply at `major`, which is none of them for a release the
    /// registry does not hold: that absence is what keeps every lookup from making a claim.
    fn applicable_records(&self, major: u16) -> impl Iterator<Item = &'static ReleaseRecord> {
        let registered = matches!(self.release(major), ReleaseLookup::Registered(_));
        RELEASES
            .iter()
            .filter(move |record| registered && record.major <= major)
    }

    /// The attribute rules registered *after* `major`; empty for an unregistered release.
    fn later_attributes(&self, major: u16) -> impl Iterator<Item = &'static AttributeRule> {
        self.later_records(major)
            .flat_map(|record| record.introduced.attributes.iter())
    }

    /// The flag rules registered *after* `major`; empty for an unregistered release.
    fn later_flags(&self, major: u16) -> impl Iterator<Item = &'static FlagRule> {
        self.later_records(major)
            .flat_map(|record| record.introduced.flags.iter())
    }

    fn later_records(&self, major: u16) -> impl Iterator<Item = &'static ReleaseRecord> {
        let registered = matches!(self.release(major), ReleaseLookup::Registered(_));
        RELEASES
            .iter()
            .filter(move |record| registered && record.major > major)
    }

    /// The shared placement rule for names that carry a location list (attributes and flags).
    ///
    /// The four answers are decided in this order: an entry applicable at `major` that declares the
    /// location is legal; an entry registered later that declares it is a version violation; a name
    /// the registry holds only for other locations is a location violation; a name it does not hold
    /// at all is no claim. Every location the registry declares for the name is reported with the
    /// rule, so the diagnostic states the whole registered answer.
    fn placement<Rule: ConstraintRule>(
        &self,
        name: &str,
        location: ClassfileLocation,
        major: u16,
        applicable: impl Iterator<Item = &'static Rule>,
        later: impl Iterator<Item = &'static Rule>,
    ) -> Placement<&'static Rule> {
        if !matches!(self.release(major), ReleaseLookup::Registered(_)) {
            return Placement::UnregisteredRelease;
        }
        let mut legal: Option<&'static Rule> = None;
        let mut named: Option<&'static Rule> = None;
        let mut later_match: Option<&'static Rule> = None;
        let mut allowed: Vec<ClassfileLocation> = Vec::new();
        for rule in applicable {
            if rule.name() != name {
                continue;
            }
            named.get_or_insert(rule);
            for candidate in rule.locations() {
                if !allowed.contains(candidate) {
                    allowed.push(*candidate);
                }
            }
            if rule.locations().contains(&location) {
                legal = Some(rule);
            }
        }
        for rule in later {
            if rule.name() != name {
                continue;
            }
            named.get_or_insert(rule);
            for candidate in rule.locations() {
                if !allowed.contains(candidate) {
                    allowed.push(*candidate);
                }
            }
            if later_match.is_none() && rule.locations().contains(&location) {
                later_match = Some(rule);
            }
        }
        match (legal, later_match, named) {
            (Some(rule), _, _) => Placement::Legal { rule },
            (None, Some(rule), _) => Placement::VersionNotApplicable { rule },
            (None, None, Some(rule)) => Placement::LocationNotApplicable { rule, allowed },
            (None, None, None) => Placement::NotRegistered,
        }
    }
}

/// The name and location list of a constraint entry, for the shared placement rule.
trait ConstraintRule {
    fn name(&self) -> &'static str;
    fn locations(&self) -> &'static [ClassfileLocation];
}

impl ConstraintRule for AttributeRule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn locations(&self) -> &'static [ClassfileLocation] {
        self.locations
    }
}

impl ConstraintRule for FlagRule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn locations(&self) -> &'static [ClassfileLocation] {
        self.locations
    }
}

/// A diagnostic stated by a release rule rather than by an artifact byte.
///
/// `provenance` stays `None`: the reader's own version diagnostics do the same, and an answer read
/// from the registry must not look like evidence taken from the input.
fn registry_diagnostic(code: &str, message: String) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity: DiagnosticSeverity::Error,
        message,
        provenance: None,
    }
}

fn location_list(locations: &[ClassfileLocation]) -> String {
    locations
        .iter()
        .map(|location| location.jvms_name())
        .collect::<Vec<_>>()
        .join(", ")
}

const CLASS_FILE: &[ClassfileLocation] = &[ClassfileLocation::ClassFile];
const FIELD_INFO: &[ClassfileLocation] = &[ClassfileLocation::FieldInfo];
const METHOD_INFO: &[ClassfileLocation] = &[ClassfileLocation::MethodInfo];
const CODE: &[ClassfileLocation] = &[ClassfileLocation::Code];
const CLASS_FIELD: &[ClassfileLocation] =
    &[ClassfileLocation::ClassFile, ClassfileLocation::FieldInfo];
const CLASS_FIELD_METHOD: &[ClassfileLocation] = &[
    ClassfileLocation::ClassFile,
    ClassfileLocation::FieldInfo,
    ClassfileLocation::MethodInfo,
];
const TYPE_ANNOTATION_TARGETS: &[ClassfileLocation] = &[
    ClassfileLocation::ClassFile,
    ClassfileLocation::FieldInfo,
    ClassfileLocation::MethodInfo,
    ClassfileLocation::Code,
];
const METHOD_PARAMETER: &[ClassfileLocation] = &[ClassfileLocation::MethodParameter];

/// The attributes the format defines at its minimum major, with no lower gate to record.
const ATTRIBUTES_BASELINE: &[AttributeRule] = &[
    AttributeRule {
        name: "Code",
        locations: METHOD_INFO,
        since: 45,
        source: "JVMS 4.7.3",
    },
    AttributeRule {
        name: "ConstantValue",
        locations: FIELD_INFO,
        since: 45,
        source: "JVMS 4.7.2",
    },
    AttributeRule {
        name: "Deprecated",
        locations: CLASS_FIELD_METHOD,
        since: 45,
        source: "JVMS 4.7.15",
    },
    AttributeRule {
        name: "Exceptions",
        locations: METHOD_INFO,
        since: 45,
        source: "JVMS 4.7.5",
    },
    AttributeRule {
        name: "InnerClasses",
        locations: CLASS_FILE,
        since: 45,
        source: "JVMS 4.7.6",
    },
    AttributeRule {
        name: "LineNumberTable",
        locations: CODE,
        since: 45,
        source: "JVMS 4.7.12",
    },
    AttributeRule {
        name: "LocalVariableTable",
        locations: CODE,
        since: 45,
        source: "JVMS 4.7.13",
    },
    AttributeRule {
        name: "SourceFile",
        locations: CLASS_FILE,
        since: 45,
        source: "JVMS 4.7.10",
    },
];

/// The attributes Java SE 5 (major 49) introduced.
const ATTRIBUTES_JAVA5: &[AttributeRule] = &[
    AttributeRule {
        name: "AnnotationDefault",
        locations: METHOD_INFO,
        since: 49,
        source: "JVMS 4.7.22",
    },
    AttributeRule {
        name: "EnclosingMethod",
        locations: CLASS_FILE,
        since: 49,
        source: "JVMS 4.7.7",
    },
    AttributeRule {
        name: "LocalVariableTypeTable",
        locations: CODE,
        since: 49,
        source: "JVMS 4.7.14",
    },
    AttributeRule {
        name: "RuntimeInvisibleAnnotations",
        locations: CLASS_FIELD_METHOD,
        since: 49,
        source: "JVMS 4.7.17",
    },
    AttributeRule {
        name: "RuntimeInvisibleParameterAnnotations",
        locations: METHOD_INFO,
        since: 49,
        source: "JVMS 4.7.19",
    },
    AttributeRule {
        name: "RuntimeVisibleAnnotations",
        locations: CLASS_FIELD_METHOD,
        since: 49,
        source: "JVMS 4.7.16",
    },
    AttributeRule {
        name: "RuntimeVisibleParameterAnnotations",
        locations: METHOD_INFO,
        since: 49,
        source: "JVMS 4.7.18",
    },
    AttributeRule {
        name: "Signature",
        locations: CLASS_FIELD_METHOD,
        since: 49,
        source: "JVMS 4.7.9",
    },
    AttributeRule {
        name: "SourceDebugExtension",
        locations: CLASS_FILE,
        since: 49,
        source: "JVMS 4.7.11",
    },
];

/// The attributes Java SE 6 (major 50) introduced.
const ATTRIBUTES_JAVA6: &[AttributeRule] = &[AttributeRule {
    name: "StackMapTable",
    locations: CODE,
    since: 50,
    source: "JVMS 4.7.4",
}];

/// The attributes Java SE 7 (major 51) introduced.
const ATTRIBUTES_JAVA7: &[AttributeRule] = &[AttributeRule {
    name: "BootstrapMethods",
    locations: CLASS_FILE,
    since: 51,
    source: "JVMS 4.7.23",
}];

/// The attributes Java SE 8 (major 52) introduced.
const ATTRIBUTES_JAVA8: &[AttributeRule] = &[
    AttributeRule {
        name: "MethodParameters",
        locations: METHOD_INFO,
        since: 52,
        source: "JVMS 4.7.24",
    },
    AttributeRule {
        name: "RuntimeInvisibleTypeAnnotations",
        locations: TYPE_ANNOTATION_TARGETS,
        since: 52,
        source: "JVMS 4.7.21",
    },
    AttributeRule {
        name: "RuntimeVisibleTypeAnnotations",
        locations: TYPE_ANNOTATION_TARGETS,
        since: 52,
        source: "JVMS 4.7.20",
    },
];

/// The attributes Java SE 9 (major 53) introduced.
const ATTRIBUTES_JAVA9: &[AttributeRule] = &[
    AttributeRule {
        name: "Module",
        locations: CLASS_FILE,
        since: 53,
        source: "JVMS 4.7.25",
    },
    AttributeRule {
        name: "ModuleMainClass",
        locations: CLASS_FILE,
        since: 53,
        source: "JVMS 4.7.27",
    },
    AttributeRule {
        name: "ModulePackages",
        locations: CLASS_FILE,
        since: 53,
        source: "JVMS 4.7.26",
    },
];

/// The attributes Java SE 11 (major 55) introduced.
const ATTRIBUTES_JAVA11: &[AttributeRule] = &[
    AttributeRule {
        name: "NestHost",
        locations: CLASS_FILE,
        since: 55,
        source: "JVMS 4.7.28",
    },
    AttributeRule {
        name: "NestMembers",
        locations: CLASS_FILE,
        since: 55,
        source: "JVMS 4.7.29",
    },
];

/// The attribute Java SE 16 (major 60) introduced.
const ATTRIBUTES_JAVA16: &[AttributeRule] = &[AttributeRule {
    name: "Record",
    locations: CLASS_FILE,
    since: 60,
    source: "JVMS 4.7.30",
}];

/// The attribute Java SE 17 (major 61) introduced.
const ATTRIBUTES_JAVA17: &[AttributeRule] = &[AttributeRule {
    name: "PermittedSubclasses",
    locations: CLASS_FILE,
    since: 61,
    source: "JVMS 4.7.31",
}];

/// The tags the format defines at its minimum major.
const TAGS_BASELINE: &[ConstantPoolTagRule] = &[
    ConstantPoolTagRule {
        tag: 1,
        name: "CONSTANT_Utf8",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 3,
        name: "CONSTANT_Integer",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 4,
        name: "CONSTANT_Float",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 5,
        name: "CONSTANT_Long",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 6,
        name: "CONSTANT_Double",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 7,
        name: "CONSTANT_Class",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 8,
        name: "CONSTANT_String",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 9,
        name: "CONSTANT_Fieldref",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 10,
        name: "CONSTANT_Methodref",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 11,
        name: "CONSTANT_InterfaceMethodref",
        since: 45,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 12,
        name: "CONSTANT_NameAndType",
        since: 45,
        source: "JVMS 4.4",
    },
];

/// The tags Java SE 7 (major 51) introduced.
const TAGS_JAVA7: &[ConstantPoolTagRule] = &[
    ConstantPoolTagRule {
        tag: 15,
        name: "CONSTANT_MethodHandle",
        since: 51,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 16,
        name: "CONSTANT_MethodType",
        since: 51,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 18,
        name: "CONSTANT_InvokeDynamic",
        since: 51,
        source: "JVMS 4.4",
    },
];

/// The tags Java SE 9 (major 53) introduced.
const TAGS_JAVA9: &[ConstantPoolTagRule] = &[
    ConstantPoolTagRule {
        tag: 19,
        name: "CONSTANT_Module",
        since: 53,
        source: "JVMS 4.4",
    },
    ConstantPoolTagRule {
        tag: 20,
        name: "CONSTANT_Package",
        since: 53,
        source: "JVMS 4.4",
    },
];

/// The tag Java SE 11 (major 55) introduced.
const TAGS_JAVA11: &[ConstantPoolTagRule] = &[ConstantPoolTagRule {
    tag: 17,
    name: "CONSTANT_Dynamic",
    since: 55,
    source: "JVMS 4.4",
}];

/// The flags Java SE 5 (major 49) introduced.
const FLAGS_JAVA5: &[FlagRule] = &[
    FlagRule {
        name: "ACC_ANNOTATION",
        bits: 0x2000,
        locations: CLASS_FILE,
        since: 49,
        source: "JVMS 4.1",
    },
    FlagRule {
        name: "ACC_BRIDGE",
        bits: 0x0040,
        locations: METHOD_INFO,
        since: 49,
        source: "JVMS 4.6",
    },
    FlagRule {
        name: "ACC_ENUM",
        bits: 0x4000,
        locations: CLASS_FIELD,
        since: 49,
        source: "JVMS 4.1, 4.5",
    },
    FlagRule {
        name: "ACC_SYNTHETIC",
        bits: 0x1000,
        locations: CLASS_FIELD_METHOD,
        since: 49,
        source: "JVMS 4.1, 4.5, 4.6",
    },
    FlagRule {
        name: "ACC_VARARGS",
        bits: 0x0080,
        locations: METHOD_INFO,
        since: 49,
        source: "JVMS 4.6",
    },
];

/// The parameter flags Java SE 8 (major 52) introduced with `MethodParameters`.
const FLAGS_JAVA8: &[FlagRule] = &[
    FlagRule {
        name: "ACC_FINAL",
        bits: 0x0010,
        locations: METHOD_PARAMETER,
        since: 52,
        source: "JVMS 4.7.24",
    },
    FlagRule {
        name: "ACC_MANDATED",
        bits: 0x8000,
        locations: METHOD_PARAMETER,
        since: 52,
        source: "JVMS 4.7.24",
    },
    FlagRule {
        name: "ACC_SYNTHETIC",
        bits: 0x1000,
        locations: METHOD_PARAMETER,
        since: 52,
        source: "JVMS 4.7.24",
    },
];

/// The flag Java SE 9 (major 53) introduced.
const FLAGS_JAVA9: &[FlagRule] = &[FlagRule {
    name: "ACC_MODULE",
    bits: 0x8000,
    locations: CLASS_FILE,
    since: 53,
    source: "JVMS 4.1",
}];

/// The opcodes the format reserves at every release, and the rules that gate the rest.
const OPCODES_BASELINE: &[OpcodeRule] = &[
    OpcodeRule {
        opcode: 0xca,
        name: "breakpoint",
        constraint: OpcodeConstraint::Reserved,
        source: "JVMS 6.2",
    },
    OpcodeRule {
        opcode: 0xfe,
        name: "impdep1",
        constraint: OpcodeConstraint::Reserved,
        source: "JVMS 6.2",
    },
    OpcodeRule {
        opcode: 0xff,
        name: "impdep2",
        constraint: OpcodeConstraint::Reserved,
        source: "JVMS 6.2",
    },
];

/// The opcode rules Java SE 7 (major 51) introduced.
const OPCODES_JAVA7: &[OpcodeRule] = &[
    OpcodeRule {
        opcode: 0xba,
        name: "invokedynamic",
        constraint: OpcodeConstraint::RegisteredFrom(51),
        source: "JVMS 6.5, 4.9.1",
    },
    OpcodeRule {
        opcode: 0xa8,
        name: "jsr",
        constraint: OpcodeConstraint::ForbiddenFrom(51),
        source: "JVMS 4.9.1",
    },
    OpcodeRule {
        opcode: 0xa9,
        name: "ret",
        constraint: OpcodeConstraint::ForbiddenFrom(51),
        source: "JVMS 4.9.1",
    },
];

const VALIDATED_ANY_MINOR: ReleaseBand = ReleaseBand::DialectValidated {
    java8_runtime: Java8RuntimeRule::Accepted,
};
const VALIDATED_JAVA8_MINOR_ZERO: ReleaseBand = ReleaseBand::DialectValidated {
    java8_runtime: Java8RuntimeRule::AcceptedAtMinorZero,
};
const PROBE: ReleaseBand = ReleaseBand::StructuralProbe;
const PROBE_PREVIEW: ReleaseBand = ReleaseBand::StructuralProbeWithPreview {
    preview: PreviewRule {
        marker: PREVIEW_MARKER,
        dialect_support: VersionDialectSupport::UnsupportedPreview,
        source: "JVMS 4.1",
    },
};

/// Constraint classes this registry does not record for the releases 45–52.
const NOTES_LEGACY: &[&str] = &[
    "the historical CONSTANT_Unicode tag (2) and the unused tags 13 and 14 are not registered",
    "ACC_STRICT (0x0800) is not registered",
    "no opcode rule is registered beyond the entries listed here",
];

/// Constraint classes this registry does not record for the releases 53–60.
const NOTES_MODERN: &[&str] = &[
    "attributes JVMS does not define (JDK attributes such as ModuleTarget, ModuleHashes, ModuleResolution, SourceID and CompilationID) are not registered",
    "the combination rules between access flags (JVMS 4.1, 4.5, 4.6) are not registered",
    "ACC_STRICT (0x0800) is not registered",
    "no opcode rule is registered beyond the entries listed here",
    "constraints a release of this band introduces and JVMS 4.7 is not cited for here are not registered",
];

/// Constraint classes this registry does not record for the releases 61–71.
const NOTES_MODERN_AFTER_STRICT: &[&str] = &[
    "attributes JVMS does not define (JDK attributes such as ModuleTarget, ModuleHashes, ModuleResolution, SourceID and CompilationID) are not registered",
    "the combination rules between access flags (JVMS 4.1, 4.5, 4.6) are not registered",
    "the change JEP 306 makes to ACC_STRICT (0x0800) from major 61 on is not registered",
    "no opcode rule is registered beyond the entries listed here",
    "constraints a release of this band introduces and JVMS 4.7 is not cited for here are not registered",
];

/// A release that registers nothing of its own, with the honesty of the band it closes.
const NOTES_NONE: &[&str] = &[
    "this release registers no constraint of its own: only the constraints registered at earlier releases are claimed for it",
    "constraints a release of this band introduces and JVMS 4.7 is not cited for here are not registered",
];
const NOTES_NONE_LEGACY: &[&str] = &[
    "this release registers no constraint of its own: only the constraints registered at earlier releases are claimed for it",
    "the historical CONSTANT_Unicode tag (2) and the unused tags 13 and 14 are not registered",
    "ACC_STRICT (0x0800) is not registered",
];
const NOTES_NONE_MODERN: &[&str] = &[
    "this release registers no constraint of its own: only the constraints registered at earlier releases are claimed for it",
    "attributes JVMS does not define (JDK attributes such as ModuleTarget, ModuleHashes, ModuleResolution, SourceID and CompilationID) are not registered",
    "the combination rules between access flags (JVMS 4.1, 4.5, 4.6) are not registered",
    "ACC_STRICT (0x0800) is not registered",
    "no opcode rule is registered beyond the entries listed here",
];

/// Every release this registry holds, ascending and without gaps.
///
/// Each entry names the constraints its release *introduces*; a lookup for a release applies every
/// record at or below it. The band carries this build's support decision for the release, and
/// `unregistered` states the constraint classes the record deliberately leaves out, so the table
/// cannot be mistaken for a complete statement of the release.
static RELEASES: &[ReleaseRecord] = &[
    ReleaseRecord {
        major: 45,
        band: VALIDATED_ANY_MINOR,
        introduced: IntroducedConstraints {
            constant_pool_tags: TAGS_BASELINE,
            attributes: ATTRIBUTES_BASELINE,
            flags: &[],
            opcodes: OPCODES_BASELINE,
        },
        unregistered: NOTES_LEGACY,
    },
    ReleaseRecord {
        major: 46,
        band: VALIDATED_ANY_MINOR,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE_LEGACY,
    },
    ReleaseRecord {
        major: 47,
        band: VALIDATED_ANY_MINOR,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE_LEGACY,
    },
    ReleaseRecord {
        major: 48,
        band: VALIDATED_ANY_MINOR,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE_LEGACY,
    },
    ReleaseRecord {
        major: 49,
        band: VALIDATED_ANY_MINOR,
        introduced: IntroducedConstraints {
            constant_pool_tags: &[],
            attributes: ATTRIBUTES_JAVA5,
            flags: FLAGS_JAVA5,
            opcodes: &[],
        },
        unregistered: NOTES_LEGACY,
    },
    ReleaseRecord {
        major: 50,
        band: VALIDATED_ANY_MINOR,
        introduced: IntroducedConstraints {
            constant_pool_tags: &[],
            attributes: ATTRIBUTES_JAVA6,
            flags: &[],
            opcodes: &[],
        },
        unregistered: NOTES_LEGACY,
    },
    ReleaseRecord {
        major: 51,
        band: VALIDATED_ANY_MINOR,
        introduced: IntroducedConstraints {
            constant_pool_tags: TAGS_JAVA7,
            attributes: ATTRIBUTES_JAVA7,
            flags: &[],
            opcodes: OPCODES_JAVA7,
        },
        unregistered: NOTES_LEGACY,
    },
    ReleaseRecord {
        major: 52,
        band: VALIDATED_JAVA8_MINOR_ZERO,
        introduced: IntroducedConstraints {
            constant_pool_tags: &[],
            attributes: ATTRIBUTES_JAVA8,
            flags: FLAGS_JAVA8,
            opcodes: &[],
        },
        unregistered: NOTES_LEGACY,
    },
    ReleaseRecord {
        major: 53,
        band: PROBE,
        introduced: IntroducedConstraints {
            constant_pool_tags: TAGS_JAVA9,
            attributes: ATTRIBUTES_JAVA9,
            flags: FLAGS_JAVA9,
            opcodes: &[],
        },
        unregistered: NOTES_MODERN,
    },
    ReleaseRecord {
        major: 54,
        band: PROBE,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE_MODERN,
    },
    ReleaseRecord {
        major: 55,
        band: PROBE,
        introduced: IntroducedConstraints {
            constant_pool_tags: TAGS_JAVA11,
            attributes: ATTRIBUTES_JAVA11,
            flags: &[],
            opcodes: &[],
        },
        unregistered: NOTES_MODERN,
    },
    ReleaseRecord {
        major: 56,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE_MODERN,
    },
    ReleaseRecord {
        major: 57,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE_MODERN,
    },
    ReleaseRecord {
        major: 58,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE_MODERN,
    },
    ReleaseRecord {
        major: 59,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE_MODERN,
    },
    ReleaseRecord {
        major: 60,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints {
            constant_pool_tags: &[],
            attributes: ATTRIBUTES_JAVA16,
            flags: &[],
            opcodes: &[],
        },
        unregistered: NOTES_MODERN,
    },
    ReleaseRecord {
        major: 61,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints {
            constant_pool_tags: &[],
            attributes: ATTRIBUTES_JAVA17,
            flags: &[],
            opcodes: &[],
        },
        unregistered: NOTES_MODERN_AFTER_STRICT,
    },
    ReleaseRecord {
        major: 62,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 63,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 64,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 65,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 66,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 67,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 68,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 69,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 70,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
    ReleaseRecord {
        major: 71,
        band: PROBE_PREVIEW,
        introduced: IntroducedConstraints::NONE,
        unregistered: NOTES_NONE,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn placement_message(diagnostic: &Diagnostic) -> String {
        assert_eq!(diagnostic.provenance, None);
        diagnostic.message.clone()
    }

    #[test]
    fn registry_covers_every_major_between_minimum_and_ceiling() {
        let registry = feature_registry();
        let majors: Vec<u16> = registry.releases().map(|record| record.major).collect();
        let expected: Vec<u16> = (MINIMUM_MAJOR..=HIGHEST_REGISTERED_MAJOR).collect();
        assert_eq!(majors, expected);
        assert_eq!(
            registry.release(MINIMUM_MAJOR),
            ReleaseLookup::Registered(&RELEASES[0])
        );
        assert_eq!(
            registry.release(HIGHEST_REGISTERED_MAJOR).registration(),
            ReleaseRegistration::Registered
        );
        assert_eq!(
            registry.release(MINIMUM_MAJOR - 1),
            ReleaseLookup::UnregisteredBelowMinimum
        );
        assert_eq!(
            registry.release(HIGHEST_REGISTERED_MAJOR + 1),
            ReleaseLookup::UnregisteredFutureRelease
        );
        assert_eq!(registry.release_record(72), None);
        assert_eq!(registry.modern_minor_since(), 56);
    }

    #[test]
    fn unregistered_releases_make_no_claim() {
        let registry = feature_registry();
        for major in [44, 72, 99, u16::MAX] {
            assert_eq!(
                registry.attribute_placement("Record", ClassfileLocation::ClassFile, major),
                AttributePlacement::UnregisteredRelease,
                "major {major}"
            );
            assert_eq!(
                registry.attribute_placement("Record", ClassfileLocation::FieldInfo, major),
                AttributePlacement::UnregisteredRelease,
                "major {major}"
            );
            assert_eq!(
                registry.flag_placement("ACC_MODULE", ClassfileLocation::ClassFile, major),
                FlagPlacement::UnregisteredRelease,
                "major {major}"
            );
            assert_eq!(
                registry.opcode_status(0xba, major),
                OpcodeStatus::UnregisteredRelease,
                "major {major}"
            );
            assert_eq!(
                registry.constant_pool_tag(1, major),
                ConstantPoolTagStatus::UnregisteredRelease,
                "major {major}"
            );
            assert_eq!(registry.attributes(major).count(), 0, "major {major}");
            assert_eq!(registry.flags(major).count(), 0, "major {major}");
            assert_eq!(registry.opcodes(major).count(), 0, "major {major}");
            assert_eq!(
                registry.constant_pool_tags(major).count(),
                0,
                "major {major}"
            );
            assert_eq!(registry.attribute("Record", major), None, "major {major}");
            assert!(registry.unregistered_notes(major).is_empty());
            assert_eq!(
                registry.attribute_diagnostic("Record", ClassfileLocation::ClassFile, major),
                None,
                "major {major}"
            );
            assert_eq!(
                registry.opcode_diagnostic(0xba, major),
                None,
                "major {major}"
            );
        }
        assert_eq!(registry.opcode_diagnostic(0xca, 72), None);
        assert_eq!(
            registry.opcode_diagnostic(0xca, 71).unwrap().code,
            "classfile_opcode_reserved"
        );
    }

    #[test]
    fn modern_attributes_are_bound_to_their_releases() {
        let registry = feature_registry();
        let cases = [
            ("Module", 53, 52),
            ("ModulePackages", 53, 52),
            ("ModuleMainClass", 53, 52),
            ("NestHost", 55, 54),
            ("NestMembers", 55, 54),
            ("Record", 60, 59),
            ("PermittedSubclasses", 61, 60),
        ];
        for (name, since, before) in cases {
            let legal = registry.attribute_placement(
                name,
                ClassfileLocation::ClassFile,
                HIGHEST_REGISTERED_MAJOR,
            );
            assert_eq!(
                legal,
                AttributePlacement::Legal {
                    rule: registry.attribute(name, HIGHEST_REGISTERED_MAJOR).unwrap()
                },
                "{name} at {HIGHEST_REGISTERED_MAJOR}"
            );
            assert_eq!(
                registry
                    .attribute(name, HIGHEST_REGISTERED_MAJOR)
                    .unwrap()
                    .since,
                since,
                "{name} since"
            );
            assert_eq!(
                registry.attribute_placement(name, ClassfileLocation::ClassFile, before),
                AttributePlacement::VersionNotApplicable {
                    rule: registry.attribute(name, HIGHEST_REGISTERED_MAJOR).unwrap()
                },
                "{name} at {before}"
            );
            let diagnostic =
                registry.attribute_diagnostic(name, ClassfileLocation::ClassFile, before);
            let diagnostic = diagnostic.expect("a version violation is diagnosed");
            assert_eq!(
                diagnostic.code,
                "classfile_attribute_version_not_applicable"
            );
            assert_eq!(
                placement_message(&diagnostic),
                format!(
                    "attribute \"{name}\" is registered from major {since} ({}); it is not valid at major {before}",
                    registry
                        .attribute(name, HIGHEST_REGISTERED_MAJOR)
                        .unwrap()
                        .source
                )
            );
        }
    }

    #[test]
    fn attribute_placement_diagnostics_never_guess_from_a_name() {
        let registry = feature_registry();
        let record = registry
            .attribute("Record", HIGHEST_REGISTERED_MAJOR)
            .unwrap();
        assert_eq!(
            registry.attribute_placement("Record", ClassfileLocation::MethodInfo, 60),
            AttributePlacement::LocationNotApplicable {
                rule: record,
                allowed: vec![ClassfileLocation::ClassFile],
            }
        );
        let diagnostic = registry
            .attribute_diagnostic("Record", ClassfileLocation::MethodInfo, 60)
            .expect("a location violation is diagnosed");
        assert_eq!(
            diagnostic.code,
            "classfile_attribute_location_not_applicable"
        );
        assert_eq!(
            placement_message(&diagnostic),
            "attribute \"Record\" is registered only for ClassFile (JVMS 4.7.30); it is not valid in a method_info structure"
        );

        let nest_members = registry
            .attribute("NestMembers", HIGHEST_REGISTERED_MAJOR)
            .unwrap();
        assert_eq!(
            registry.attribute_placement("NestMembers", ClassfileLocation::FieldInfo, 55),
            AttributePlacement::LocationNotApplicable {
                rule: nest_members,
                allowed: vec![ClassfileLocation::ClassFile],
            }
        );

        // A name the registry does not hold is no claim: not legal, and not reported as illegal.
        assert_eq!(
            registry.attribute_placement("BootstrapMethods", ClassfileLocation::Code, 71),
            AttributePlacement::LocationNotApplicable {
                rule: registry.attribute("BootstrapMethods", 71).unwrap(),
                allowed: vec![ClassfileLocation::ClassFile],
            }
        );
        assert_eq!(
            registry.attribute_placement("SomeCustomAttribute", ClassfileLocation::ClassFile, 71),
            AttributePlacement::NotRegistered
        );
        assert_eq!(
            registry.attribute_diagnostic("SomeCustomAttribute", ClassfileLocation::ClassFile, 71),
            None
        );
        assert_eq!(
            registry.attribute_diagnostic("Synthetic", ClassfileLocation::ClassFile, 71),
            None,
            "Synthetic is not registered as an attribute, so it is not reported as a violation"
        );
    }

    #[test]
    fn cumulative_lookups_inherit_earlier_releases() {
        let registry = feature_registry();
        assert!(registry.attribute("Record", 60).is_some());
        assert!(registry.attribute("Record", 59).is_none());
        assert_eq!(
            registry.attributes(61).count(),
            registry.attributes(71).count(),
            "releases 62-71 register no new attribute"
        );
        assert!(
            registry.attributes(60).count() > registry.attributes(59).count(),
            "release 60 introduces an attribute"
        );
        assert_eq!(registry.attributes(45).count(), ATTRIBUTES_BASELINE.len());
        assert_eq!(registry.attributes(44).count(), 0);
        assert_eq!(
            registry.constant_pool_tag(1, 71),
            ConstantPoolTagStatus::Registered {
                rule: &TAGS_BASELINE[0]
            }
        );
        assert_eq!(
            registry.constant_pool_tag(17, 54),
            ConstantPoolTagStatus::VersionNotApplicable {
                rule: &TAGS_JAVA11[0]
            }
        );
        assert_eq!(
            registry.constant_pool_tag(17, 55),
            ConstantPoolTagStatus::Registered {
                rule: &TAGS_JAVA11[0]
            }
        );
        assert_eq!(
            registry.constant_pool_tag(13, 71),
            ConstantPoolTagStatus::Unregistered
        );
    }

    #[test]
    fn flag_placement_covers_version_location_and_unregistered_names() {
        let registry = feature_registry();
        let module = &FLAGS_JAVA9[0];
        assert_eq!(
            registry.flag_placement("ACC_MODULE", ClassfileLocation::ClassFile, 53),
            FlagPlacement::Legal { rule: module }
        );
        assert_eq!(
            registry.flag_placement("ACC_MODULE", ClassfileLocation::ClassFile, 52),
            FlagPlacement::VersionNotApplicable { rule: module }
        );
        let version = registry
            .flag_diagnostic("ACC_MODULE", ClassfileLocation::ClassFile, 52)
            .expect("a flag version violation is diagnosed");
        assert_eq!(version.code, "classfile_flag_version_not_applicable");
        assert_eq!(
            placement_message(&version),
            "access flag ACC_MODULE (0x8000) is registered from major 53 (JVMS 4.1); it is not valid at major 52"
        );

        assert_eq!(
            registry.flag_placement("ACC_MODULE", ClassfileLocation::MethodInfo, 53),
            FlagPlacement::LocationNotApplicable {
                rule: module,
                allowed: vec![ClassfileLocation::ClassFile],
            }
        );
        let location = registry
            .flag_diagnostic("ACC_MODULE", ClassfileLocation::MethodInfo, 53)
            .expect("a flag location violation is diagnosed");
        assert_eq!(location.code, "classfile_flag_location_not_applicable");

        // One name may be registered for several locations by different releases, and the answer
        // reports the location the query asked about, not the newest entry with that name.
        assert_eq!(
            registry.flag_placement("ACC_SYNTHETIC", ClassfileLocation::ClassFile, 71),
            FlagPlacement::Legal {
                rule: &FLAGS_JAVA5[3]
            }
        );
        assert_eq!(
            registry.flag_placement("ACC_SYNTHETIC", ClassfileLocation::MethodParameter, 71),
            FlagPlacement::Legal {
                rule: &FLAGS_JAVA8[2]
            }
        );
        assert_eq!(
            registry.flag_placement("ACC_SYNTHETIC", ClassfileLocation::MethodParameter, 51),
            FlagPlacement::VersionNotApplicable {
                rule: &FLAGS_JAVA8[2]
            }
        );
        assert_eq!(
            registry.flag_placement("ACC_SYNTHETIC", ClassfileLocation::Code, 71),
            FlagPlacement::LocationNotApplicable {
                rule: &FLAGS_JAVA5[3],
                allowed: vec![
                    ClassfileLocation::ClassFile,
                    ClassfileLocation::FieldInfo,
                    ClassfileLocation::MethodInfo,
                    ClassfileLocation::MethodParameter,
                ],
            }
        );

        // ACC_STRICT is not registered: the registry makes no claim about it either way.
        assert_eq!(
            registry.flag_placement("ACC_STRICT", ClassfileLocation::MethodInfo, 71),
            FlagPlacement::NotRegistered
        );
        assert_eq!(
            registry.flag_diagnostic("ACC_STRICT", ClassfileLocation::MethodInfo, 71),
            None
        );
    }

    #[test]
    fn opcode_rules_cover_introduction_forbidding_and_reserved_opcodes() {
        let registry = feature_registry();
        let invokedynamic = &OPCODES_JAVA7[0];
        assert_eq!(
            registry.opcode_status(0xba, 51),
            OpcodeStatus::Legal {
                rule: invokedynamic
            }
        );
        assert_eq!(
            registry.opcode_status(0xba, 50),
            OpcodeStatus::VersionNotApplicable {
                rule: invokedynamic
            }
        );
        let early = registry
            .opcode_diagnostic(0xba, 50)
            .expect("an early opcode is diagnosed");
        assert_eq!(early.code, "classfile_opcode_version_not_applicable");
        assert_eq!(
            placement_message(&early),
            "opcode 0xba (invokedynamic) is registered from major 51 (JVMS 6.5, 4.9.1); it is not valid at major 50"
        );

        let jsr = &OPCODES_JAVA7[1];
        assert_eq!(
            registry.opcode_status(0xa8, 50),
            OpcodeStatus::Legal { rule: jsr }
        );
        assert_eq!(
            registry.opcode_status(0xa8, 51),
            OpcodeStatus::Forbidden { rule: jsr }
        );
        let forbidden = registry
            .opcode_diagnostic(0xa8, 71)
            .expect("a forbidden opcode is diagnosed");
        assert_eq!(forbidden.code, "classfile_opcode_forbidden");
        assert_eq!(
            placement_message(&forbidden),
            "opcode 0xa8 (jsr) is forbidden from major 51 (JVMS 4.9.1); it must not appear at major 71"
        );

        assert_eq!(
            registry.opcode_status(0xca, 45),
            OpcodeStatus::Reserved {
                rule: &OPCODES_BASELINE[0]
            }
        );
        let reserved = registry
            .opcode_diagnostic(0xff, 71)
            .expect("a reserved opcode is diagnosed");
        assert_eq!(reserved.code, "classfile_opcode_reserved");
        assert_eq!(
            placement_message(&reserved),
            "opcode 0xff (impdep2) is reserved by the JVMS (JVMS 6.2); it must not appear in a class file"
        );

        assert_eq!(registry.opcode_status(0x00, 71), OpcodeStatus::Unregistered);
        assert_eq!(registry.opcode_diagnostic(0x00, 71), None);
    }

    #[test]
    fn every_registered_entry_cites_a_source_within_its_release() {
        let registry = feature_registry();
        for record in registry.releases() {
            for rule in record.introduced.constant_pool_tags {
                assert!(rule.source.starts_with("JVMS"), "{rule:?}");
                assert_eq!(
                    rule.since, record.major,
                    "{rule:?} is introduced by its own record"
                );
                assert!(!rule.name.is_empty());
            }
            for rule in record.introduced.attributes {
                assert!(rule.source.starts_with("JVMS"), "{rule:?}");
                assert_eq!(rule.since, record.major, "{rule:?}");
                assert!(!rule.locations.is_empty(), "{rule:?}");
                assert!(!rule.name.is_empty());
            }
            for rule in record.introduced.flags {
                assert!(rule.source.starts_with("JVMS"), "{rule:?}");
                assert_eq!(rule.since, record.major, "{rule:?}");
                assert!(!rule.locations.is_empty(), "{rule:?}");
                assert_ne!(rule.bits, 0, "{rule:?}");
            }
            for rule in record.introduced.opcodes {
                assert!(rule.source.starts_with("JVMS"), "{rule:?}");
                let floor = match rule.constraint {
                    OpcodeConstraint::RegisteredFrom(since)
                    | OpcodeConstraint::ForbiddenFrom(since) => since,
                    OpcodeConstraint::Reserved => MINIMUM_MAJOR,
                };
                assert_eq!(floor, record.major, "{rule:?}");
            }
            assert!(
                !record.unregistered.is_empty(),
                "major {} states no unregistered constraint class",
                record.major
            );
        }
        let mut opcodes: Vec<u8> = registry.opcode_rules().map(|rule| rule.opcode).collect();
        let registered = opcodes.len();
        opcodes.sort_unstable();
        opcodes.dedup();
        assert_eq!(
            opcodes.len(),
            registered,
            "no opcode is registered by two records"
        );
        assert_eq!(
            registry.attribute("Record", 71).unwrap().source,
            "JVMS 4.7.30"
        );
    }

    #[test]
    fn dialect_bands_match_the_releases_they_cover() {
        let registry = feature_registry();
        for record in registry.releases() {
            let preview = record.preview();
            let expected_dialect = if record.major <= 52 {
                VersionDialectSupport::Supported
            } else {
                VersionDialectSupport::StructuralProbeOnly
            };
            assert_eq!(
                record.dialect_support(),
                expected_dialect,
                "{}",
                record.major
            );
            assert_eq!(
                preview.is_some(),
                record.major >= 56,
                "the preview marker is registered from major 56 on"
            );
            if let Some(preview) = preview {
                assert_eq!(preview.marker, PREVIEW_MARKER);
                assert_eq!(
                    preview.dialect_support,
                    VersionDialectSupport::UnsupportedPreview
                );
            }
            let accepted = record.java8_runtime(0) == Java8RuntimeCompatibility::Accepted;
            assert_eq!(accepted, record.major <= 52, "{}", record.major);
            if record.major == 52 {
                assert_eq!(record.java8_runtime(1), Java8RuntimeCompatibility::Rejected);
            }
            assert_eq!(
                record.minor_form(),
                if record.major >= 56 {
                    MinorForm::ZeroOrPreviewMarker
                } else {
                    MinorForm::Unconstrained
                },
                "{}",
                record.major
            );
        }
    }
}
