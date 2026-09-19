//! Versioned framework/resource query plugins (P4 3.1/3.2): a derived-fact plane beside the
//! structural XRef scan.
//!
//! # What a plugin is here
//!
//! A plugin is a **registered rule** of this engine, not a code-loading extension point: [`PLUGINS`]
//! is a table of [`PluginRule`] descriptors, and a request enables one of them **by id and by
//! version**. Every descriptor states the whole contract the P4 requirement asks for — its id, its
//! rule version, the input category it reads, the config path inside the snapshot it recognizes,
//! the versioned schema of what it publishes, the evidence one item carries, what its coverage
//! claims, the budget category that bounds it, the trust boundary it runs under, the source its
//! rule was read from, and what it deliberately does not claim (`not_claimed`, never empty).
//!
//! A rule may be **registered without being performed** ([`PluginSupport::Unsupported`]): the
//! boundary is then a readable record with a reason instead of an omission. A configuration this
//! registry does not hold — an id it does not know, or a version of an id it does — is
//! **`Unsupported`**, never an empty answer: "this archive declares nothing" and "this engine does
//! not read that configuration" are different statements, and only the first one is a statement
//! this plane is in a position to make. The states are [`PluginRuleAnalysis`]'s, and the refusal
//! codes are what a test pins.
//!
//! # What this plane does not change
//!
//! A plugin publishes **derived facts** ([`PluginItem`]) on a plane of its own. It cannot write
//! back into the P1/P2 structural facts: this plane has no `QueryRelation`, no `XrefItem` and no
//! X1 edge, its items are stamped with the rule id and version that produced them, and a request
//! that enables a rule leaves the snapshot's own enumeration byte for byte what it was. The generic
//! structural scan stays what it was — target-driven, cursor-bound, and answering a different
//! question ("does this artifact mention this symbol") — which is exactly why the framework strings
//! this plane interprets never enter it (P4 design decision 4; Risk 4: plugin rules must not
//! pollute the core result).
//!
//! # The trust boundary, stated once
//!
//! A plugin of this build is ordinary code linked into this process ([`PLUGIN_TRUST_DOMAIN`]).
//! What keeps it inside its contract is that its input is **the authorized snapshot content
//! only**, reached through this engine's own budgeted read face, and that reading it is all it
//! does: no byte of the archive is executed, no class is loaded, linked or initialized, no
//! launcher or agent is started, no JNI library is loaded, no bootstrap is called and no network
//! resource is fetched — a declaration that names a class this snapshot does not even hold is
//! published as the name it spells. **A Rust function is not a sandbox**, and nothing here claims
//! one: an untrusted extension must run in a **separate process** or in **Wasm** with only the
//! authorized input handed to it, and publishing such an extension is a different change with its
//! own isolation work — not this registry, and not a trait this crate hands out today.
//!
//! # Bounds and coverage
//!
//! The budget category a rule declares is the one that really bounds it: one `result_items` per
//! published item, charged **before** it is published (the 2.2/2.3 discipline), plus the input's
//! own read charges and the request's elapsed-time limit, polled while the configuration is
//! parsed. A refusal is a **stop**, never a wrong answer: the rule keeps the prefix it published,
//! its analysis becomes [`PluginRuleAnalysis::Skipped`] with the dimension that refused, its
//! coverage becomes `Partial`, and the entries it did not read are named there — which is what
//! "the range that was read" means for the P4 3.2 scenario. The independent structural query is a
//! different request with its own budget, and it completes over its own facts regardless.
//!
//! Two stops are not one statement, and both are published. A refusal **inside** a rule — its own
//! read, its own parse, its own item charge — makes that rule [`PluginRuleAnalysis::Skipped`] with
//! the dimension that refused. A refusal of the snapshot's own **listing** leaves every rule a
//! prefix to read: its coverage is `Partial`, its `has_more` is set, and the request's execution
//! names the limit that stopped the listing while the listing's own diagnostic is published beside
//! it. A rule that never started because an earlier rule's request stopped is `NotRequested` with
//! `plugin_request_stopped`: a rule that made no claim says so.

use std::fmt;

use serde::{Deserialize, Serialize};

use jarde_reader::artifact::{
    ArtifactKind, ArtifactSnapshot, EnumerationReport, PhysicalEntry, budget_dimension_code,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    ByteSpan, Coverage, CoverageDimension, CoverageRange, CoverageState, Diagnostic,
    DiagnosticSeverity, ExecutionReport, JvmBytes, Location, PhysicalEntryId, Provenance,
    TerminationReason,
};
use jarde_reader::view::{PhysicalScope, PhysicalView};

use crate::xref::escaped_raw_name;
use crate::xref::resource::{SERVICES_PREFIX, service_key, service_providers};
use crate::xref::to_u64;

/// What the trust boundary of every rule in this registry is, stated once.
///
/// Every [`PluginRule`] carries this statement, so a reader of one descriptor cannot read a plugin
/// as something it is not. It says the two things the P4 requirement needs said: what a plugin may
/// read (the authorized snapshot input, through this engine's budgeted read face) and what a
/// *future* untrusted extension would need (a separate process or Wasm — **not** this registry,
/// because a Rust function is not a sandbox).
pub const PLUGIN_TRUST_DOMAIN: &str = "same trust domain as the engine: ordinary in-process code \
     that reads the authorized snapshot input through the engine's own budgeted read face, and \
     nothing else — no target byte is executed, no class is loaded and no network resource is \
     fetched. A Rust function is not a sandbox: an untrusted extension must run in a separate \
     process or in Wasm with only that input handed over, under its own change.";

/// The name and version of one registered plugin rule, spelled `name@version`.
///
/// The spelling is the engine's convention — the recovery layer's `RuleVersion` and the JVM
/// layer's own mirror read the same way — so a reader of any report reads one convention. The type
/// is this crate's own because a plugin rule is a fact about *this* registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct PluginRuleVersion {
    name: &'static str,
    version: &'static str,
}

impl PluginRuleVersion {
    /// The version of one registered rule.
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Self { name, version }
    }

    /// The rule's name.
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// The rule's version.
    pub const fn version(self) -> &'static str {
        self.version
    }
}

impl fmt::Display for PluginRuleVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.name, self.version)
    }
}

/// Which authorized input of a snapshot one rule reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginInputCategory {
    /// The decompressed content of one archive entry whose raw name matches the rule's config
    /// path. Nothing else of the snapshot is an input of this plane: no class header, no constant
    /// pool, no method body, no bootstrap attribute.
    ArchiveResourceEntry,
}

/// The path inside a snapshot one rule recognizes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginConfigPath {
    /// The leading bytes of an entry's raw name this rule recognizes, compared ASCII
    /// case-insensitively the way a ZIP lookup is; the raw bytes are preserved and never
    /// normalized.
    pub prefix: &'static str,
    /// What one matching entry declares, in one sentence.
    pub declares: &'static str,
}

/// The versioned shape of what one rule publishes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginOutputSchema {
    /// Stable schema name, as a report would spell it.
    pub name: &'static str,
    /// The schema version. A consumer reads an item only under the version its own rule declares,
    /// so a shape change is a new version and never a silent reinterpretation.
    pub version: u16,
    /// The item's fields, as the tests read them off a serialized item. A rule this build does not
    /// perform declares its schema name and version and no fields: the shape of a rule that is not
    /// performed is not a claim this build makes.
    pub fields: &'static [&'static str],
}

/// The budget category that bounds one rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginBudget {
    /// The counted dimension whose limit bounds how many items the rule publishes; one item costs
    /// one unit, charged before it is published.
    pub dimension: CountedBudgetDimension,
    /// What else bounds one run of the rule: the artifact read dimensions the declared input
    /// itself costs (`archive_entries`, `entry_bytes`, `read_bytes`) and the request's
    /// elapsed-time limit, which the rule polls while it parses the configuration.
    pub statement: &'static str,
}

/// Whether this build performs one registered rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PluginSupport {
    /// The rule's declared input is read and its declared schema is published.
    Performed,
    /// The rule is registered and this build deliberately does not perform it. The reason is what
    /// a reader checks instead of an omission.
    Unsupported {
        /// Why this build does not perform the rule.
        reason: &'static str,
    },
}

/// One registered plugin rule: the whole contract of one configuration.
///
/// A record states what it *does* claim (its input, its config path, its schema, its evidence, its
/// coverage, its budget and its support) and, in `not_claimed`, what it deliberately does not —
/// the shape the release registry and the X3 pattern table give their own entries, so a boundary
/// is a readable record rather than an absence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginRule {
    /// Stable rule id, as a request and a report spell it.
    pub id: &'static str,
    /// The rule version this descriptor *is*.
    pub rule: PluginRuleVersion,
    /// Which authorized input the rule reads.
    pub input: PluginInputCategory,
    /// The path inside the snapshot the rule recognizes.
    pub config_path: PluginConfigPath,
    /// The versioned shape of the results the rule publishes.
    pub schema: PluginOutputSchema,
    /// What one published item's evidence is.
    pub evidence: &'static str,
    /// What the rule's own coverage claims.
    pub coverage: &'static str,
    /// The budget category that bounds the rule.
    pub budget: PluginBudget,
    /// Where the rule was read from.
    pub source: &'static str,
    /// The trust boundary the rule runs under ([`PLUGIN_TRUST_DOMAIN`]). Never empty.
    pub isolation: &'static str,
    /// What this rule deliberately does not claim. Never empty.
    pub not_claimed: &'static [&'static str],
    /// Whether this build performs the rule.
    pub support: PluginSupport,
}

/// The registered plugin rules, in registry order.
///
/// The table is the whole vocabulary of this plane: a request enables a rule by id **and** version,
/// and a configuration the table does not hold — at the id or at the version — is `Unsupported`.
/// Two ids cannot share one version, and `PLUGINS` is the only place a rule is declared.
pub const PLUGINS: &[PluginRule] = &[
    PluginRule {
        id: "service-loader-registrations",
        rule: PluginRuleVersion::new("service-loader-registrations", "1"),
        input: PluginInputCategory::ArchiveResourceEntry,
        config_path: PluginConfigPath {
            prefix: "META-INF/services/",
            declares: "one provider class per line for the service interface the entry name \
                       carries, in the syntax of a provider-configuration file (`#` starts a \
                       comment, blank lines are ignored, a name ending in `.` continues on the \
                       next line)",
        },
        schema: PluginOutputSchema {
            name: "plugin.service_provider",
            version: 1,
            fields: &["rule", "rule_version", "source_entry", "matched", "value"],
        },
        evidence: "the entry the declaration was read from (its container origin, its ordinal and \
                   its raw name bytes) and the byte range inside that entry's decompressed content \
                   that holds the declared name; the name is the raw bytes the archive stores, not \
                   a decoded or resolved identifier",
        coverage: "every entry of the scope whose raw name matches the config path is either read \
                   (and listed in `scanned` with the content range that was read) or named in \
                   `skipped` because the scan stopped before it; a rule that was not performed \
                   makes no coverage claim at all",
        budget: PluginBudget {
            dimension: CountedBudgetDimension::ResultItems,
            statement: "one `result_items` per published declaration, charged before it is \
                        published, plus the input's own read charges (`archive_entries`, \
                        `entry_bytes`, `read_bytes`) and the request's elapsed-time limit, polled \
                        while the configuration is parsed",
        },
        source: "JDK javadoc: java.util.ServiceLoader, provider-configuration files \
                 (`META-INF/services/<service interface>`)",
        isolation: PLUGIN_TRUST_DOMAIN,
        not_claimed: &[
            "no class a declaration names is read, decoded, resolved, loaded or initialized: the \
             name the file spells is the whole answer, and a name this snapshot does not hold is \
             still published as declared",
            "the declarations are not a call of `ServiceLoader.load`: no thread context loader, no \
             provider-configuration order and no activation statement is made here — the 2.3 \
             pattern plane names the interface a call site asks for, and the two are separate \
             facts",
            "a line this engine cannot read as a name is not repaired, decoded or guessed",
        ],
        support: PluginSupport::Performed,
    },
    PluginRule {
        id: "spring-factories",
        rule: PluginRuleVersion::new("spring-factories", "1"),
        input: PluginInputCategory::ArchiveResourceEntry,
        config_path: PluginConfigPath {
            prefix: "META-INF/spring.factories/",
            declares: "a properties document whose keys are interface names and whose values are \
                       comma-separated class lists",
        },
        schema: PluginOutputSchema {
            name: "plugin.spring_factory",
            version: 1,
            fields: &[],
        },
        evidence: "not stated: this build performs no read for this rule, so no item and no \
                   evidence exists",
        coverage: "no coverage: the rule is registered as unsupported and this build reads \
                   nothing for it",
        budget: PluginBudget {
            dimension: CountedBudgetDimension::ResultItems,
            statement: "declared for a rule this build does not perform: the request is refused \
                        before any input is read, so nothing is charged",
        },
        source: "Spring Framework reference documentation: `META-INF/spring.factories`",
        isolation: PLUGIN_TRUST_DOMAIN,
        not_claimed: &[
            "no `spring.factories` value is read as a class name by this build: the format is a \
             properties document whose values may be comma-separated lists, and reading one as a \
             precise class reference is exactly the interpretation the P4 requirement refuses",
        ],
        support: PluginSupport::Unsupported {
            reason: "the configuration is a properties document with comma-separated class lists; \
                     this build registers the rule so the boundary is readable, and performing it \
                     is another change with its own schema version",
        },
    },
];

/// Every registered plugin rule, in registry order.
pub const fn plugins() -> &'static [PluginRule] {
    PLUGINS
}

/// The rule one request enables, or `None` when this registry holds no rule under that id **and**
/// version.
///
/// The comparison is byte-exact on both dimensions. An unregistered configuration is a different
/// statement from a registered rule this build does not perform: the first is `Unsupported` with
/// no descriptor behind it, the second is `Unsupported` with the rule's own reason.
pub fn plugin_for(id: &str, version: &str) -> Option<&'static PluginRule> {
    PLUGINS
        .iter()
        .find(|rule| rule.id == id && rule.rule.version() == version)
}

/// One enabled rule: the id and the exact version the caller enables.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginSelection {
    pub id: String,
    pub version: String,
}

/// One plugin request: the physical view to scan and the rules to enable.
///
/// The scope of this plane is the snapshot's own root container, which is what
/// [`PhysicalScope::SnapshotAll`] names; a rule is registered over that range only, and a nested
/// tree scope is a `NotRequested` state rather than a silently different scan.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginRequest {
    pub physical: PhysicalView,
    /// The rules the caller enables, each at the exact version it enables, in report order. An
    /// empty list is an input error (`plugin_no_rules`): this plane answers the rule a caller
    /// names, and "no rule enabled" is not a scan.
    pub rules: Vec<PluginSelection>,
    /// `0` means "no item limit". An item limit is not a cursor: a truncated rule replays nothing.
    pub max_items: u64,
}

/// Whether one rule ran, was refused, or stopped under the budget.
///
/// `Performed` is the state of a rule that read its declared input, **including** one that found no
/// declaration at all: its coverage names the entries it read, which is what makes "nothing was
/// declared" a covered statement. The three other states claim no range, and their codes are what a
/// consumer switches on.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum PluginRuleAnalysis {
    /// The rule ran over the entries of this request's scope.
    Performed,
    /// The configuration this request enables is not a rule of this registry — neither at the
    /// enabled id nor at the enabled version — so nothing was read and no coverage is claimed.
    Unsupported {
        /// Stable refusal code.
        code: &'static str,
        /// The refusal in one sentence, naming what this registry does hold instead.
        message: String,
    },
    /// The rule is registered and this request is not a subject of it: the input it reads is not
    /// part of this snapshot, or of this scope. Nothing was read.
    NotRequested {
        /// Stable refusal code.
        code: &'static str,
        /// The refusal in one sentence.
        message: String,
    },
    /// The rule started and stopped under this request's own budget before it finished. The items
    /// it already published stay, its coverage is `Partial`, and the range it read is named there.
    Skipped {
        /// Why it stopped, in the engine's own stop vocabulary.
        reason: TerminationReason,
    },
}

/// Whether this request performed any of the rules it enabled.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum PluginAnalysis {
    /// At least one enabled rule ran over the scope (a rule that started and stopped under the
    /// budget ran). The per-rule reports say what each one did.
    Performed,
    /// No enabled rule ran, and why: the first per-rule statement of the request in request order.
    /// A request nothing could be performed for can therefore never be read as a scan that found
    /// nothing.
    NotPerformed {
        /// The refusal code of the first statement of this request.
        code: &'static str,
    },
}

/// What one rule declared.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PluginValue {
    /// One provider a `META-INF/services/<service>` entry declares, under the service interface
    /// the entry name carries. Both names are the raw bytes the archive holds: neither is decoded,
    /// resolved, loaded or initialized, and a name this snapshot does not hold is still published
    /// as declared.
    ServiceProvider {
        service: JvmBytes,
        provider: JvmBytes,
    },
}

/// One derived fact: the rule that produced it, the input it was read from, the range that matched,
/// and what it declared.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginItem {
    /// The registered rule that answered, by id.
    pub rule: &'static str,
    /// The rule version.
    pub rule_version: PluginRuleVersion,
    /// The entry the declaration was read from.
    pub source_entry: PhysicalEntryId,
    /// The byte range inside that entry's decompressed content that holds the declaration.
    pub matched: ByteSpan,
    /// What the rule declared.
    pub value: PluginValue,
}

/// What one enabled rule produced.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginRuleReport {
    /// The rule identity the request enabled, exactly as the request spelled it.
    pub requested: PluginSelection,
    /// The registered descriptor that answered: present exactly when this registry holds the
    /// enabled id at the enabled version, and `None` is what makes an `Unsupported` state a
    /// registry answer rather than an omission.
    pub rule: Option<&'static PluginRule>,
    pub analysis: PluginRuleAnalysis,
    /// The declarations this rule published, in read order: entry enumeration order, then the
    /// order of the declarations inside the entry.
    pub items: Vec<PluginItem>,
    /// This rule's own coverage. A rule that was not performed answers
    /// [`Coverage::not_requested`], which claims no range at all.
    pub coverage: Coverage,
    /// Whether an item limit, the snapshot's own listing or this rule's own stop ended it before
    /// the end of its range: this rule's answer is a prefix.
    pub has_more: bool,
    /// How many items this rule published.
    pub returned_items: u64,
}

/// What one plugin request produced.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginReport {
    pub physical: PhysicalView,
    pub analysis: PluginAnalysis,
    /// One report per enabled rule, in request order.
    pub rules: Vec<PluginRuleReport>,
    pub execution: ExecutionReport,
    /// The listing's own diagnostics (the enumeration this request shared), one diagnostic per
    /// refusal, and one per stop. Uncharged control metadata, exactly like the other planes'.
    pub diagnostics: Vec<Diagnostic>,
}

impl PluginReport {
    /// The declarations of every rule this request performed, in report order.
    pub fn items(&self) -> impl Iterator<Item = &PluginItem> {
        self.rules.iter().flat_map(|rule| rule.items.iter())
    }

    /// The rules this request performed, in report order.
    pub fn performed(&self) -> impl Iterator<Item = &PluginRuleReport> {
        self.rules.iter().filter(|rule| {
            matches!(
                rule.analysis,
                PluginRuleAnalysis::Performed | PluginRuleAnalysis::Skipped { .. }
            )
        })
    }
}

/// Request-level checks of one plugin request: shape only, no artifact access.
///
/// Stable codes: `plugin_snapshot_mismatch`, `plugin_no_rules` (invalid input) and
/// `plugin_artifact_tree_root_mismatch` — the only root a fresh snapshot establishes is its own
/// root container, so a scope that names another one cannot describe this snapshot and is refused
/// instead of being silently ignored.
pub(crate) fn validate_request(snapshot: &ArtifactSnapshot, request: &PluginRequest) -> Result<()> {
    if &request.physical.snapshot != snapshot.id() {
        return Err(Error::invalid_input(
            "plugin_snapshot_mismatch",
            "PluginRequest physical view snapshot does not match the artifact snapshot",
        ));
    }
    if request.rules.is_empty() {
        return Err(Error::invalid_input(
            "plugin_no_rules",
            "PluginRequest enables no rule; this plane answers the rule a caller names by id and \
             version, and an empty request is not a scan",
        ));
    }
    if let PhysicalScope::ArtifactTree { root_container } = &request.physical.scope
        && root_container.0 != "root"
    {
        return Err(Error::invalid_input(
            "plugin_artifact_tree_root_mismatch",
            "PluginRequest tree root does not match the snapshot root container",
        ));
    }
    Ok(())
}

/// Runs one plugin request: every enabled rule, over the entries of the snapshot's own scope.
///
/// The snapshot's entries are listed once and shared by the rules of the request, and only an
/// entry whose raw name matches the rule's config path is ever read. A budget refusal or a
/// cancellation is a stop: the scan keeps its prefix, names the range it read, and reports the
/// dimension that refused; every other error is a failure of the input and is returned to the
/// caller rather than silently shortening the answer.
pub fn execute(
    snapshot: &ArtifactSnapshot,
    request: &PluginRequest,
    budget: &mut Budget,
) -> Result<PluginReport> {
    validate_request(snapshot, request)?;
    // A request this plane is not a subject of at all is decided **before** a single entry is
    // listed, so it performs no read and charges nothing for one.
    let decline = decline_of(snapshot, request);
    let listed = match decline {
        Some(_) => None,
        None => Some(snapshot.enumerate(budget)?),
    };
    let mut diagnostics = match &listed {
        Some(enumerated) => enumerated.diagnostics.clone(),
        None => Vec::new(),
    };
    let mut rules: Vec<PluginRuleReport> = Vec::with_capacity(request.rules.len());
    // Two different stops, and they are not the same statement: the **listing's** own stop leaves
    // every rule a prefix to read (its coverage and `has_more` say so, and the listing's own
    // diagnostic is published), while a **rule's** stop ends the request's scan.
    let listing_stop = listed
        .as_ref()
        .and_then(|enumerated| stop_reason(&enumerated.execution));
    let mut stop: Option<TerminationReason> = None;
    let mut cancelled = matches!(
        listed.as_ref().map(|enumerated| &enumerated.execution),
        Some(ExecutionReport::Cancelled { .. })
    );
    let listing_failed = matches!(
        listed.as_ref().map(|enumerated| &enumerated.execution),
        Some(ExecutionReport::Failed { .. })
    );
    for selection in &request.rules {
        if let Some(reason) = &stop {
            // A stop ends the request's scan, exactly as it does in every other plane: a rule that
            // never started makes no claim of its own, and saying `Skipped` here would read as a
            // rule that did.
            let message = format!(
                "the request stopped under {} before this rule ran",
                describe_reason(reason)
            );
            rules.push(refused(
                selection,
                plugin_for(&selection.id, &selection.version),
                PluginRuleAnalysis::NotRequested {
                    code: "plugin_request_stopped",
                    message,
                },
            ));
            continue;
        }
        let input = match &listed {
            Some(enumerated) => RuleInput::Entries(enumerated),
            // `listed` is `None` exactly when this request has a decline, and the fallback is the
            // conservative statement for an input this plane never listed.
            None => RuleInput::Absent(decline.unwrap_or(Decline::INPUT_ABSENT)),
        };
        let outcome = run_rule(
            snapshot,
            request,
            selection,
            input,
            budget,
            &mut diagnostics,
        )?;
        if let Some(error) = &outcome.stop {
            if matches!(error, Error::Cancelled { .. }) {
                cancelled = true;
            }
            stop = Some(reason_of(error));
        }
        rules.push(outcome.report);
    }
    let analysis = if rules.iter().any(|rule| {
        matches!(
            rule.analysis,
            PluginRuleAnalysis::Performed | PluginRuleAnalysis::Skipped { .. }
        )
    }) {
        PluginAnalysis::Performed
    } else {
        PluginAnalysis::NotPerformed {
            code: first_statement(&rules),
        }
    };
    let (stop, failed) = match (stop, listing_stop) {
        (Some(reason), _) => (Some(reason), false),
        (None, Some(reason)) => (Some(reason), listing_failed),
        (None, None) => (None, false),
    };
    let usage = budget.usage();
    let execution = if cancelled {
        ExecutionReport::Cancelled { usage }
    } else if let Some(reason) = stop {
        if failed {
            ExecutionReport::Failed { reason, usage }
        } else {
            ExecutionReport::Partial { reason, usage }
        }
    } else {
        ExecutionReport::Complete { usage }
    };
    Ok(PluginReport {
        physical: request.physical.clone(),
        analysis,
        rules,
        execution,
        diagnostics,
    })
}

/// Why this request is not a subject of the rules it enables: a scope this rule is not registered
/// over, or a snapshot that holds no archive entry at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Decline {
    code: &'static str,
    message: &'static str,
}

impl Decline {
    /// The statement for a request whose input was never listed at all: nothing was read.
    const INPUT_ABSENT: Self = Self {
        code: "plugin_input_absent",
        message: "this request holds no archive entry, and an entry whose raw name matches the \
                  config path is the only input this rule reads",
    };
}

/// The decline of one request, or `None` when the request is a subject of its rules.
fn decline_of(snapshot: &ArtifactSnapshot, request: &PluginRequest) -> Option<Decline> {
    if let PhysicalScope::ArtifactTree { .. } = request.physical.scope {
        return Some(Decline {
            code: "plugin_scope_not_registered",
            message: "this rule reads the snapshot's own root container and is not registered \
                      over a nested tree scope",
        });
    }
    if snapshot.kind() != ArtifactKind::Zip {
        return Some(Decline {
            code: "plugin_input_absent",
            message: "a standalone class file holds no archive entry, and an entry whose raw name \
                      matches the config path is the only input this rule reads",
        });
    }
    None
}

/// What one request gives one rule to read.
enum RuleInput<'a> {
    /// The entries of the snapshot's own root container, listed once for the whole request.
    Entries(&'a EnumerationReport),
    /// This request is not a subject of the rule, and why: nothing was read, nothing charged, and
    /// no coverage claimed.
    Absent(Decline),
}

/// What one rule's run produced, and the stop that ended it when one did.
struct RuleOutcome {
    report: PluginRuleReport,
    stop: Option<Error>,
}

fn run_rule(
    snapshot: &ArtifactSnapshot,
    request: &PluginRequest,
    selection: &PluginSelection,
    input: RuleInput<'_>,
    budget: &mut Budget,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<RuleOutcome> {
    let Some(rule) = plugin_for(&selection.id, &selection.version) else {
        let (code, message) = if let Some(registered) = PLUGINS
            .iter()
            .find(|candidate| candidate.id == selection.id)
            .map(|candidate| candidate.rule.version())
        {
            (
                "plugin_rule_version_not_registered",
                format!(
                    "the configuration \"{}\" is registered at version {registered} and not at \
                     version {}: the rule version is part of the identity a result is read under",
                    selection.id, selection.version
                ),
            )
        } else {
            (
                "plugin_rule_not_registered",
                format!(
                    "no configuration named \"{}\" is registered: this engine reads no such \
                     format, and an empty item list would be the claim that it declared nothing",
                    selection.id
                ),
            )
        };
        diagnostics.push(refusal(code, DiagnosticSeverity::Warning, message.clone()));
        return Ok(RuleOutcome {
            report: refused(
                selection,
                None,
                PluginRuleAnalysis::Unsupported { code, message },
            ),
            stop: None,
        });
    };
    if let PluginSupport::Unsupported { reason } = rule.support {
        diagnostics.push(refusal(
            "plugin_rule_unsupported",
            DiagnosticSeverity::Info,
            reason.to_string(),
        ));
        return Ok(RuleOutcome {
            report: refused(
                selection,
                Some(rule),
                PluginRuleAnalysis::Unsupported {
                    code: "plugin_rule_unsupported",
                    message: reason.to_string(),
                },
            ),
            stop: None,
        });
    }
    let enumerated = match input {
        RuleInput::Entries(enumerated) => enumerated,
        RuleInput::Absent(decline) => {
            return Ok(decline_report(selection, rule, diagnostics, decline));
        }
    };
    scan(
        rule,
        selection,
        enumerated,
        request,
        snapshot,
        budget,
        diagnostics,
    )
}

/// The state of a rule this request is not a subject of: nothing was read, so no range is claimed.
fn decline_report(
    selection: &PluginSelection,
    rule: &'static PluginRule,
    diagnostics: &mut Vec<Diagnostic>,
    decline: Decline,
) -> RuleOutcome {
    diagnostics.push(refusal(
        decline.code,
        DiagnosticSeverity::Warning,
        decline.message.to_string(),
    ));
    RuleOutcome {
        report: refused(
            selection,
            Some(rule),
            PluginRuleAnalysis::NotRequested {
                code: decline.code,
                message: decline.message.to_string(),
            },
        ),
        stop: None,
    }
}

fn scan(
    rule: &'static PluginRule,
    selection: &PluginSelection,
    enumerated: &EnumerationReport,
    request: &PluginRequest,
    snapshot: &ArtifactSnapshot,
    budget: &mut Budget,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<RuleOutcome> {
    let recognized: Vec<&PhysicalEntry> = enumerated
        .entries
        .iter()
        .filter(|entry| registration_key(rule, &entry.id.raw_name.0).is_some())
        .collect();
    let mut items: Vec<PluginItem> = Vec::new();
    let mut scanned: Vec<CoverageRange> = Vec::new();
    let mut unread: Vec<&PhysicalEntry> = Vec::new();
    let mut stop: Option<Error> = None;
    let mut returned: u64 = 0;
    let mut has_more = false;
    for entry in &recognized {
        if stop.is_some() || has_more {
            unread.push(entry);
            continue;
        }
        let Some(key) = registration_key(rule, &entry.id.raw_name.0) else {
            continue;
        };
        let materialized = match snapshot.read_entry_for_analysis(entry, budget) {
            Ok(materialized) => materialized,
            Err(error) if is_stop(&error) => {
                diagnostics.push(stop_diagnostic(&error, Some(entry)));
                stop = Some(error);
                unread.push(entry);
                continue;
            }
            Err(error) => return Err(error),
        };
        scanned.push(CoverageRange {
            label: entry_label(rule.id, entry),
            start: 0,
            end: to_u64(materialized.bytes.len())?,
        });
        let providers = match service_providers(&materialized.bytes, budget) {
            Ok(providers) => providers,
            Err(error) if is_stop(&error) => {
                diagnostics.push(stop_diagnostic(&error, Some(entry)));
                stop = Some(error);
                continue;
            }
            Err(error) => return Err(error),
        };
        for (span, provider) in providers {
            if item_limit_reached(request.max_items, returned) {
                has_more = true;
                break;
            }
            if let Err(error) = budget.charge(rule.budget.dimension, 1) {
                if is_stop(&error) {
                    diagnostics.push(stop_diagnostic(&error, Some(entry)));
                    stop = Some(error);
                } else {
                    return Err(error);
                }
                break;
            }
            items.push(PluginItem {
                rule: rule.id,
                rule_version: rule.rule,
                source_entry: entry.id.clone(),
                matched: span,
                value: PluginValue::ServiceProvider {
                    service: JvmBytes(key.to_vec()),
                    provider: JvmBytes(provider),
                },
            });
            returned += 1;
        }
    }
    let complete = stop.is_none()
        && !has_more
        && matches!(enumerated.execution, ExecutionReport::Complete { .. });
    let listed_completely = matches!(enumerated.execution, ExecutionReport::Complete { .. });
    let skipped = unread
        .into_iter()
        .map(|entry| unread_range(rule.id, entry))
        .collect();
    let analysis = match &stop {
        Some(error) => PluginRuleAnalysis::Skipped {
            reason: reason_of(error),
        },
        None => PluginRuleAnalysis::Performed,
    };
    Ok(RuleOutcome {
        report: PluginRuleReport {
            requested: selection.clone(),
            rule: Some(rule),
            analysis,
            items,
            coverage: coverage(complete, scanned, skipped),
            // A rule's answer is a prefix when an item limit, the listing or its own stop ended
            // its range before the end of it.
            has_more: has_more || stop.is_some() || !listed_completely,
            returned_items: returned,
        },
        stop,
    })
}

/// One rule this request never started: no items, and no coverage claim.
fn refused(
    selection: &PluginSelection,
    rule: Option<&'static PluginRule>,
    analysis: PluginRuleAnalysis,
) -> PluginRuleReport {
    PluginRuleReport {
        requested: selection.clone(),
        rule,
        analysis,
        items: Vec::new(),
        coverage: Coverage::not_requested(),
        has_more: false,
        returned_items: 0,
    }
}

/// The coverage of one performed rule: its own range, and `NotRequested` for the two dimensions
/// this plane never touches — nothing here resolves a runtime definition and nothing here
/// analyses dynamic behaviour.
fn coverage(complete: bool, scanned: Vec<CoverageRange>, skipped: Vec<CoverageRange>) -> Coverage {
    Coverage {
        artifact_structural: CoverageDimension {
            state: if complete {
                CoverageState::CompleteWithinSchema
            } else {
                CoverageState::Partial
            },
            scanned,
            skipped,
            uninterpreted_extensions: Vec::new(),
        },
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

/// Whether one entry name is an input of this rule, and the registration key it carries.
///
/// The path rule is the resource consumer's and is reused rather than restated: `xref::resource`
/// owns what a `META-INF/services` entry is for this engine, so this plane and the structural scan
/// agree byte for byte on which entries are configurations and what the key inside the name is. A
/// rule whose declared prefix is *not* the one the shared matcher recognizes reads no entry at all:
/// the table cannot silently borrow another configuration's syntax.
pub(crate) fn registration_key<'a>(rule: &PluginRule, raw_name: &'a [u8]) -> Option<&'a [u8]> {
    if rule.config_path.prefix.as_bytes() != SERVICES_PREFIX {
        return None;
    }
    service_key(raw_name)
}

/// Whether one error is this request's own stop rather than a failure of the input.
///
/// A budget refusal and a cancellation are what a bounded request ends under; every other error
/// (damaged bytes, an overflow) is a failure the caller is told about instead of a range this plane
/// silently shortened.
fn is_stop(error: &Error) -> bool {
    matches!(
        error,
        Error::BudgetExceeded { .. } | Error::Cancelled { .. }
    )
}

/// The stop vocabulary of one error, in the engine's own terms.
fn reason_of(error: &Error) -> TerminationReason {
    match error {
        Error::BudgetExceeded { dimension, .. } => TerminationReason::BudgetExceeded {
            dimension: *dimension,
        },
        Error::Unsupported { code, .. } => TerminationReason::Unsupported { code: code.clone() },
        Error::Cancelled { reason } => TerminationReason::Error {
            code: format!("cancelled: {reason}"),
        },
        Error::InvalidInput { code, .. } => TerminationReason::Error { code: code.clone() },
        Error::Io { operation, .. } => TerminationReason::Error {
            code: operation.clone(),
        },
    }
}

/// The reason one report stopped under, when it stopped.
fn stop_reason(execution: &ExecutionReport) -> Option<TerminationReason> {
    match execution {
        ExecutionReport::Complete { .. } => None,
        ExecutionReport::Cancelled { .. } => Some(TerminationReason::Error {
            code: "cancelled".into(),
        }),
        ExecutionReport::Partial { reason, .. } | ExecutionReport::Failed { reason, .. } => {
            Some(reason.clone())
        }
    }
}

fn describe_reason(reason: &TerminationReason) -> String {
    match reason {
        TerminationReason::BudgetExceeded { dimension } => {
            format!("the {} budget", budget_dimension_code(*dimension))
        }
        TerminationReason::Error { code } => code.clone(),
        TerminationReason::Unsupported { code } => code.clone(),
    }
}

/// The first per-rule statement of one request, in request order.
fn first_statement(rules: &[PluginRuleReport]) -> &'static str {
    rules
        .iter()
        .find_map(|rule| match &rule.analysis {
            PluginRuleAnalysis::Unsupported { code, .. }
            | PluginRuleAnalysis::NotRequested { code, .. } => Some(*code),
            PluginRuleAnalysis::Performed | PluginRuleAnalysis::Skipped { .. } => None,
        })
        .unwrap_or("plugin_request_not_performed")
}

/// Whether the request's own item limit has been reached (`0` means no limit).
fn item_limit_reached(max_items: u64, returned: u64) -> bool {
    max_items != 0 && returned >= max_items
}

/// The label of one entry in a rule's coverage: the rule's own name and the entry's raw name, so
/// two rules never share a range and a reader can find the entry again.
fn entry_label(rule: &str, entry: &PhysicalEntry) -> String {
    format!(
        "plugin:{rule}:entry:{}",
        escaped_raw_name(&entry.id.raw_name.0)
    )
}

/// The range one read entry covers: its whole decompressed content.
fn unread_range(rule: &str, entry: &PhysicalEntry) -> CoverageRange {
    CoverageRange {
        label: entry_label(rule, entry),
        start: 0,
        end: entry.uncompressed_size,
    }
}

/// The diagnostic of one refusal or decline. It carries no provenance: no input byte backs it.
fn refusal(code: &'static str, severity: DiagnosticSeverity, message: String) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity,
        message,
        provenance: None,
    }
}

/// The diagnostic of one stop: the dimension that refused (or the cancellation) and, when the stop
/// happened while one entry was being read, that entry's own provenance.
fn stop_diagnostic(error: &Error, entry: Option<&PhysicalEntry>) -> Diagnostic {
    Diagnostic {
        code: match error {
            Error::BudgetExceeded { dimension, .. } => {
                format!("budget_exceeded_{}", budget_dimension_code(*dimension))
            }
            Error::Cancelled { .. } => "cancelled".into(),
            Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
            Error::Io { operation, .. } => operation.clone(),
        },
        severity: DiagnosticSeverity::Warning,
        message: error.to_string(),
        provenance: entry.map(|entry| Provenance {
            location: Location::Entry {
                id: entry.id.clone(),
                span: ByteSpan::new(0, entry.uncompressed_size),
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarde_reader::artifact::ArtifactInput;

    fn limits() -> jarde_reader::budget::Limits {
        jarde_reader::budget::Limits {
            input_bytes: 1 << 22,
            archive_entries: 64,
            entry_bytes: 1 << 22,
            read_bytes: 1 << 22,
            class_bytes: 1 << 22,
            attribute_bytes: 1 << 22,
            output_bytes: 1 << 22,
            code_bytes: 1 << 22,
            result_items: 64,
            class_headers: 64,
            method_bodies: 64,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            normalization_clones: 1 << 10,
            nested_depth: 4,
            dependency_depth: 8,
            analysis_steps: 1 << 20,
            elapsed_millis: u64::MAX,
        }
    }

    fn selection(id: &str, version: &str) -> PluginSelection {
        PluginSelection {
            id: id.to_string(),
            version: version.to_string(),
        }
    }

    #[test]
    fn every_registered_rule_states_a_source_a_boundary_and_an_unclaimed_list() {
        for rule in plugins() {
            assert!(!rule.source.is_empty(), "{}", rule.id);
            assert_eq!(rule.isolation, PLUGIN_TRUST_DOMAIN, "{}", rule.id);
            assert!(!rule.not_claimed.is_empty(), "{}", rule.id);
            assert_eq!(rule.rule.name(), rule.id, "{}", rule.id);
            assert!(!rule.config_path.prefix.is_empty(), "{}", rule.id);
            assert!(!rule.config_path.declares.is_empty(), "{}", rule.id);
            assert_eq!(plugin_for(rule.id, rule.rule.version()), Some(rule));
        }
        assert!(PLUGIN_TRUST_DOMAIN.contains("not a sandbox"));
        assert!(PLUGIN_TRUST_DOMAIN.contains("separate process"));
        assert!(PLUGIN_TRUST_DOMAIN.contains("Wasm"));
    }

    #[test]
    fn the_index_the_config_path_declares_is_the_one_the_shared_matcher_reads() {
        let services = PLUGINS
            .iter()
            .find(|rule| rule.id == "service-loader-registrations")
            .expect("the fixture rule is registered");
        assert_eq!(services.config_path.prefix.as_bytes(), SERVICES_PREFIX);
        assert_eq!(
            registration_key(services, b"META-INF/services/com.example.Service"),
            Some(&b"com.example.Service"[..])
        );
        assert_eq!(registration_key(services, b"META-INF/MANIFEST.MF"), None);
        // A rule whose declared prefix is not the shared matcher's reads nothing: the table cannot
        // borrow another configuration's syntax by naming its path.
        let mut borrowed = *services;
        borrowed.config_path.prefix = "META-INF/spring.factories/";
        assert_eq!(
            registration_key(&borrowed, b"META-INF/services/com.example.Service"),
            None
        );
    }

    #[test]
    fn a_request_without_a_rule_and_a_request_for_another_snapshot_are_input_errors() {
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(
            ArtifactInput::bytes(vec![0xca, 0xfe, 0xba, 0xbe]),
            &mut budget,
        )
        .expect("a class magic opens as a standalone class snapshot");
        let request = PluginRequest {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            rules: Vec::new(),
            max_items: 0,
        };
        let error = execute(&snapshot, &request, &mut budget).expect_err("no rule is not a scan");
        assert_eq!(
            error,
            Error::invalid_input(
                "plugin_no_rules",
                "PluginRequest enables no rule; this plane answers the rule a caller names by id \
                 and version, and an empty request is not a scan"
            )
        );
        let request = PluginRequest {
            physical: PhysicalView {
                snapshot: jarde_reader::model::SnapshotId("other".into()),
                scope: PhysicalScope::SnapshotAll,
            },
            rules: vec![selection("service-loader-registrations", "1")],
            max_items: 0,
        };
        let error = execute(&snapshot, &request, &mut budget).expect_err("another snapshot");
        assert!(matches!(
            error,
            Error::InvalidInput { ref code, .. } if code == "plugin_snapshot_mismatch"
        ));
        // The only root a fresh snapshot establishes is its own root container.
        let request = PluginRequest {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::ArtifactTree {
                    root_container: jarde_reader::model::ContainerId("other".into()),
                },
            },
            rules: vec![selection("service-loader-registrations", "1")],
            max_items: 0,
        };
        let error = execute(&snapshot, &request, &mut budget).expect_err("another tree root");
        assert!(matches!(
            error,
            Error::InvalidInput { ref code, .. } if code == "plugin_artifact_tree_root_mismatch"
        ));
    }

    #[test]
    fn a_standalone_class_file_has_no_entry_for_a_resource_rule_to_read() {
        let mut budget = Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(
            ArtifactInput::bytes(vec![0xca, 0xfe, 0xba, 0xbe]),
            &mut budget,
        )
        .expect("a class magic opens as a standalone class snapshot");
        let request = PluginRequest {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            rules: vec![selection("service-loader-registrations", "1")],
            max_items: 0,
        };
        let report = execute(&snapshot, &request, &mut budget).expect("the request is answered");
        assert_eq!(
            report.analysis,
            PluginAnalysis::NotPerformed {
                code: "plugin_input_absent"
            }
        );
        let rule = &report.rules[0];
        assert!(matches!(
            rule.analysis,
            PluginRuleAnalysis::NotRequested {
                code: "plugin_input_absent",
                ..
            }
        ));
        assert!(rule.items.is_empty());
        assert_eq!(rule.coverage, Coverage::not_requested());
        assert_eq!(
            rule.rule.map(|rule| rule.id),
            Some("service-loader-registrations")
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "plugin_input_absent")
        );
    }
}
