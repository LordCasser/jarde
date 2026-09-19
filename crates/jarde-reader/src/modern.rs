//! P4 1.2: the modern structural facts of one class file, read under the release-bound registry.
//!
//! P0–P3 read a class file's declaration structure and its constant pool. This pass reads the
//! *modern* structures the P4 change is about — `Record` components, the `PermittedSubclasses`
//! declaration, the deferred constant-dynamic graph and modern string-concat sites — and, before it
//! reads any of them, it asks [`crate::release_registry`] whether that release legally places the
//! attribute it is about to read. The registry states the rules; this pass reads an artifact and
//! reports what it found in the vocabulary the registry defines.
//!
//! # What the registry decides here, and what it does not
//!
//! One read path, one answer per class-level attribute the registry holds a rule for:
//!
//! * `Legal { rule }` — the entry is read, and the fact carries that rule as its **version
//!   evidence** (the `since` release and the JVMS section the rule cites).
//! * `VersionNotApplicable` / `LocationNotApplicable` — the entry is **not** read into a fact and
//!   the registry's diagnostic for that placement is reported
//!   (`classfile_attribute_version_not_applicable`, `classfile_attribute_location_not_applicable`).
//!   That diagnostic stays the registry's own: it names the attribute and the release rule, it
//!   carries no provenance, and it takes no class name, method name or path from the input. The
//!   entry's own class-file range is reported next to it in [`ModernAttributePlacement`], so the
//!   rule and the evidence for the artifact stay two separate statements.
//! * `NotRegistered` / `UnregisteredRelease` — the registry holds no rule for that name at any
//!   release it covers, or holds no record for this release at all. This pass then claims nothing:
//!   the name is not listed in [`ModernFacts::attributes`], no diagnostic is invented for it, and no
//!   content is read. A fact is published only when a release rule confirms it, which is what the
//!   requirement's "版本规则确认……合法存在" asks for: a fact is never inferred from a class name, a
//!   member name or a path.
//!
//! The header plane is untouched. [`crate::classfile::inspect_header`] states version facts only,
//! and this pass is where an attribute's placement is answered — the boundary P4 1.1 drew and its
//! `header_inspection_leaves_attribute_and_flag_legality_to_the_fact_passes` test pins.
//!
//! # The facts this pass owns
//!
//! | fact | type | origin |
//! | --- | --- | --- |
//! | `Record` components (JVMS 4.7.30) | [`RecordFacts`] | the entry's class-file range, each component's name/descriptor index, and each component attribute's own `attribute_info` range |
//! | `PermittedSubclasses` (JVMS 4.7.31) | [`PermittedSubclassesFacts`] | the entry's class-file range and each permitted name's `CONSTANT_Class` index |
//! | constant-dynamic graph | [`CondyGraph`] | constant-pool indexes and their spans, `BootstrapMethods` indexes, and per-use-site edge paths |
//! | modern string concat | [`ConcatSite`] | the `CONSTANT_InvokeDynamic` index and span, the bootstrap index, and the factory reference the bootstrap handle names |
//!
//! Module, nestmate and `PermittedSubclasses` facts are read by P1's [`crate::classfile::attribute_facts`]
//! as well; this pass reuses that read for `PermittedSubclasses` and reports only the placement of
//! the others, because the P1 owner already publishes their structure and a second reader of the
//! same bytes would be a second answer to the same question.
//!
//! # Ownership of the modern-concat fact
//!
//! A concat site is a **reader** fact. It is read from the constant pool and the `BootstrapMethods`
//! table — the two structures this crate already reads — plus the bootstrap handle's `MethodRef`,
//! and it is read without interpreting a recipe, resolving a handle or calling a factory. The BCI of
//! the `invokedynamic` that consumed the site is the bytecode reader's own fact
//! ([`crate::classfile::InstructionFact::bci`]), and the join key between the two is the site's
//! `constant_pool_index`, so no BCI is invented here and none is lost. That split is why the site's
//! structure belongs to `jarde-reader` while scheduling, dispatch and typed call-site modelling stay
//! above it.
//!
//! The `concat@1` presentation P3 defined — the `StringBuilder` chain a Java 8 output level would
//! need — is deliberately **not** this pass's answer: this pass reports the modern site as a fact
//! and, when the requested output level cannot represent it, reports the conflict and keeps the
//! modern origin ([`ModernFacts::assess_output_level`]). It never decides that an
//! `invokedynamic`-based concat and a `StringBuilder` chain are the same program.
//!
//! # Billing
//!
//! * `AttributeBytes` — one charge per attribute content read, `6 + content length`, exactly like
//!   [`crate::classfile::attribute_facts`] and [`crate::classfile::bootstrap_methods`] bill the
//!   entries they read. The charges are made by those two functions, not repeated here.
//! * `ResultItems` — one per published fact entry: a placement row, a record component, a permitted
//!   name's entry, a concat site, a conflict. (Permitted names are billed by the P1 list reader
//!   that produces them.)
//! * `IrItems`, `IrEdges`, `AnalysisSteps`, `DependencyDepth` — the graph's own dimensions: one per
//!   node, one per edge, one per node visit, and the depth of the path a visit enters. These are the
//!   dimensions P2 uses for derived graphs, and **a refusal on one of them stops the walk instead of
//!   failing the read**: the graph is returned with [`CondyGraph::budget`] naming the dimension and
//!   the limit that stopped it. A truncated graph is never reported as complete; see
//!   [`CondyGraph::is_complete`].
//! * Every other dimension — a cancellation or the elapsed-time limit — fails the read through
//!   `Budget::poll`, because those refusals are the request's to absorb, not this pass's.
//!
//! # Codes this pass adds
//!
//! | code | severity | decided by |
//! | --- | --- | --- |
//! | `classfile_condy_bootstrap_out_of_range` | warning | a dynamic entry names a `BootstrapMethods` entry the attribute does not declare |
//! | `classfile_concat_handle_unresolved` | info | an `invokedynamic` site's bootstrap handle does not name a `CONSTANT_MethodRef`, so no factory can be named for it |
//!
//! Both are structural findings about the bytes this pass read, and both keep the registry's rule
//! for a *legal* placement out of them: a malformed reference is not a release-rule violation. Like
//! the registry's diagnostics they carry no provenance, because a reader fact pass over a byte slice
//! has no snapshot identity to name; the class-file positions travel in the facts instead.

use crate::budget::{Budget, BudgetDimension, CountedBudgetDimension};
use crate::classfile::{
    AttributeReader, AttributeShell, BootstrapMethodFacts, ClassFacts, CpEntryFacts, CpEntryKind,
    ModernFeature, ModernOrigin, NestedAttributeFact, OutputLevel, OutputLevelConflict,
    OutputLevelStatus, attribute_content, bootstrap_methods, charge_item, cp_entry, cp_utf8,
    ensure_unique, entry_offset, read_class_name_list, read_u16,
};
use crate::error::{Error, Result};
use crate::model::{ByteSpan, Diagnostic, DiagnosticSeverity, JvmBytes};
use crate::release_registry::{
    AttributePlacement, AttributeRule, ClassfileLocation, ConstantPoolTagRule,
    ConstantPoolTagStatus, feature_registry,
};
use std::collections::{BTreeMap, BTreeSet};

/// `CONSTANT_Dynamic` (JVMS 4.4.10).
const TAG_DYNAMIC: u8 = 17;
/// `CONSTANT_InvokeDynamic` (JVMS 4.4.10).
const TAG_INVOKE_DYNAMIC: u8 = 18;
/// The factory owner a modern string-concat site names.
const CONCAT_FACTORY: &[u8] = b"java/lang/invoke/StringConcatFactory";

// ---------------------------------------------------------------------------
// Record and sealed declarations
// ---------------------------------------------------------------------------

/// One `Record` component (JVMS 4.7.30).
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordComponentFacts {
    /// Zero-based position in the attribute's component list.
    pub index: u16,
    /// `name_index` exactly as the declaration records it.
    pub name_index: u16,
    /// Raw name bytes of that index; a component name is never text-decoded here.
    pub name: JvmBytes,
    pub descriptor_index: u16,
    /// Raw descriptor bytes of that index.
    pub descriptor: JvmBytes,
    /// The component's own `attributes` table as `attribute_info` entries: each one keeps its
    /// `attribute_name_index`, the range of the whole entry and the range of its content, so a
    /// caller can slice it out of the class bytes. This pass interprets none of them.
    pub attributes: Vec<NestedAttributeFact>,
}

/// A class's `Record` attribute, with the release rule that placed it.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordFacts {
    /// `attribute_name_index` of the entry, read back from the shell's own span.
    pub name_index: u16,
    /// Class-file range of the whole `attribute_info`.
    pub span: ByteSpan,
    /// Class-file range of the entry's content.
    pub content_span: ByteSpan,
    /// Components in declaration order.
    pub components: Vec<RecordComponentFacts>,
    /// The registry rule this entry was read under: the version evidence for the fact.
    pub rule: &'static AttributeRule,
}

/// A sealed class's `PermittedSubclasses` attribute, with the release rule that placed it.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermittedSubclassesFacts {
    pub name_index: u16,
    pub span: ByteSpan,
    pub content_span: ByteSpan,
    /// Permitted subclass internal names, expanded from `CONSTANT_Class`, in declaration order.
    pub permitted: Vec<JvmBytes>,
    pub rule: &'static AttributeRule,
}

/// One class-level attribute the registry holds a rule for, as one read found it.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModernAttributePlacement {
    /// Raw name bytes from the entry's `attribute_name_index`.
    pub name: JvmBytes,
    /// That index, read back from the shell's own span.
    pub name_index: u16,
    /// Class-file range of the whole `attribute_info`.
    pub span: ByteSpan,
    /// Class-file range of the entry's content.
    pub content_span: ByteSpan,
    /// The registry's answer for `(name, ClassFile, this release)`.
    pub placement: AttributePlacement,
    /// Whether this pass read the entry's content into a typed fact. True for `Record`,
    /// `PermittedSubclasses` and `BootstrapMethods` when the placement is `Legal`; false for every
    /// other registered name, whose typed fact another reader pass owns.
    pub read: bool,
}

// ---------------------------------------------------------------------------
// The deferred constant-dynamic graph
// ---------------------------------------------------------------------------

/// Identity of one node of the deferred constant-dynamic graph.
///
/// The two index spaces are the two a bootstrap table really uses: constant-pool indexes are 1-based
/// and are what an instruction or an attribute records, and `BootstrapMethods` indexes are 0-based
/// positions in that attribute's table. A node is never addressed by anything else, so a path of
/// nodes can be resolved without re-reading the class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CondyNodeRef {
    ConstantPool { index: u16 },
    Bootstrap { index: u16 },
}

/// What one constant-pool node is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CondyNodeKind {
    /// `CONSTANT_Dynamic` (tag 17): a constant a bootstrap method decides.
    Dynamic,
    /// `CONSTANT_InvokeDynamic` (tag 18): a call site a bootstrap method decides.
    InvokeDynamic,
    /// `CONSTANT_MethodHandle` (tag 15): a bootstrap method, or a static handle argument.
    MethodHandle,
    /// `CONSTANT_MethodType` (tag 16): a descriptor argument.
    MethodType,
    /// Any other loadable constant a bootstrap argument may name (JVMS 4.4).
    StaticArgument,
}

/// One node of the graph.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CondyNode {
    pub node: CondyNodeRef,
    /// The entry's kind, or `None` for a `BootstrapMethods` entry, which is a table position rather
    /// than a constant-pool entry and has no span of its own in the class file.
    pub kind: Option<CondyNodeKind>,
    /// Class-file range of the constant-pool entry; `None` for a `BootstrapMethods` entry.
    pub span: Option<ByteSpan>,
}

/// Which reference one edge of the graph follows.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CondyEdgeKind {
    /// A dynamic entry names the `BootstrapMethods` entry that governs it.
    Bootstrap,
    /// A bootstrap entry names its method handle (`bootstrap_method_ref`).
    BootstrapHandle,
    /// A bootstrap entry names one of its static arguments.
    BootstrapArgument,
}

/// One edge of the graph.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CondyEdge {
    pub kind: CondyEdgeKind,
    pub from: CondyNodeRef,
    pub to: CondyNodeRef,
    /// The argument's position in the entry's `arguments` array, for
    /// [`CondyEdgeKind::BootstrapArgument`]; `None` for the two bootstrap references.
    pub argument_index: Option<u16>,
}

/// One node one use-site reached, with the path that reached it.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CondyReach {
    pub node: CondyNodeRef,
    /// Ordinals into [`CondyGraph::edges`], in the order they were walked from the use-site's own
    /// node. Two use-sites that share a subgraph name the **same** ordinals, because an expansion is
    /// stored once: that is what makes a shared node, a shared edge and a shared path checkable
    /// instead of merely claimed.
    pub via: Vec<u32>,
    /// The earlier reach of the same node in this use-site's list when the walk met the node again.
    /// A repeat is never followed — that is the visited set which makes the walk terminate — and the
    /// repeat's own `via` is the path that reached it a second time.
    pub repeat: Option<u32>,
}

impl CondyReach {
    /// Number of edges walked from the use-site: the use-site's own node is depth 0.
    pub fn depth(&self) -> usize {
        self.via.len()
    }
}

/// One entry point of the graph and everything it reaches.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CondyUseSite {
    /// The dynamic entry that opens the walk.
    pub site: CondyNodeRef,
    /// The nodes the walk reached from it, deepest first, in the order the walk closed them.
    pub reaches: Vec<CondyReach>,
}

/// One repeat that closed a cycle: the walk returned to a node already on the path it was walking.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CondyCycle {
    /// Ordinal into [`CondyGraph::use_sites`].
    pub site: usize,
    /// Ordinal into that use-site's `reaches`: the reach that closed the cycle. Its `via` is the
    /// path that leads back to `node`, so the cycle's edges are the last edge of that path plus the
    /// earlier path to `node`.
    pub reach: u32,
    pub node: CondyNodeRef,
}

/// Why a walk stopped before the graph was exhausted.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CondyStop {
    /// The `IrItems` limit is exhausted.
    Nodes { limit: u64 },
    /// The `IrEdges` limit is exhausted.
    Edges { limit: u64 },
    /// The `AnalysisSteps` limit is exhausted.
    Steps { limit: u64 },
    /// The `DependencyDepth` limit is exhausted; `entering` is the depth the walk was about to
    /// enter and `limit` the limit that refused it.
    Depth { limit: u64, entering: u64 },
}

/// What one graph walk consumed, in the reader's own dimensions.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CondyBudget {
    /// `IrItems`: nodes added.
    pub nodes: u64,
    /// `IrEdges`: edges added.
    pub edges: u64,
    /// `AnalysisSteps`: node visits, repeats included.
    pub steps: u64,
    /// High-water mark of the path depth the walk reached.
    pub max_depth: u64,
    /// The dimension that stopped the walk, or `None` when it ran to the end of every use-site.
    pub stopped: Option<CondyStop>,
}

/// The deferred constant-dynamic graph of one class, as structural facts.
///
/// No bootstrap method is executed, no handle is resolved, no descriptor is linked and no recipe is
/// interpreted: the graph is what the constant pool and the `BootstrapMethods` table say about each
/// other. `visited` is the set of nodes already reached, which is why a shared subgraph is expanded
/// once and why a cycle is recorded rather than followed.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CondyGraph {
    /// Nodes in first-visit order. Every node an edge or a reach names is here.
    pub nodes: Vec<CondyNode>,
    /// Edges in discovery order; ordinals are what a reach's `via` names.
    pub edges: Vec<CondyEdge>,
    /// Entry points in constant-pool order.
    pub use_sites: Vec<CondyUseSite>,
    /// Cycles the walk closed, in the order it closed them.
    pub cycles: Vec<CondyCycle>,
    /// How many entries the `BootstrapMethods` attribute declares (0 when it was not read).
    pub bootstrap_entries: u16,
    /// The registry's answer for the `CONSTANT_Dynamic` tag at this class's release: the version
    /// evidence for every node kind here.
    pub dynamic_tag: ConstantPoolTagStatus,
    pub budget: CondyBudget,
}

impl CondyGraph {
    /// Whether the walk reached the end of every use-site within its budget. A graph that is not
    /// complete names the dimension that stopped it in [`CondyGraph::budget`], so "no more facts"
    /// and "no more budget" are never the same answer.
    pub fn is_complete(&self) -> bool {
        self.budget.stopped.is_none()
    }

    /// The node with this identity, when the walk registered it.
    pub fn node(&self, node: CondyNodeRef) -> Option<&CondyNode> {
        self.nodes.iter().find(|entry| entry.node == node)
    }
}

// ---------------------------------------------------------------------------
// Modern string concat
// ---------------------------------------------------------------------------

/// Which `StringConcatFactory` entry a site calls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConcatStrategy {
    /// `makeConcat`: the factory builds the result from the dynamic arguments alone.
    Concat,
    /// `makeConcatWithConstants`: the factory also reads a recipe from the site's static arguments.
    ConcatWithConstants,
    /// Another entry of the factory, reported under the name the site really declares.
    Other,
}

/// One `invokedynamic` site whose bootstrap method handle is
/// `java.lang.invoke.StringConcatFactory`. (JVMS 4.7.23 does not name the factory; the site names
/// it, in its own bootstrap handle, and this pass reports that reference rather than assuming it.)
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcatSite {
    /// The `CONSTANT_InvokeDynamic` entry, 1-based: the join key from an instruction's own
    /// `constant_pool_index`, which is where the site's BCI lives.
    pub constant_pool_index: u16,
    pub span: ByteSpan,
    pub name: JvmBytes,
    pub descriptor: JvmBytes,
    pub bootstrap_index: u16,
    pub strategy: ConcatStrategy,
    /// The internal name of the class the bootstrap handle's `MethodRef` names.
    pub factory_owner: JvmBytes,
    pub factory_name: JvmBytes,
    pub factory_descriptor: JvmBytes,
    /// The recipe of a `makeConcatWithConstants` site: the bytes of its first static argument when
    /// that argument is a `CONSTANT_String`. Read as bytes, never expanded or evaluated.
    pub recipe: Option<JvmBytes>,
    /// The registry's rule for the `CONSTANT_InvokeDynamic` tag at this class's release: the version
    /// evidence for the site.
    pub tag_rule: &'static ConstantPoolTagRule,
}

// ---------------------------------------------------------------------------
// The pass
// ---------------------------------------------------------------------------

/// Every modern structural fact of one class, plus the output-level answer over them.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModernFacts {
    pub major_version: u16,
    /// One row per class-level attribute the registry holds a rule for, in declaration order.
    pub attributes: Vec<ModernAttributePlacement>,
    /// `Record` components, present only when the registry placed the entry legally.
    pub record: Option<RecordFacts>,
    /// `PermittedSubclasses`, present only when the registry placed the entry legally.
    pub permitted_subclasses: Option<PermittedSubclassesFacts>,
    /// Modern string-concat sites, in constant-pool order.
    pub concat: Vec<ConcatSite>,
    pub condy: CondyGraph,
    /// The answer for the requested output level. Never `NotEvaluated`: this pass is the one that
    /// evaluates it, and `NotEvaluated` remains the header plane's honest state.
    pub output_level: OutputLevelStatus,
    /// The registry's placement diagnostics, then this pass's own structural ones, in that order.
    pub diagnostics: Vec<Diagnostic>,
}

impl ModernFacts {
    /// The output-level answer over these facts, for any level.
    ///
    /// A level that cannot represent one of the constructs design decision 5 names gets a
    /// [`OutputLevelStatus::Conflict`] naming each conflicting fact's origin and the release rule
    /// that requires it. The facts themselves are untouched: the conflict is the fallback answer,
    /// and no modern construct is rewritten into an equivalent-looking representation of another
    /// level.
    pub fn assess_output_level(&self, level: OutputLevel) -> OutputLevelStatus {
        let mut conflicts = Vec::new();
        if let Some(record) = &self.record {
            conflicts.push(OutputLevelConflict {
                feature: ModernFeature::RecordComponents,
                level,
                since: record.rule.since,
                source: record.rule.source.to_owned(),
                origin: ModernOrigin::ClassAttribute {
                    name: JvmBytes(b"Record".to_vec()),
                    name_index: record.name_index,
                    span: record.span.clone(),
                    content_span: record.content_span.clone(),
                },
            });
        }
        if let Some(sealed) = &self.permitted_subclasses {
            conflicts.push(OutputLevelConflict {
                feature: ModernFeature::PermittedSubclasses,
                level,
                since: sealed.rule.since,
                source: sealed.rule.source.to_owned(),
                origin: ModernOrigin::ClassAttribute {
                    name: JvmBytes(b"PermittedSubclasses".to_vec()),
                    name_index: sealed.name_index,
                    span: sealed.span.clone(),
                    content_span: sealed.content_span.clone(),
                },
            });
        }
        for site in &self.concat {
            conflicts.push(OutputLevelConflict {
                feature: ModernFeature::StringConcat,
                level,
                since: site.tag_rule.since,
                source: site.tag_rule.source.to_owned(),
                origin: ModernOrigin::ConstantPoolEntry {
                    index: site.constant_pool_index,
                    span: site.span.clone(),
                },
            });
        }
        if conflicts.is_empty() {
            OutputLevelStatus::Representable { level }
        } else {
            OutputLevelStatus::Conflict { level, conflicts }
        }
    }

    /// The graph's own completeness, for callers that only ask about the deferred facts.
    pub fn is_complete(&self) -> bool {
        self.condy.is_complete()
    }
}

/// Reads the modern structural facts of one class under `level`.
///
/// `facts` is the [`ClassFacts`] read of the same bytes: this pass never re-decodes the constant
/// pool or the class-level shell list, it reads attribute content the shell list names. `level` is
/// the output level the caller asks about; the facts are the same for every level, and only
/// [`ModernFacts::output_level`] answers the level's question.
///
/// # Errors
///
/// The same structural errors the attribute readers raise — `classfile_duplicate_attribute` for a
/// repeated single-valued entry, `classfile_invalid_attribute_content` for content that ends before
/// its declared structure or has trailing bytes, and the constant-pool errors
/// [`crate::classfile::cp_utf8`] and [`crate::classfile::cp_class_name`] raise. A budget refusal on
/// the graph's own dimensions is not an error: it stops the walk and is reported in
/// [`CondyGraph::budget`].
#[allow(dead_code)]
pub fn modern_facts(
    bytes: &[u8],
    facts: &ClassFacts,
    level: OutputLevel,
    budget: &mut Budget,
) -> Result<ModernFacts> {
    budget.poll()?;
    let registry = feature_registry();
    let major = facts.major_version;
    let mut attributes = Vec::new();
    let mut diagnostics = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    let mut record = None;
    let mut permitted_subclasses = None;
    let mut bootstrap_shell = None;

    for shell in &facts.attributes {
        budget.poll()?;
        let raw = shell.name.raw().0.clone();
        // The registry holds JVMS attribute names, which are ASCII. A name that is not valid UTF-8
        // cannot be one of them, so the registry makes no claim about it and neither does this pass.
        let Ok(name) = std::str::from_utf8(&raw) else {
            continue;
        };
        let placement = registry.attribute_placement(name, ClassfileLocation::ClassFile, major);
        let rule = match &placement {
            AttributePlacement::Legal { rule } => Some(*rule),
            AttributePlacement::VersionNotApplicable { .. }
            | AttributePlacement::LocationNotApplicable { .. } => {
                diagnostics.extend(registry.attribute_diagnostic(
                    name,
                    ClassfileLocation::ClassFile,
                    major,
                ));
                None
            }
            AttributePlacement::NotRegistered | AttributePlacement::UnregisteredRelease => continue,
        };
        let name_index = shell_name_index(bytes, shell)?;
        let mut read = false;
        if let Some(rule) = rule {
            match name {
                "Record" => {
                    ensure_unique(&mut seen, "Record")?;
                    record = Some(read_record(
                        bytes,
                        shell,
                        &facts.constant_pool,
                        rule,
                        name_index,
                        budget,
                    )?);
                    read = true;
                }
                "PermittedSubclasses" => {
                    ensure_unique(&mut seen, "PermittedSubclasses")?;
                    let permitted = read_class_name_list(
                        &mut AttributeReader::new(attribute_content(bytes, shell, budget)?),
                        &facts.constant_pool,
                        budget,
                    )?;
                    permitted_subclasses = Some(PermittedSubclassesFacts {
                        name_index,
                        span: shell.span.clone(),
                        content_span: shell.content_span.clone(),
                        permitted,
                        rule,
                    });
                    read = true;
                }
                "BootstrapMethods" => {
                    ensure_unique(&mut seen, "BootstrapMethods")?;
                    bootstrap_shell = Some(shell);
                    read = true;
                }
                _ => {}
            }
        }
        charge_item(budget)?;
        attributes.push(ModernAttributePlacement {
            name: JvmBytes(raw),
            name_index,
            span: shell.span.clone(),
            content_span: shell.content_span.clone(),
            placement,
            read,
        });
    }

    let bootstraps = match bootstrap_shell {
        Some(shell) => bootstrap_methods(bytes, shell, &facts.constant_pool, budget)?,
        None => Vec::new(),
    };
    let bootstrap_entries = u16::try_from(bootstraps.len()).map_err(|_| {
        Error::invalid_input(
            "classfile_size_overflow",
            "bootstrap method count does not fit u16",
        )
    })?;
    let dynamic_tag = registry.constant_pool_tag(TAG_DYNAMIC, major);
    let condy = condy_graph(
        &facts.constant_pool,
        &bootstraps,
        bootstrap_entries,
        bootstrap_shell.is_some(),
        dynamic_tag,
        budget,
        &mut diagnostics,
    )?;
    let concat = concat_sites(
        &facts.constant_pool,
        &bootstraps,
        major,
        budget,
        &mut diagnostics,
    )?;

    let mut result = ModernFacts {
        major_version: major,
        attributes,
        record,
        permitted_subclasses,
        concat,
        condy,
        output_level: OutputLevelStatus::NotEvaluated,
        diagnostics,
    };
    result.output_level = result.assess_output_level(level);
    Ok(result)
}

/// `attribute_name_index` of one class-level shell, read back from the entry's own span.
///
/// The shell carries the entry's name and its two ranges, and the index is the first field of the
/// range the shell starts at (JVMS 4.7). Reading it back keeps the fact's origin complete: a caller
/// can check that the index really spells the name the shell reports.
fn shell_name_index(bytes: &[u8], shell: &AttributeShell) -> Result<u16> {
    let start = usize::try_from(shell.span.start).map_err(|_| {
        Error::invalid_input(
            "classfile_invalid_attribute_span",
            "attribute start does not fit a byte offset",
        )
    })?;
    read_u16(bytes, start)
}

/// Reads the `Record` attribute's components.
fn read_record(
    bytes: &[u8],
    shell: &AttributeShell,
    pool: &[CpEntryFacts],
    rule: &'static AttributeRule,
    name_index: u16,
    budget: &mut Budget,
) -> Result<RecordFacts> {
    let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
    let count = reader.u16()?;
    let mut components = Vec::new();
    for index in 0..count {
        budget.poll()?;
        let component_name_index = reader.u16()?;
        let descriptor_index = reader.u16()?;
        let attribute_count = usize::from(reader.u16()?);
        let mut attributes = Vec::new();
        for _ in 0..attribute_count {
            budget.poll()?;
            let start = entry_offset(&shell.content_span, reader.position())?;
            let nested_name_index = reader.u16()?;
            let length = u64::from(reader.u32()?);
            let length_usize = usize::try_from(length).map_err(|_| {
                Error::invalid_input(
                    "classfile_invalid_attribute_content",
                    "record component attribute length does not fit a byte range",
                )
            })?;
            let content_start = entry_offset(&shell.content_span, reader.position())?;
            reader.skip(length_usize)?;
            let end = entry_offset(&shell.content_span, reader.position())?;
            attributes.push(NestedAttributeFact {
                name_index: nested_name_index,
                name: cp_utf8(pool, nested_name_index)?,
                span: ByteSpan::new(start, end - start),
                content_span: ByteSpan::new(content_start, length),
            });
        }
        charge_item(budget)?;
        components.push(RecordComponentFacts {
            index,
            name_index: component_name_index,
            name: cp_utf8(pool, component_name_index)?,
            descriptor_index,
            descriptor: cp_utf8(pool, descriptor_index)?,
            attributes,
        });
    }
    reader.expect_end()?;
    Ok(RecordFacts {
        name_index,
        span: shell.span.clone(),
        content_span: shell.content_span.clone(),
        components,
        rule,
    })
}

// ---------------------------------------------------------------------------
// The graph walk
// ---------------------------------------------------------------------------

/// One counted dimension of the walk, mapped onto the reader's own vocabulary.
#[derive(Clone, Copy)]
enum GraphDimension {
    Nodes,
    Edges,
    Steps,
}

impl GraphDimension {
    const fn counted(self) -> CountedBudgetDimension {
        match self {
            Self::Nodes => CountedBudgetDimension::IrItems,
            Self::Edges => CountedBudgetDimension::IrEdges,
            Self::Steps => CountedBudgetDimension::AnalysisSteps,
        }
    }

    const fn stop(self, limit: u64) -> CondyStop {
        match self {
            Self::Nodes => CondyStop::Nodes { limit },
            Self::Edges => CondyStop::Edges { limit },
            Self::Steps => CondyStop::Steps { limit },
        }
    }
}

/// One frame of the walk. The walk is an explicit stack rather than a recursive call, so a
/// pathological graph cannot consume host stack: the only depth there is, is the one the
/// `DependencyDepth` budget measures.
enum Step {
    Enter {
        node: CondyNodeRef,
        via: Vec<u32>,
    },
    /// Leaving a node: the node it names leaves the path set with it, so only a repeat that meets a
    /// node *still* on the path is a cycle.
    Exit(CondyNodeRef),
}

/// The mutable state of one class's graph walk.
struct Walk<'a> {
    pool: &'a [CpEntryFacts],
    bootstraps: &'a [BootstrapMethodFacts],
    nodes: Vec<CondyNode>,
    node_ids: BTreeSet<CondyNodeRef>,
    edges: Vec<CondyEdge>,
    edge_ids: BTreeMap<(CondyNodeRef, CondyNodeRef, CondyEdgeKind, Option<u16>), u32>,
    use_sites: Vec<CondyUseSite>,
    cycles: Vec<CondyCycle>,
    budget: CondyBudget,
}

impl Walk<'_> {
    /// Charges one graph dimension of the walk.
    ///
    /// `Ok(true)` means the charge was made, `Ok(false)` means this dimension's limit refused it —
    /// the stop is recorded and the walk ends without failing the read. Every other refusal, a
    /// cancellation or the elapsed-time limit above all, is the request's to absorb and is returned
    /// as the error it is: the graph never absorbs a stop it does not own.
    fn charge(&mut self, budget: &mut Budget, dimension: GraphDimension) -> Result<bool> {
        let counted = dimension.counted();
        match budget.charge(counted, 1) {
            Ok(()) => {
                match dimension {
                    GraphDimension::Nodes => self.budget.nodes += 1,
                    GraphDimension::Edges => self.budget.edges += 1,
                    GraphDimension::Steps => self.budget.steps += 1,
                }
                Ok(true)
            }
            Err(Error::BudgetExceeded {
                dimension: refused, ..
            }) if refused == BudgetDimension::from(counted) => {
                self.budget.stopped = Some(dimension.stop(budget.limits().counted_limit(counted)));
                Ok(false)
            }
            Err(other) => Err(other),
        }
    }

    /// Registers a node once, charging it as a derived storage item.
    fn ensure_node(&mut self, budget: &mut Budget, node: CondyNodeRef) -> Result<bool> {
        if self.node_ids.contains(&node) {
            return Ok(true);
        }
        if !self.charge(budget, GraphDimension::Nodes)? {
            return Ok(false);
        }
        let entry = match node {
            CondyNodeRef::ConstantPool { index } => {
                let fact = cp_entry(self.pool, index)?;
                CondyNode {
                    node,
                    kind: Some(node_kind(&fact.kind)),
                    span: Some(fact.span.clone()),
                }
            }
            CondyNodeRef::Bootstrap { .. } => CondyNode {
                node,
                kind: None,
                span: None,
            },
        };
        self.node_ids.insert(node);
        self.nodes.push(entry);
        Ok(true)
    }

    /// Adds an edge once and returns its ordinal. The same reference from two use-sites or two
    /// arguments keeps one ordinal, which is what makes a shared edge checkable in two `via` paths.
    fn edge(&mut self, budget: &mut Budget, edge: CondyEdge) -> Result<Option<u32>> {
        let key = (edge.from, edge.to, edge.kind, edge.argument_index);
        if let Some(ordinal) = self.edge_ids.get(&key) {
            return Ok(Some(*ordinal));
        }
        if !self.ensure_node(budget, edge.from)? || !self.ensure_node(budget, edge.to)? {
            return Ok(None);
        }
        if !self.charge(budget, GraphDimension::Edges)? {
            return Ok(None);
        }
        let ordinal = u32::try_from(self.edges.len()).map_err(|_| {
            Error::invalid_input(
                "classfile_size_overflow",
                "condy edge ordinal does not fit u32",
            )
        })?;
        self.edge_ids.insert(key, ordinal);
        self.edges.push(edge);
        Ok(Some(ordinal))
    }

    /// The references one node leaves, in the order the structures declare them.
    fn outgoing(&self, node: CondyNodeRef) -> Result<Vec<CondyEdge>> {
        let mut edges = Vec::new();
        match node {
            CondyNodeRef::ConstantPool { index } => {
                if let Some(bootstrap) = dynamic_bootstrap(&cp_entry(self.pool, index)?.kind) {
                    edges.push(CondyEdge {
                        kind: CondyEdgeKind::Bootstrap,
                        from: node,
                        to: CondyNodeRef::Bootstrap { index: bootstrap },
                        argument_index: None,
                    });
                }
            }
            CondyNodeRef::Bootstrap { index } => {
                let Some(entry) = self.bootstraps.get(usize::from(index)) else {
                    // The entry is not declared. The node stays in the graph — the reference exists
                    // and the diagnostic says why it is dangling — and it has no outgoing edge.
                    return Ok(edges);
                };
                edges.push(CondyEdge {
                    kind: CondyEdgeKind::BootstrapHandle,
                    from: node,
                    to: CondyNodeRef::ConstantPool {
                        index: entry.method_ref,
                    },
                    argument_index: None,
                });
                for (ordinal, argument) in entry.arguments.iter().enumerate() {
                    edges.push(CondyEdge {
                        kind: CondyEdgeKind::BootstrapArgument,
                        from: node,
                        to: CondyNodeRef::ConstantPool { index: *argument },
                        argument_index: Some(u16::try_from(ordinal).map_err(|_| {
                            Error::invalid_input(
                                "classfile_size_overflow",
                                "bootstrap argument ordinal does not fit u16",
                            )
                        })?),
                    });
                }
            }
        }
        Ok(edges)
    }

    /// Walks one use-site: `false` means the graph's budget is exhausted and every later use-site
    /// stays unentered.
    ///
    /// The reaches a stopped walk recorded are still pushed with their entry point: every path in
    /// them was really walked, so they are the answer's reliable prefix, and
    /// [`CondyGraph::budget`] states that the walk did not finish.
    fn walk_site(&mut self, budget: &mut Budget, site: CondyNodeRef) -> Result<bool> {
        if self.budget.stopped.is_some() {
            return Ok(false);
        }
        if !self.ensure_node(budget, site)? {
            return Ok(false);
        }
        let site_ordinal = self.use_sites.len();
        let mut reaches: Vec<CondyReach> = Vec::new();
        let complete = self.descend(budget, site, site_ordinal, &mut reaches)?;
        self.use_sites.push(CondyUseSite { site, reaches });
        Ok(complete)
    }

    /// The walk itself: an explicit stack, a visited set that expands a node once, and a path set
    /// that turns a repeat of a node still on the path into a recorded cycle.
    fn descend(
        &mut self,
        budget: &mut Budget,
        site: CondyNodeRef,
        site_ordinal: usize,
        reaches: &mut Vec<CondyReach>,
    ) -> Result<bool> {
        let mut visited: BTreeMap<CondyNodeRef, u32> = BTreeMap::new();
        let mut on_path: Vec<CondyNodeRef> = Vec::new();
        let mut on_path_ids: BTreeSet<CondyNodeRef> = BTreeSet::new();
        let mut stack = vec![Step::Enter {
            node: site,
            via: Vec::new(),
        }];
        while let Some(step) = stack.pop() {
            budget.poll()?;
            match step {
                Step::Exit(node) => {
                    on_path.pop();
                    on_path_ids.remove(&node);
                }
                Step::Enter { node, via } => {
                    if !self.charge(budget, GraphDimension::Steps)? {
                        return Ok(false);
                    }
                    let ordinal = u32::try_from(reaches.len()).map_err(|_| {
                        Error::invalid_input(
                            "classfile_size_overflow",
                            "condy reach ordinal does not fit u32",
                        )
                    })?;
                    let earlier = visited.get(&node).copied();
                    if let Some(earlier) = earlier {
                        // The visited set is what keeps a shared subgraph from being expanded twice
                        // and a cycle from being followed: the reach records where the path went,
                        // and the walk stops at the repeated node.
                        if on_path_ids.contains(&node) {
                            self.cycles.push(CondyCycle {
                                site: site_ordinal,
                                reach: ordinal,
                                node,
                            });
                        }
                        reaches.push(CondyReach {
                            node,
                            via,
                            repeat: Some(earlier),
                        });
                        continue;
                    }
                    visited.insert(node, ordinal);
                    reaches.push(CondyReach {
                        node,
                        via: via.clone(),
                        repeat: None,
                    });
                    on_path.push(node);
                    on_path_ids.insert(node);
                    stack.push(Step::Exit(node));
                    let mut children = Vec::new();
                    for edge in self.outgoing(node)? {
                        let Some(edge_ordinal) = self.edge(budget, edge.clone())? else {
                            return Ok(false);
                        };
                        let mut child_via = via.clone();
                        child_via.push(edge_ordinal);
                        let entering = u64::try_from(child_via.len()).unwrap_or(u64::MAX);
                        if budget.observe_dependency_depth(entering).is_err() {
                            self.budget.stopped = Some(CondyStop::Depth {
                                limit: budget.limits().dependency_depth,
                                entering,
                            });
                            return Ok(false);
                        }
                        self.budget.max_depth = self.budget.max_depth.max(entering);
                        children.push(Step::Enter {
                            node: edge.to,
                            via: child_via,
                        });
                    }
                    // Pushed reversed so the stack pops them in the order the structures declare
                    // them: the bootstrap handle first, then its arguments.
                    stack.extend(children.into_iter().rev());
                }
            }
        }
        Ok(true)
    }
}

/// Builds the deferred constant-dynamic graph of one class.
///
/// Every `CONSTANT_Dynamic` and `CONSTANT_InvokeDynamic` entry is an entry point: the entry is what
/// a consumer can name, and the constant pool is where the reader can name it without owning the
/// bytecode that used it. The paths therefore start at the entry, and the code-level use-site (its
/// BCI and opcode) is the bytecode reader's own fact, joined by the entry's index.
fn condy_graph(
    pool: &[CpEntryFacts],
    bootstraps: &[BootstrapMethodFacts],
    bootstrap_entries: u16,
    table_read: bool,
    dynamic_tag: ConstantPoolTagStatus,
    budget: &mut Budget,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<CondyGraph> {
    // A dynamic entry whose table entry is not declared is diagnosed only when this class *declares*
    // a table this pass read. A class whose `BootstrapMethods` entry the registry does not place has
    // no table to be broken, and that release fact is already stated by its own placement row: the
    // two statements are never folded into one another.
    let mut dangling: BTreeSet<u16> = BTreeSet::new();
    if table_read {
        for entry in pool {
            if let Some(index) = dynamic_bootstrap(&entry.kind)
                && usize::from(index) >= bootstraps.len()
                && dangling.insert(index)
            {
                diagnostics.push(fact_diagnostic(
                    "classfile_condy_bootstrap_out_of_range",
                    DiagnosticSeverity::Warning,
                    format!(
                        "constant-pool entry {} names BootstrapMethods entry {index}, which the attribute does not declare ({bootstrap_entries} declared)",
                        entry.index
                    ),
                ));
            }
        }
    }
    let mut walk = Walk {
        pool,
        bootstraps,
        nodes: Vec::new(),
        node_ids: BTreeSet::new(),
        edges: Vec::new(),
        edge_ids: BTreeMap::new(),
        use_sites: Vec::new(),
        cycles: Vec::new(),
        budget: CondyBudget {
            nodes: 0,
            edges: 0,
            steps: 0,
            max_depth: 0,
            stopped: None,
        },
    };
    for entry in pool {
        if dynamic_bootstrap(&entry.kind).is_none() {
            continue;
        }
        if !walk.walk_site(budget, CondyNodeRef::ConstantPool { index: entry.index })? {
            break;
        }
    }
    Ok(CondyGraph {
        nodes: walk.nodes,
        edges: walk.edges,
        use_sites: walk.use_sites,
        cycles: walk.cycles,
        bootstrap_entries,
        dynamic_tag,
        budget: walk.budget,
    })
}

/// The `BootstrapMethods` index a dynamic entry names, for the two dynamic tags.
fn dynamic_bootstrap(kind: &CpEntryKind) -> Option<u16> {
    match kind {
        CpEntryKind::Dynamic {
            bootstrap_method_attr_index,
            ..
        }
        | CpEntryKind::InvokeDynamic {
            bootstrap_method_attr_index,
            ..
        } => Some(*bootstrap_method_attr_index),
        _ => None,
    }
}

/// The node kind of one constant-pool entry.
fn node_kind(kind: &CpEntryKind) -> CondyNodeKind {
    match kind {
        CpEntryKind::Dynamic { .. } => CondyNodeKind::Dynamic,
        CpEntryKind::InvokeDynamic { .. } => CondyNodeKind::InvokeDynamic,
        CpEntryKind::MethodHandle { .. } => CondyNodeKind::MethodHandle,
        CpEntryKind::MethodType { .. } => CondyNodeKind::MethodType,
        _ => CondyNodeKind::StaticArgument,
    }
}

// ---------------------------------------------------------------------------
// Concat sites
// ---------------------------------------------------------------------------

/// Reads the class's modern string-concat sites.
///
/// A site is published only when the registry registers the `CONSTANT_InvokeDynamic` tag for this
/// release — the construct's carrier is what the release rule decides — and when the site's own
/// bootstrap handle names a `MethodRef` whose owner is `java.lang.invoke.StringConcatFactory`. Both
/// are reads of the bytes: no factory is called and no recipe is expanded.
fn concat_sites(
    pool: &[CpEntryFacts],
    bootstraps: &[BootstrapMethodFacts],
    major: u16,
    budget: &mut Budget,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<Vec<ConcatSite>> {
    let registry = feature_registry();
    let tag_rule = match registry.constant_pool_tag(TAG_INVOKE_DYNAMIC, major) {
        ConstantPoolTagStatus::Registered { rule } => rule,
        // The release does not register the tag: no site of this class is a fact this pass states.
        _ => return Ok(Vec::new()),
    };
    let mut sites = Vec::new();
    let mut unresolved: BTreeSet<u16> = BTreeSet::new();
    for entry in pool {
        budget.poll()?;
        let CpEntryKind::InvokeDynamic {
            bootstrap_method_attr_index,
            name,
            descriptor,
            ..
        } = &entry.kind
        else {
            continue;
        };
        let Some(bootstrap) = bootstraps.get(usize::from(*bootstrap_method_attr_index)) else {
            // The out-of-range reference is diagnosed by the graph walk, once per table entry.
            continue;
        };
        let handle = cp_entry(pool, bootstrap.method_ref)?;
        let CpEntryKind::MethodHandle {
            reference_index, ..
        } = &handle.kind
        else {
            continue;
        };
        let Some(factory) =
            cp_entry(pool, *reference_index)
                .ok()
                .and_then(|fact| match &fact.kind {
                    CpEntryKind::MethodRef {
                        owner,
                        name,
                        descriptor,
                        ..
                    } => Some((owner.clone(), name.clone(), descriptor.clone())),
                    _ => None,
                })
        else {
            if unresolved.insert(entry.index) {
                diagnostics.push(fact_diagnostic(
                    "classfile_concat_handle_unresolved",
                    DiagnosticSeverity::Info,
                    format!(
                        "invokedynamic entry {} names bootstrap handle {} whose reference is not a CONSTANT_MethodRef, so no factory can be named for the site",
                        entry.index, bootstrap.method_ref
                    ),
                ));
            }
            continue;
        };
        if factory.0.0.as_slice() != CONCAT_FACTORY {
            continue;
        }
        let strategy = match name.0.as_slice() {
            b"makeConcat" => ConcatStrategy::Concat,
            b"makeConcatWithConstants" => ConcatStrategy::ConcatWithConstants,
            _ => ConcatStrategy::Other,
        };
        let recipe = match (strategy, bootstrap.arguments.first()) {
            (ConcatStrategy::ConcatWithConstants, Some(argument)) => {
                match &cp_entry(pool, *argument)?.kind {
                    CpEntryKind::String { value, .. } => Some(value.clone()),
                    _ => None,
                }
            }
            _ => None,
        };
        charge_item(budget)?;
        sites.push(ConcatSite {
            constant_pool_index: entry.index,
            span: entry.span.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
            bootstrap_index: *bootstrap_method_attr_index,
            strategy,
            factory_owner: factory.0,
            factory_name: factory.1,
            factory_descriptor: factory.2,
            recipe,
            tag_rule,
        });
    }
    Ok(sites)
}

/// One diagnostic of this pass: a structural finding, with no provenance.
///
/// The position of the finding travels in the facts, not in the diagnostic: a reader fact pass over
/// a byte slice has no snapshot identity to name, and inventing one would be a claim about an
/// artifact this pass never opened.
fn fact_diagnostic(code: &str, severity: DiagnosticSeverity, message: String) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity,
        message,
        provenance: None,
    }
}
