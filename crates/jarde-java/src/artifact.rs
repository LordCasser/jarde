//! The **artifact binding**: what one recovered artifact *is*, stated as contractual facts, and the
//! one judgement that decides whether evidence may be attached to a text a caller already holds
//! (change `add-demand-driven-core-results`, D3').
//!
//! # Why a binding and not just a text
//!
//! A caller that wants to explain a text it received earlier hands the run an *expectation*: the
//! binding it kept from the first recovery. The run never trusts that value — it computes the
//! binding of the artifact **it** just committed and compares the two — so a value that no trusted
//! read produced can never be mistaken for a cache hit. Anything the comparison does not cover is a
//! dimension on which two different artifacts could collide and be explained by each other's
//! evidence, and the dimensions this binding states are exactly the ones that decide the bytes:
//!
//! | dimension | what it states | what it excludes |
//! | --- | --- | --- |
//! | [`ArtifactBinding::method`] | the **complete physical method identity** — snapshot, container chain, entry ordinal, class-bytes digest and length, syntactic variant, raw name and descriptor | two same-named classes of different content, one class stored twice, one byte-identical class reached through two origins |
//! | [`ArtifactBinding::member_ordinal`] | the member **record** the read established, when it walked the member table — compared when both sides state one | two records of one name and descriptor inside one class |
//! | [`ArtifactBinding::environment`] | the environment identity the run was presented under: runtime view, declared domains, providers and the snapshots the entry provided | one artifact read under another loader, scope or profile view |
//! | the configuration ([`ArtifactBinding::schema`], [`ArtifactBinding::engine`], [`ArtifactBinding::registry`], [`ArtifactBinding::profile`], [`ArtifactBinding::rules`]) | this binding's own schema, the recovery build that wrote the artifact, the reader's dialect registry, the recovery profile and the registered rule table | a rule set, a dialect registry or a format that changed while the textual shape stayed the same |
//! | [`ArtifactBinding::text_digest`] / [`ArtifactBinding::text_bytes`] | a digest of the exact UTF-8 bytes of the artifact, and their length | a text that changed under the same identity and configuration (another debug table, a tighter budget) |
//!
//! # The binding holds facts, never the artifact
//!
//! Nothing here owns a payload: no IR table, no region tree, no AST, no copy of the text. Every
//! field is a contract fact — an identity, a version, a configuration value or a digest — and the
//! type is `Clone + Debug + Eq + PartialEq + Serialize` for that reason: the report it travels in
//! derives the same four, and the payloads this layer reads (`MethodIr`, the region tree, the built
//! program) derive none of `Eq`/`Serialize`, so a binding that kept one would stop the report from
//! being compared and serialized at all. A caller may therefore hold a binding for as long as it
//! likes: it is a value, and the run's own work is released when the run ends.
//!
//! # A short label is never an identity
//!
//! There is deliberately no `class@short`, no display suffix and no name-only accessor on this type.
//! A label is a derived convenience whose spelling depends on the set currently being shown (D09) and
//! it must never be an input to a judgement: two definitions whose readable labels collide are still
//! two artifacts here, because the physical identity — the entry ordinal and the class-bytes digest
//! included — is what is compared.
//!
//! # The digest this layer computes itself
//!
//! The text identity is a digest of the committed text (FNV-1a, 128-bit, over the exact bytes,
//! [`TEXT_DIGEST`]) computed here. `blake3` — the crate the layers below identity a class with — is
//! not a dependency of this crate, and this layer states its own algorithm rather than pretending to
//! reuse one it cannot reach: [`ARTIFACT_SCHEMA`] is bumped when the algorithm changes, so a binding
//! always states which digest produced it.

use jarde_jvm::environment::EnvironmentIdentity;
use jarde_reader::model::{PhysicalClassLocation, PhysicalMethodId};
use jarde_reader::prepared::MethodOrdinal;
use jarde_reader::release_registry::HIGHEST_REGISTERED_MAJOR;
use serde::{Deserialize, Serialize};

use crate::pass::{PASSES, RecoveryProfile};

/// The schema of the binding's own contract: which facts it states, and which algorithm digests the
/// text.
///
/// A binding of another schema is a **mismatch** and never an agreement, exactly as a report of
/// another schema is refused elsewhere in this workspace. Bumped when a dimension is added, removed
/// or re-defined, and when [`TEXT_DIGEST`]'s algorithm changes.
pub const ARTIFACT_SCHEMA: u16 = 1;

/// The algorithm [`ArtifactBinding::text_digest`] states its value with.
pub const TEXT_DIGEST: &str = "fnv1a-128";

/// The code of the one refusal this module states: the artifact this run committed is not the
/// artifact the request named.
///
/// It is a refusal of the **evidence**, not of the run: the artifact of this run is committed, is
/// valid, and is delivered. What does not happen is that this run's evidence is attached to the
/// caller's text.
pub const ARTIFACT_MISMATCH_CODE: &str = "evidence_artifact_mismatch";

/// The code of the refusal of an expectation this entry cannot check at all: the request named an
/// artifact and the run holds no subject to bind its own artifact to.
///
/// This is `Unverifiable` and neither a mismatch nor a delivery: an entry that did not state the
/// identity it wrote its artifact for cannot claim the artifact is the one the caller named, and it
/// must not answer with the evidence of a text nobody compared. It is a *usage* problem of the
/// request — the direct [`crate::recover`] entry with no
/// [`subject`](crate::RecoveryRequest::with_subject) — and it is stated as its own code so it is
/// never read as "the texts agree" or "the texts differ".
pub const ARTIFACT_UNVERIFIABLE_CODE: &str = "evidence_artifact_unverifiable";

/// The code of the refusal of a request that states an expectation and selects nothing.
///
/// The same shape rule the driver range already has: an artifact no category is expanded for
/// explains nothing, so the combination is refused rather than answered with an empty comparison.
pub const EXPECTATION_SHAPE_CODE: &str = "jre_evidence_artifact_shape";

/// What one run's artifact is **of**, as the entry that performed the read states it.
///
/// The entry holds the trusted read: it knows the physical identity the run was bound to, the member
/// record the read established and the environment the run was presented under. Those three are the
/// inputs a binding cannot be built without, and they are the entry's own facts — never the
/// caller's, and never the request's spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactSubject {
    method: PhysicalMethodId,
    member_ordinal: Option<MethodOrdinal>,
    environment: EnvironmentIdentity,
}

impl ArtifactSubject {
    /// The three facts one entry states about the artifact its run is about to commit.
    ///
    /// `member_ordinal` is the record the read **established** — the ordinal the member table walk
    /// located for this member — and `None` when this entry walked no member table at all. It is
    /// never guessed from a position in the request: a class that declares one name and descriptor
    /// twice has two records and no established one, and an entry that walked the table states the
    /// ordinal it located (see [`crate::RecoveryReport::artifact`]).
    pub fn new(
        method: PhysicalMethodId,
        member_ordinal: Option<MethodOrdinal>,
        environment: EnvironmentIdentity,
    ) -> Self {
        Self {
            method,
            member_ordinal,
            environment,
        }
    }

    /// The complete physical identity of the method this run is about.
    pub fn method(&self) -> &PhysicalMethodId {
        &self.method
    }

    /// The member record the entry's read established, when it walked the member table.
    pub fn member_ordinal(&self) -> Option<MethodOrdinal> {
        self.member_ordinal
    }

    /// The environment identity the run was presented under.
    pub fn environment(&self) -> &EnvironmentIdentity {
        &self.environment
    }
}

/// One recovered artifact, stated as the contract facts that identify it.
///
/// The value is produced by the run that committed the artifact ([`crate::recover`], or the entry
/// point that ran it) and is what a caller keeps to ask a later request for the evidence of *that*
/// text. The later request presents it as `expected_artifact`; the later run computes its own
/// binding and compares ([`Self::mismatch`]), and only an agreement attaches evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactBinding {
    schema: u16,
    method: PhysicalMethodId,
    member_ordinal: Option<MethodOrdinal>,
    environment: EnvironmentIdentity,
    engine: String,
    registry: u16,
    profile: RecoveryProfile,
    rules: Vec<String>,
    text_digest: String,
    text_bytes: u64,
}

impl ArtifactBinding {
    /// The binding of one committed artifact: the subject the entry stated, the configuration the
    /// run applied, and the digest of the text it wrote.
    pub(crate) fn of(subject: &ArtifactSubject, profile: &RecoveryProfile, text: &str) -> Self {
        Self {
            schema: ARTIFACT_SCHEMA,
            method: subject.method.clone(),
            member_ordinal: subject.member_ordinal,
            environment: subject.environment.clone(),
            engine: env!("CARGO_PKG_VERSION").to_string(),
            registry: HIGHEST_REGISTERED_MAJOR,
            profile: profile.clone(),
            rules: registered_rules(),
            text_digest: digest_of(text),
            text_bytes: u64::try_from(text.len()).unwrap_or(u64::MAX),
        }
    }

    /// The schema this binding's own facts are stated under.
    pub fn schema(&self) -> u16 {
        self.schema
    }

    /// The complete physical identity of the method this artifact presents.
    pub fn method(&self) -> &PhysicalMethodId {
        &self.method
    }

    /// The member record this artifact was recovered from, when the entry's read established one.
    pub fn member_ordinal(&self) -> Option<MethodOrdinal> {
        self.member_ordinal
    }

    /// The environment identity this artifact was recovered under.
    pub fn environment(&self) -> &EnvironmentIdentity {
        &self.environment
    }

    /// The recovery layer's own build that wrote this artifact.
    ///
    /// A build of another version is a different engine, and two engines do not agree about an
    /// artifact by default: a binding carried across builds is therefore refused unless the build
    /// states the same version. The refusal is the fail-safe direction — an artifact that two builds
    /// *would* have written identically is not explained by a binding one of them wrote.
    pub fn engine(&self) -> &str {
        &self.engine
    }

    /// The reader's dialect registry these facts were parsed under.
    pub fn registry(&self) -> u16 {
        self.registry
    }

    /// The recovery profile (release, multi-release policy, layout) this artifact was written under.
    pub fn profile(&self) -> &RecoveryProfile {
        &self.profile
    }

    /// Every rule this build registers, in table order, as `rule@version`.
    pub fn rules(&self) -> impl Iterator<Item = &str> {
        self.rules.iter().map(String::as_str)
    }

    /// The digest of the exact UTF-8 bytes of this artifact, under [`TEXT_DIGEST`].
    pub fn text_digest(&self) -> &str {
        &self.text_digest
    }

    /// How many bytes the artifact is.
    pub fn text_bytes(&self) -> u64 {
        self.text_bytes
    }

    /// Every dimension on which `expected` and this binding disagree, or `None` when they state the
    /// same artifact.
    ///
    /// The comparison is a field-by-field reading of the contract facts and nothing else: no name,
    /// no label and no position in a member table stands in for an identity. Every differing
    /// dimension is stated, so a caller sees all of them rather than the first one found.
    pub fn mismatch(&self, expected: &Self) -> Option<ArtifactMismatch> {
        let mut dimensions = Vec::new();
        let mut reasons = Vec::new();
        if self.schema != expected.schema {
            dimensions.push(ArtifactDimension::Schema);
            reasons.push(format!(
                "the binding schema (`{}` against `{}`)",
                expected.schema, self.schema
            ));
        }
        if self.method != expected.method {
            dimensions.push(ArtifactDimension::Method);
            reasons.push(format!(
                "the physical method ({} against {})",
                identify(&expected.method),
                identify(&self.method)
            ));
        }
        if let (Some(named), Some(record)) = (expected.member_ordinal, self.member_ordinal)
            && named != record
        {
            // Compared only when **both** sides state a record: an entry that walked no member table
            // states none, and the absence of that evidence is not a conflicting answer about the
            // member. Two stated records must agree — that is the dimension that keeps two records of
            // one name and descriptor apart when a read established either of them.
            dimensions.push(ArtifactDimension::MemberOrdinal);
            reasons.push(format!(
                "the member record (the request named {}, this run established {})",
                ordinal_of(Some(named)),
                ordinal_of(Some(record))
            ));
        }
        if self.environment != expected.environment {
            dimensions.push(ArtifactDimension::Environment);
            reasons.push(format!(
                "the environment (the request named snapshot `{}` under loader `{}`, this run was \
                 presented under snapshot `{}` under loader `{}`)",
                expected.environment.runtime.physical.snapshot.0,
                expected.environment.runtime.load_domain.loader.0,
                self.environment.runtime.physical.snapshot.0,
                self.environment.runtime.load_domain.loader.0,
            ));
        }
        if let Some(reason) = self.configuration_reason(expected) {
            dimensions.push(ArtifactDimension::Configuration);
            reasons.push(reason);
        }
        if self.text_digest != expected.text_digest || self.text_bytes != expected.text_bytes {
            dimensions.push(ArtifactDimension::Text);
            reasons.push(format!(
                "the text (the request named {} byte(s) digested `{}` under {TEXT_DIGEST}, this run \
                 committed {} byte(s) digested `{}`)",
                expected.text_bytes, expected.text_digest, self.text_bytes, self.text_digest
            ));
        }
        if dimensions.is_empty() {
            return None;
        }
        Some(ArtifactMismatch {
            dimensions,
            message: format!(
                "the artifact this run committed is not the one the request named: {}. The run's \
                 own artifact is unaffected and is delivered; the evidence it selected was not \
                 attached to the artifact the request named, because the two are different \
                 artifacts.",
                reasons.join("; ")
            ),
        })
    }

    /// The configuration half of the comparison, as one phrase when it differs.
    ///
    /// Engine, registry, profile and the registered rule table are one dimension because they are one
    /// question — "was this artifact written by the same engine under the same rules" — and the
    /// phrase names every part of it that changed.
    fn configuration_reason(&self, expected: &Self) -> Option<String> {
        let mut changed = Vec::new();
        if self.engine != expected.engine {
            changed.push(format!(
                "the recovery build (`{}` against `{}`)",
                expected.engine, self.engine
            ));
        }
        if self.registry != expected.registry {
            changed.push(format!(
                "the reader's dialect registry ({} against {})",
                expected.registry, self.registry
            ));
        }
        if self.profile != expected.profile {
            changed.push(format!(
                "the recovery profile (release {}, {}, {} against release {}, {}, {})",
                expected.profile.java_release,
                spell(&expected.profile),
                layout(&expected.profile),
                self.profile.java_release,
                spell(&self.profile),
                layout(&self.profile),
            ));
        }
        if self.rules != expected.rules {
            changed.push(format!(
                "the registered rule table ({} rule(s) against {} rule(s), first difference: `{}` \
                 against `{}`)",
                expected.rules.len(),
                self.rules.len(),
                first_difference(&expected.rules, &self.rules).unwrap_or("none"),
                first_difference(&self.rules, &expected.rules).unwrap_or("none"),
            ));
        }
        (!changed.is_empty()).then(|| {
            format!(
                "the configuration the artifact was written under ({})",
                changed.join(", ")
            )
        })
    }
}

/// One contract dimension of [`ArtifactBinding`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactDimension {
    /// The binding's own schema.
    Schema,
    /// The physical identity of the method.
    Method,
    /// The member record the read established.
    MemberOrdinal,
    /// The environment the run was presented under.
    Environment,
    /// Schema, reader facts, profile and registered rules.
    Configuration,
    /// The exact text.
    Text,
}

impl ArtifactDimension {
    /// Every dimension, in the order [`ArtifactMismatch`] states them.
    pub const ALL: [Self; 6] = [
        Self::Schema,
        Self::Method,
        Self::MemberOrdinal,
        Self::Environment,
        Self::Configuration,
        Self::Text,
    ];

    /// The dimension's own name, as a report or a diagnostic states it.
    pub fn spell(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Method => "method",
            Self::MemberOrdinal => "member_ordinal",
            Self::Environment => "environment",
            Self::Configuration => "configuration",
            Self::Text => "text",
        }
    }
}

/// The one refusal this module states: the artifact the request named is not the artifact the run
/// committed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArtifactMismatch {
    dimensions: Vec<ArtifactDimension>,
    message: String,
}

impl ArtifactMismatch {
    /// Every dimension that differed, in [`ArtifactDimension::ALL`] order.
    pub fn dimensions(&self) -> &[ArtifactDimension] {
        &self.dimensions
    }

    /// Whether one dimension is among them.
    pub fn disagrees_on(&self, dimension: ArtifactDimension) -> bool {
        self.dimensions.contains(&dimension)
    }

    /// The refusal, stated in one sentence: what differed, and that this run's own artifact stands.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The code this refusal is registered under.
    pub fn code(&self) -> &'static str {
        ARTIFACT_MISMATCH_CODE
    }
}

/// What one run states about the artifact it committed and about the artifact its request named.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ArtifactAgreement {
    /// The request named no artifact: the ordinary first recovery. Nothing was compared, and
    /// nothing had to be.
    NotStated,
    /// The request named an artifact and this run's artifact **is** it. The evidence this run
    /// materialized describes it, so the selection was answered.
    Agreed,
    /// The request named an artifact and this run committed a different one. Nothing of the
    /// selection was materialized: this run's evidence describes *this* artifact, and attaching it
    /// to the caller's would be the silent re-pointing this contract exists to prevent.
    Mismatched {
        /// Every dimension that differed.
        mismatch: ArtifactMismatch,
    },
    /// The request named an artifact and there was nothing to compare it with: the run stopped
    /// before an artifact was committed, or the entry held no subject for the artifact it did
    /// commit. Never a claim that the two agree.
    Unverifiable {
        /// Why the expectation could not be checked, as one sentence.
        reason: String,
    },
}

impl ArtifactAgreement {
    /// The code a report states this verdict under, when it is a refusal.
    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::NotStated | Self::Agreed => None,
            Self::Mismatched { .. } => Some(ARTIFACT_MISMATCH_CODE),
            Self::Unverifiable { .. } => Some(ARTIFACT_UNVERIFIABLE_CODE),
        }
    }

    /// Whether the evidence a request selected may be materialized: there was nothing to check, or
    /// the check agreed.
    pub fn attaches(&self) -> bool {
        matches!(self, Self::NotStated | Self::Agreed)
    }

    /// The one sentence a diagnostic states this verdict with, when it is a refusal.
    pub fn refusal(&self) -> Option<&str> {
        match self {
            Self::NotStated | Self::Agreed => None,
            Self::Mismatched { mismatch } => Some(mismatch.message()),
            Self::Unverifiable { reason } => Some(reason),
        }
    }
}

/// What one report states about the artifact its run committed, and about the artifact its request
/// named.
///
/// The two halves answer two different questions and neither stands in for the other: `binding` is
/// this run's own product identity — a caller keeps it to explain this text later — and `agreement`
/// is what this run decided about the artifact the request named. The binding is `None` exactly when
/// there is no artifact: the run stopped before one was committed, or the entry held no subject to
/// bind the one it produced.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveryArtifact {
    binding: Option<ArtifactBinding>,
    agreement: ArtifactAgreement,
}

impl RecoveryArtifact {
    /// An artifact this run committed, under the entry's own subject, judged against `expected`.
    pub(crate) fn of(binding: ArtifactBinding, expected: Option<&ArtifactBinding>) -> Self {
        let agreement = match expected {
            None => ArtifactAgreement::NotStated,
            Some(expected) => match binding.mismatch(expected) {
                None => ArtifactAgreement::Agreed,
                Some(mismatch) => ArtifactAgreement::Mismatched { mismatch },
            },
        };
        Self {
            binding: Some(binding),
            agreement,
        }
    }

    /// An artifact the entry held no subject for: the text is delivered, and no binding is claimed.
    ///
    /// A request that named an artifact gets `Unverifiable` — never an agreement — because a run that
    /// was not told the identity it wrote for cannot claim the caller's text is the one it produced.
    pub(crate) fn unbound(expected: Option<&ArtifactBinding>) -> Self {
        let agreement = match expected {
            None => ArtifactAgreement::NotStated,
            Some(_) => ArtifactAgreement::Unverifiable {
                reason: "the request named an artifact to attach its evidence to, and this entry \
                         stated no subject for the artifact it produced, so the two were not \
                         compared"
                    .to_owned(),
            },
        };
        Self {
            binding: None,
            agreement,
        }
    }

    /// A run that committed no artifact: there is nothing to bind and nothing to compare.
    pub(crate) fn not_produced(expected: Option<&ArtifactBinding>) -> Self {
        let agreement = match expected {
            None => ArtifactAgreement::NotStated,
            Some(_) => ArtifactAgreement::Unverifiable {
                reason:
                    "the run stopped before an artifact was committed, so there is no artifact \
                         to check the one the request named against"
                        .to_owned(),
            },
        };
        Self {
            binding: None,
            agreement,
        }
    }

    /// The contract facts of this run's artifact, when it committed one.
    pub fn binding(&self) -> Option<&ArtifactBinding> {
        self.binding.as_ref()
    }

    /// What this run states about the artifact its request named.
    pub fn agreement(&self) -> &ArtifactAgreement {
        &self.agreement
    }

    /// Whether the evidence this run selected was attached to the artifact the request named.
    pub fn attaches(&self) -> bool {
        self.agreement.attaches()
    }
}

impl Default for RecoveryArtifact {
    /// The statement of a run that has not happened: no binding, nothing compared.
    fn default() -> Self {
        Self {
            binding: None,
            agreement: ArtifactAgreement::NotStated,
        }
    }
}

/// Every rule this build registers, in table order, as `rule@version`.
///
/// The rule table is part of what decides an artifact's text, so the binding states it: a build
/// whose rules changed (a rule version moved, a rule left, a rule joined) produces bindings that do
/// not agree with the ones its predecessor produced, whatever the bytes are.
fn registered_rules() -> Vec<String> {
    PASSES.iter().map(|pass| pass.rule().to_string()).collect()
}

/// The digest of one artifact's **exact** UTF-8 bytes, under [`TEXT_DIGEST`].
///
/// FNV-1a, 128-bit: the published offset basis and prime, one xor and one wrapping multiply per
/// byte, no normalization of any kind. Two texts that differ in one byte differ here, which is the
/// whole job of the value: it decides whether the text a caller holds is the text this run wrote,
/// and the identity, environment and configuration dimensions decide everything else.
pub(crate) fn digest_of(text: &str) -> String {
    const OFFSET_BASIS: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;
    let mut hash = OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:032x}")
}

/// One physical method, spelled by the facts that identify it and by nothing else.
///
/// This is a *message* rendering of the identity — snapshot, entry ordinal, class-bytes digest,
/// variant, raw name and descriptor — and never a label a judgement could read: the comparison
/// above compares the values themselves.
fn identify(method: &PhysicalMethodId) -> String {
    let owner = &method.owner;
    let place = match &owner.location {
        PhysicalClassLocation::StandaloneRoot { snapshot } => {
            format!("the standalone class of snapshot `{}`", snapshot.0)
        }
        PhysicalClassLocation::ArchiveEntry { entry } => format!(
            "entry #{} `{}` of container `{}` in snapshot `{}`",
            entry.ordinal,
            String::from_utf8_lossy(&entry.raw_name.0),
            entry.container().0,
            entry.snapshot().0
        ),
    };
    let variant = match &owner.variant {
        jarde_reader::model::PhysicalVariant::Base => "base".to_string(),
        jarde_reader::model::PhysicalVariant::MultiRelease { version } => {
            format!("multi-release {version}")
        }
        jarde_reader::model::PhysicalVariant::Other { label } => format!("variant `{label}`"),
    };
    format!(
        "{place} ({variant}), class bytes {} byte(s) digested `{}`, member `{}` `{}`",
        owner.class_bytes.length,
        owner.class_bytes.digest.0,
        String::from_utf8_lossy(&method.name.0),
        String::from_utf8_lossy(&method.descriptor.0),
    )
}

/// One member record, as a message states it.
fn ordinal_of(ordinal: Option<MethodOrdinal>) -> String {
    match ordinal {
        Some(ordinal) => format!("record #{}", ordinal.0),
        None => "no record".to_string(),
    }
}

/// The multi-release policy of a profile, as a phrase.
fn spell(profile: &RecoveryProfile) -> String {
    format!("{:?}", profile.multi_release)
}

/// The layout of a profile, as a phrase.
fn layout(profile: &RecoveryProfile) -> String {
    format!("{:?}", profile.layout)
}

/// The first rule one table holds that the other does not, in table order.
fn first_difference<'a>(left: &'a [String], right: &[String]) -> Option<&'a str> {
    left.iter()
        .find(|rule| !right.contains(rule))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    //! What one binding states, and what one judgement about it states: the two are the contract this
    //! module exists for, and both are values a test can build — the subjects below are the identity
    //! facts a read establishes, and the texts are the artifacts a run commits.

    use super::*;
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalVariant, SnapshotId,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
        MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
        RuntimeView,
    };

    fn subject() -> ArtifactSubject {
        let snapshot = SnapshotId("snapshot".to_string());
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest("class".to_string()),
                length: 532,
            },
            variant: PhysicalVariant::Base,
        };
        let domain = LoadDomain {
            loader: LoaderId("app".to_string()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        ArtifactSubject::new(
            PhysicalMethodId {
                owner: definition,
                name: JvmBytes(b"value".to_vec()),
                descriptor: JvmBytes(b"()I".to_vec()),
            },
            Some(MethodOrdinal(3)),
            EnvironmentIdentity {
                runtime: RuntimeView {
                    physical: PhysicalView {
                        snapshot,
                        scope: PhysicalScope::SnapshotAll,
                    },
                    profile: RuntimeProfile {
                        java_release: 8,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    load_domain: domain.clone(),
                },
                domain_loaders: vec![domain.loader.clone()],
                providers: Vec::new(),
                content: Vec::new(),
            },
        )
    }

    fn binding_of(text: &str) -> ArtifactBinding {
        ArtifactBinding::of(&subject(), &crate::pass::JAVA_8, text)
    }

    /// Two texts of the same length are two different artifacts: the digest is what tells them apart,
    /// and a length, a name or a bytecode index would not.
    #[test]
    fn the_text_identity_is_a_digest_and_not_a_length() {
        assert_eq!(
            digest_of(""),
            "6c62272e07bb014262b821756295c58d",
            "the published FNV-1a 128-bit offset basis, which is what the empty text digests to"
        );
        assert_eq!(digest_of("return 1;").len(), 32);
        assert_eq!(
            digest_of("return 1;"),
            digest_of("return 1;"),
            "the digest of one text is one value"
        );
        assert_ne!(digest_of("return 1;"), digest_of("return 2;"));
        assert_ne!(
            digest_of("return aa;"),
            digest_of("return bb;"),
            "two texts of one length are still two texts"
        );

        let one = binding_of("return aa;");
        let other = binding_of("return bb;");
        assert_eq!(one.text_bytes(), other.text_bytes(), "same length");
        assert_ne!(one.text_digest(), other.text_digest(), "different digest");
        let mismatch = one.mismatch(&other).expect("the two artifacts differ");
        assert_eq!(
            mismatch.dimensions(),
            [ArtifactDimension::Text],
            "and the text is the one dimension that changed"
        );
    }

    /// Every dimension is compared, and each one is reachable on its own.
    #[test]
    fn every_contract_dimension_is_compared() {
        let base = binding_of("return 1;");
        assert!(
            base.mismatch(&base).is_none(),
            "one binding agrees with itself"
        );

        let mut other_schema = base.clone();
        other_schema.schema = ARTIFACT_SCHEMA + 1;
        assert_eq!(
            base.mismatch(&other_schema).expect("schema").dimensions(),
            [ArtifactDimension::Schema]
        );

        let mut other_method = base.clone();
        other_method.method.name = JvmBytes(b"other".to_vec());
        assert_eq!(
            base.mismatch(&other_method).expect("method").dimensions(),
            [ArtifactDimension::Method]
        );

        let mut other_ordinal = base.clone();
        other_ordinal.member_ordinal = Some(MethodOrdinal(4));
        assert_eq!(
            base.mismatch(&other_ordinal)
                .expect("member record")
                .dimensions(),
            [ArtifactDimension::MemberOrdinal]
        );
        // An entry that walked no member table states no record, and stating none is not a
        // conflicting answer: the artifact is the same one, described with one fact less.
        let mut no_record = base.clone();
        no_record.member_ordinal = None;
        assert!(
            base.mismatch(&no_record).is_none(),
            "the absence of the record evidence is not a disagreement about the member"
        );

        let mut other_environment = base.clone();
        other_environment
            .environment
            .content
            .push(SnapshotId("x".to_string()));
        assert_eq!(
            base.mismatch(&other_environment)
                .expect("environment")
                .dimensions(),
            [ArtifactDimension::Environment]
        );

        let mut other_profile = base.clone();
        other_profile.profile.java_release = 7;
        assert_eq!(
            base.mismatch(&other_profile).expect("profile").dimensions(),
            [ArtifactDimension::Configuration]
        );

        let mut other_rules = base.clone();
        other_rules.rules.push("invented@1".to_string());
        let mismatch = base.mismatch(&other_rules).expect("rule table");
        assert_eq!(mismatch.dimensions(), [ArtifactDimension::Configuration]);
        assert!(
            mismatch.message().contains("invented@1"),
            "the refusal names what changed: {}",
            mismatch.message()
        );

        let mut other_text = base.clone();
        other_text.text_digest = digest_of("return 1; ");
        assert_eq!(
            base.mismatch(&other_text).expect("text").dimensions(),
            [ArtifactDimension::Text]
        );

        // Every differing dimension is stated at once, in one order, rather than the first found.
        let mut everything = base.clone();
        everything.schema = ARTIFACT_SCHEMA + 1;
        everything.member_ordinal = Some(MethodOrdinal(4));
        everything.text_bytes = 0;
        assert_eq!(
            base.mismatch(&everything).expect("everything").dimensions(),
            [
                ArtifactDimension::Schema,
                ArtifactDimension::MemberOrdinal,
                ArtifactDimension::Text
            ]
        );
    }

    /// The binding states the rule table this build registers, and the schema it states its own facts
    /// under: a rule set that moved and a binding that moved are two different things, and both are
    /// stated.
    #[test]
    fn the_configuration_states_the_rule_table_this_build_registers() {
        let binding = binding_of("return 1;");
        assert_eq!(binding.schema(), ARTIFACT_SCHEMA);
        assert_eq!(binding.profile(), &crate::pass::JAVA_8);
        assert_eq!(binding.engine(), env!("CARGO_PKG_VERSION"));
        assert_eq!(binding.registry(), HIGHEST_REGISTERED_MAJOR);
        let rules: Vec<&str> = binding.rules().collect();
        assert_eq!(
            rules.len(),
            PASSES.len(),
            "every registered rule is stated once"
        );
        assert_eq!(rules[0], PASSES[0].rule().to_string());
        assert!(rules.iter().all(|rule| rule.contains('@')));
    }
}
