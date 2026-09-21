//! X3: the bounded reflection and `ServiceLoader` patterns of one explicit scope (P4 2.3).
//!
//! # What this plane answers, and what it refuses to answer
//!
//! The requirement is narrow on purpose: a reflection call site answers with a
//! `pattern_inferred_target` **only** when the name it asks for is a constant this engine *proves*
//! under a bounded local value flow, and every other input is `Unknown` with the definition that
//! kept it from being a constant. There is no confidence number here and no open-ended search:
//!
//! * a target is inferred from two things and nothing else — a **constant** the class file's own
//!   bytes hold (`ldc` of a `String`, `ldc` of a `Class`, or a `getstatic` of a field the call
//!   site's own class declares with a `ConstantValue`) and the **registered rule** of the overload
//!   the call site names. The overload is part of the answer because it decides the conclusion:
//!   `Class.forName(String)` looks the name up in the caller's own loader, while
//!   `Class.forName(String, boolean, ClassLoader)` names a loader object this snapshot does not
//!   identify, and `ServiceLoader.load(Class)` uses the *thread context* loader — three loader
//!   assumptions, each stated in its own result;
//! * every other value is `Unknown`, never a guess: a parameter or `this` (entry state), a merge of
//!   two paths, a caught reference, a call result, a concatenation step, an array element, or a
//!   field whose value this slice does not prove — each named by the definition it came from;
//! * **nothing is executed and nothing is loaded.** No bootstrap, reflection call, launcher, JNI
//!   or network resource is touched, no class is initialized, and the target is never decoded or
//!   resolved as a member: the answer is a name-level statement about *what the call site asks
//!   for*. That is why a class the snapshot does not even hold still gets an inferred name — and
//!   why the snapshot's own answer about that name is published *beside* the inference as
//!   [`PatternTargetState`] and never folded into it;
//! * **the propagation is bounded by one body.** The value flow is the P3 4.3 SSA of the very
//!   method body the call site lives in (4.1's frames over 3.5's canonical graph), reached by one
//!   ordinary method-analysis run per body: no second dataflow is written here, no value is
//!   followed across a call, and the run's own `MethodBodies`, `ClassBytes`, `CodeBytes`,
//!   `IrItems`, `IrEdges` and `AnalysisSteps` charges are what bounds it. A refused charge, a
//!   listing that stopped and a damaged read all end the scan with the prefix it published, and
//!   the report says so instead of completing over range it never searched.
//!
//! # Evidence a reviewer can check
//!
//! An inferred site carries five things and no summary judgement about them: the **API overload**
//! (as the constant pool spells it, beside the registered rule id), the **constant input** (the
//! bytes the class file really holds), the **propagation scope** ([`Propagation`]: the proving
//! instruction, the bounded path the value travelled — its source, every store and load it was
//! carried through, and the call site — and the budget dimension that bounds the flow), the **loader
//! assumption** ([`LoaderAssumption`], with the loader itself when the assumption is the request's
//! own caller domain), and the **rule version** ([`RuleVersion`]). [`PatternInference`] has exactly
//! those fields plus the target they support, so a conclusion cannot be published without them.
//!
//! # What this slice does not claim
//!
//! Every registered rule carries a non-empty `not_claimed` list (an invariant the tests pin), and
//! an overload the registry does not hold is **not a subject of this plane at all**: it produces
//! no site record, because "no rule" is not "not supported" — the discipline the release registry
//! states for a name it does not hold. Four overloads this project deliberately derives no target
//! from are registered as [`PatternSupport::Unsupported`] *with their reason*, so a reader sees the
//! boundary instead of an omission.
//!
//! # Where the answer comes from, and what it costs
//!
//! The scope is the P1 physical scope, enumerated by name exactly as 2.5's dispatch range is
//! (`SnapshotAll` or one tree root), and every name is demanded through the 2.2 closure under the
//! request's caller loader, so a position the order resolves outside the scope, or cannot resolve
//! at all, is unread range and never an exclusion (its name is published in
//! [`ReflectionPatternReport::unresolved_dependencies`], the record 2.2 established). A class
//! whose constant pool names no registered overload is skipped without reading a body. For a class
//! that does name one, each member with a body is analysed once and **only that run's own decode,
//! pool and SSA table are read**: nothing is decoded twice and no table is rebuilt from a report.
//!
//! Site records exist exactly when this scan read the body's decode. A body whose run stopped
//! before it produced one leaves its call sites unread, and that is stated at the scan's own plane
//! — a charged diagnostic and a partial coverage — rather than guessed at per site. A run that
//! stopped *after* its decode is stated twice on purpose: every site of that body is `Unknown`
//! with the run's own stop code, and the answer is a prefix, because the values of a body whose
//! analysis stopped were never named.
//!
//! The scan publishes sites in range order — class, member declaration order, BCI — charging one
//! `ResultItems` per site and per domain diagnostic before it is published, exactly like 2.2's
//! declaration-reference query. `reads` publishes the scan's own class-name reads followed by each
//! analysed body's own run records, because each run is its own request-scoped read (so one
//! `(definition, loader)` binding may appear once per run); `unresolved_dependencies` and the
//! environment and stop planes are control metadata and are not charged.

use std::fmt;

use serde::Serialize;

use jarde_reader::accounting::with_usage;
use jarde_reader::artifact::{ArtifactSnapshot, budget_dimension_code};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    ClassFacts, CpEntryFacts, CpEntryKind, MemberHeader, MethodCodeFacts, attribute_facts, cp_entry,
};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    Coverage, CoverageDimension, CoverageRange, CoverageState, Diagnostic, DiagnosticSeverity,
    ExecutionReport, JvmBytes, PhysicalDefinitionId, PhysicalMethodId, TerminationReason,
};
use jarde_reader::view::{LoaderId, PhysicalScope};

use crate::dispatch::{covered_by_scope, enumerate_range};
use crate::environment::{
    EnvironmentIdentity, EnvironmentProblem, ResolutionEnvironment, environment_diagnostics,
    require_content_snapshot, unavailable_diagnostic, validate_environment,
};
use crate::ir::{AnalysisStage, MethodAnalysisRequest};
use crate::providers::{HeaderClosure, HeaderDemand, HeaderLookupState};
use crate::resolver::{
    DependencyGap, HeaderRead, RESOLUTION_NOT_IMPLEMENTED, ReadReason, ResolutionAnalysis,
    UnresolvedDependency, published_reads, search_coverage_with_artifact,
};
use crate::ssa::{Definition, Slot, SsaInstruction, SsaTable, ValueId};

/// The name and version of one registered rule, in the shape P3's pass table states.
///
/// The type is this crate's own mirror of `jarde_java::pass::RuleVersion` and not that type: the
/// JVM layers do not depend on the recovery layer, and a version string is a fact about *this*
/// registry. The spelling is the same on purpose — `name@version` — so a reader of either report
/// reads one convention.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct RuleVersion {
    name: &'static str,
    version: &'static str,
}

impl RuleVersion {
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

impl fmt::Display for RuleVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.name, self.version)
    }
}

/// Which input of an overload names the thing the call is about.
///
/// `Argument(n)` is the `n`-th parameter of the descriptor, counting from `0`; `Receiver` is the
/// `this` of an instance call. A rule names one or the other, never both, and the tests pin that
/// the place it names exists in its own descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputPlace {
    /// The `n`-th parameter of the registered descriptor.
    Argument(u8),
    /// The receiver of an instance call.
    Receiver,
}

/// The inputs one registered overload reads its target from.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PatternInputs {
    /// The parameter that spells the target: a binary class name for a type or a service
    /// interface, a member name for a member lookup.
    pub name: InputPlace,
    /// The class a member is looked up on, for the overloads that name one. `None` for the
    /// overloads whose target *is* a class.
    pub owner: Option<InputPlace>,
}

/// What a registered overload asks for.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ReflectedTargetKind {
    /// A class, named by a string constant (`Class.forName`).
    TypeName,
    /// A service interface, named by a `Class` literal (`ServiceLoader.load`).
    ServiceInterface,
    /// One member of one class, named by a member-name constant on a class the site also names.
    Member {
        /// Whether the looked-up member is a method or a field.
        member: ReflectedMemberKind,
    },
}

/// Which member kind a member lookup asks for.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflectedMemberKind {
    Method,
    Field,
}

/// What a registered overload says about the loader that would define the named target.
///
/// The distinction is the reason two overloads of one API state different conclusions: the loader
/// decides which order a name is looked up in, and a loader this snapshot cannot identify is a
/// *stated* assumption rather than a forbidden answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderAssumptionKind {
    /// The loader that defined the class the call site lives in — the assumption the JLS states for
    /// `Class.forName(String)`, and the one this plane can look up, because the request names the
    /// caller's own load domain.
    CallerDefiningLoader,
    /// The current thread's context loader: runtime state no snapshot holds, so the name is
    /// inferred and no order is searched for it.
    ThreadContextLoader,
    /// A loader object the call site passes: an identity the class file does not spell, so the name
    /// is inferred and no order is searched for it.
    ExplicitLoaderArgument,
}

impl LoaderAssumptionKind {
    /// The assumption in one sentence, as the result states it.
    pub const fn statement(self) -> &'static str {
        match self {
            Self::CallerDefiningLoader => {
                "the class the call site is in is defined by the request's declared caller loader, \
                 so the name is looked up in that loader's own order and the definition it selects \
                 is published beside the inference"
            }
            Self::ThreadContextLoader => {
                "the loader is the current thread's context loader, which is runtime state this \
                 snapshot does not hold: the name is what the call site asks for, and no order is \
                 searched for it"
            }
            Self::ExplicitLoaderArgument => {
                "the loader is an argument object the class file does not identify: the name is \
                 what the call site asks for, and no order is searched for it"
            }
        }
    }
}

/// Whether this slice derives a target from one registered overload.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PatternSupport {
    /// The overload's target inputs are read, and a target is inferred when they are constants.
    Supported,
    /// The overload is registered and this slice deliberately derives nothing from it. The reason
    /// is what a reader checks instead of an omission.
    Unsupported {
        /// Why this overload is registered as unsupported.
        reason: &'static str,
    },
}

/// One declared pattern: an API overload, the inputs it reads its target from, the loader it
/// assumes, and the rule version that decided the conclusion.
///
/// A record states what it *does* claim (`support`, the inputs, the loader assumption) and, in
/// `not_claimed`, what it deliberately does not — the shape the release registry gives its own
/// entries, so a boundary is a readable record rather than an absence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PatternRule {
    /// Stable rule id, as the report spells it.
    pub id: &'static str,
    /// Internal name of the API's owner, as the constant pool spells it.
    pub owner: &'static str,
    /// The API's name.
    pub name: &'static str,
    /// The overload's descriptor: two overloads of one name are two rules.
    pub descriptor: &'static str,
    /// The inputs this overload's target is read from.
    pub inputs: PatternInputs,
    /// What the overload asks for.
    pub target: ReflectedTargetKind,
    /// The loader the conclusion is stated under.
    pub loader: LoaderAssumptionKind,
    /// The rule version.
    pub rule: RuleVersion,
    /// Where the rule was read from.
    pub source: &'static str,
    /// What this rule deliberately does not claim. Never empty.
    pub not_claimed: &'static [&'static str],
    /// Whether this slice derives a target from the overload.
    pub support: PatternSupport,
}

/// The registered patterns, in registry order.
///
/// The table is the whole vocabulary of this plane: a call site answers a rule when its owner, name
/// and descriptor are equal to one entry's, byte for byte. `Class.forName(String)` and
/// `Class.forName(String, boolean, ClassLoader)` are two entries because they are two overloads
/// with two loader assumptions, and a descriptor this table does not hold is not an X3 subject.
pub const PATTERNS: &[PatternRule] = &[
    PatternRule {
        id: "class-for-name",
        owner: "java/lang/Class",
        name: "forName",
        descriptor: "(Ljava/lang/String;)Ljava/lang/Class;",
        inputs: PatternInputs {
            name: InputPlace::Argument(0),
            owner: None,
        },
        target: ReflectedTargetKind::TypeName,
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("class-for-name", "1"),
        source: "JLS 12.2 / JDK javadoc: Class.forName(String)",
        not_claimed: &[
            "the class the name denotes is not loaded, linked or initialized by this engine",
            "a name the snapshot's order does not provide is published as a missing dependency, \
             never as the statement that the class does not exist",
            "the descriptor-typed overload is a different rule with a different loader assumption",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "class-for-name-loader",
        owner: "java/lang/Class",
        name: "forName",
        descriptor: "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
        inputs: PatternInputs {
            name: InputPlace::Argument(0),
            owner: None,
        },
        target: ReflectedTargetKind::TypeName,
        loader: LoaderAssumptionKind::ExplicitLoaderArgument,
        rule: RuleVersion::new("class-for-name-loader", "1"),
        source: "JDK javadoc: Class.forName(String, boolean, ClassLoader)",
        not_claimed: &[
            "the loader argument is not propagated: no order is searched for the name",
            "the initialization flag changes nothing this plane states, and is not read",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "class-get-method",
        owner: "java/lang/Class",
        name: "getMethod",
        descriptor: "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;",
        inputs: PatternInputs {
            name: InputPlace::Argument(0),
            owner: Some(InputPlace::Receiver),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Method,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("class-get-method", "1"),
        source: "JDK javadoc: Class.getMethod(String, Class...)",
        not_claimed: &[
            "the parameter types are not propagated, so the member is named and never identified",
            "the superclass chain is not searched: whether a public method of that name exists is \
             not this plane's statement",
            "the receiver must be a class literal constant; a computed receiver is Unknown",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "class-get-declared-method",
        owner: "java/lang/Class",
        name: "getDeclaredMethod",
        descriptor: "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;",
        inputs: PatternInputs {
            name: InputPlace::Argument(0),
            owner: Some(InputPlace::Receiver),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Method,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("class-get-declared-method", "1"),
        source: "JDK javadoc: Class.getDeclaredMethod(String, Class...)",
        not_claimed: &[
            "the target class is not decoded and its member list is not read: a name that class \
             does not declare is still the name the call site asks for",
            "the parameter types are not propagated, so the member is named and never identified",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "class-get-field",
        owner: "java/lang/Class",
        name: "getField",
        descriptor: "(Ljava/lang/String;)Ljava/lang/reflect/Field;",
        inputs: PatternInputs {
            name: InputPlace::Argument(0),
            owner: Some(InputPlace::Receiver),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Field,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("class-get-field", "1"),
        source: "JDK javadoc: Class.getField(String)",
        not_claimed: &[
            "the field is not resolved: its type and its declaration are not read",
            "the receiver must be a class literal constant; a computed receiver is Unknown",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "class-get-declared-field",
        owner: "java/lang/Class",
        name: "getDeclaredField",
        descriptor: "(Ljava/lang/String;)Ljava/lang/reflect/Field;",
        inputs: PatternInputs {
            name: InputPlace::Argument(0),
            owner: Some(InputPlace::Receiver),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Field,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("class-get-declared-field", "1"),
        source: "JDK javadoc: Class.getDeclaredField(String)",
        not_claimed: &[
            "the field is not resolved: its declaration and type are not read",
            "the receiver must be a class literal constant; a computed receiver is Unknown",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "methodhandles-find-static",
        owner: "java/lang/invoke/MethodHandles$Lookup",
        name: "findStatic",
        descriptor: "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;",
        inputs: PatternInputs {
            name: InputPlace::Argument(1),
            owner: Some(InputPlace::Argument(0)),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Method,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("methodhandles-find-static", "1"),
        source: "JDK javadoc: MethodHandles.Lookup.findStatic(Class, String, MethodType)",
        not_claimed: &[
            "the method type is not propagated: the member is named and never identified",
            "the lookup class and the access rules of the lookup are not evaluated",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "methodhandles-find-virtual",
        owner: "java/lang/invoke/MethodHandles$Lookup",
        name: "findVirtual",
        descriptor: "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;",
        inputs: PatternInputs {
            name: InputPlace::Argument(1),
            owner: Some(InputPlace::Argument(0)),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Method,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("methodhandles-find-virtual", "1"),
        source: "JDK javadoc: MethodHandles.Lookup.findVirtual(Class, String, MethodType)",
        not_claimed: &[
            "the method type is not propagated: the member is named and never identified",
            "which override the handle would reach at run time is a dispatch question, not this \
             plane's",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "methodhandles-find-special",
        owner: "java/lang/invoke/MethodHandles$Lookup",
        name: "findSpecial",
        descriptor: "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;",
        inputs: PatternInputs {
            name: InputPlace::Argument(1),
            owner: Some(InputPlace::Argument(0)),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Method,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("methodhandles-find-special", "1"),
        source: "JDK javadoc: MethodHandles.Lookup.findSpecial(Class, String, MethodType, Class)",
        not_claimed: &[
            "the method type and the special caller are not propagated",
            "the access rules of the lookup are not evaluated",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "methodhandles-find-getter",
        owner: "java/lang/invoke/MethodHandles$Lookup",
        name: "findGetter",
        descriptor: "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;",
        inputs: PatternInputs {
            name: InputPlace::Argument(1),
            owner: Some(InputPlace::Argument(0)),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Field,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("methodhandles-find-getter", "1"),
        source: "JDK javadoc: MethodHandles.Lookup.findGetter(Class, String, Class)",
        not_claimed: &[
            "the field type is not propagated: the field is named and never identified",
            "the access rules of the lookup are not evaluated",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "methodhandles-find-setter",
        owner: "java/lang/invoke/MethodHandles$Lookup",
        name: "findSetter",
        descriptor: "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;",
        inputs: PatternInputs {
            name: InputPlace::Argument(1),
            owner: Some(InputPlace::Argument(0)),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Field,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("methodhandles-find-setter", "1"),
        source: "JDK javadoc: MethodHandles.Lookup.findSetter(Class, String, Class)",
        not_claimed: &[
            "the field type is not propagated: the field is named and never identified",
            "the access rules of the lookup are not evaluated",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "service-loader-load",
        owner: "java/util/ServiceLoader",
        name: "load",
        descriptor: "(Ljava/lang/Class;)Ljava/util/ServiceLoader;",
        inputs: PatternInputs {
            name: InputPlace::Argument(0),
            owner: None,
        },
        target: ReflectedTargetKind::ServiceInterface,
        loader: LoaderAssumptionKind::ThreadContextLoader,
        rule: RuleVersion::new("service-loader-load", "1"),
        source: "JDK javadoc: ServiceLoader.load(Class)",
        not_claimed: &[
            "the service's provider configuration is not read: no `META-INF/services` resource is \
             opened and no provider is enumerated",
            "the interface is not decoded: its own name is what the call site states",
            "the thread context loader is runtime state, so the interface is a name and not a \
             definition this snapshot selected",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "service-loader-load-loader",
        owner: "java/util/ServiceLoader",
        name: "load",
        descriptor: "(Ljava/lang/Class;Ljava/lang/ClassLoader;)Ljava/util/ServiceLoader;",
        inputs: PatternInputs {
            name: InputPlace::Argument(0),
            owner: None,
        },
        target: ReflectedTargetKind::ServiceInterface,
        loader: LoaderAssumptionKind::ExplicitLoaderArgument,
        rule: RuleVersion::new("service-loader-load-loader", "1"),
        source: "JDK javadoc: ServiceLoader.load(Class, ClassLoader)",
        not_claimed: &[
            "the service's provider configuration is not read",
            "the loader argument is not propagated: no order is searched for the interface",
        ],
        support: PatternSupport::Supported,
    },
    PatternRule {
        id: "methodhandles-find-constructor",
        owner: "java/lang/invoke/MethodHandles$Lookup",
        name: "findConstructor",
        descriptor: "(Ljava/lang/Class;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;",
        inputs: PatternInputs {
            name: InputPlace::Argument(1),
            owner: Some(InputPlace::Argument(0)),
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Method,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("methodhandles-find-constructor", "1"),
        source: "JDK javadoc: MethodHandles.Lookup.findConstructor(Class, MethodType)",
        not_claimed: &[
            "no target is derived from this overload: the member name is `<init>` by definition of \
             the API, and the method type is not propagated, so a name-level inference would \
             restate the class argument and add nothing",
        ],
        support: PatternSupport::Unsupported {
            reason: "the member name is fixed by the API and the method type is not propagated, so \
                     this slice has no name constant to read",
        },
    },
    PatternRule {
        id: "class-get-declared-methods",
        owner: "java/lang/Class",
        name: "getDeclaredMethods",
        descriptor: "()[Ljava/lang/reflect/Method;",
        inputs: PatternInputs {
            name: InputPlace::Receiver,
            owner: None,
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Method,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("class-get-declared-methods", "1"),
        source: "JDK javadoc: Class.getDeclaredMethods()",
        not_claimed: &[
            "no target is derived from this overload: the call enumerates every declared method \
             and names none",
        ],
        support: PatternSupport::Unsupported {
            reason: "the overload takes no name: it enumerates the receiver's members, so there is \
                     no constant input to read",
        },
    },
    PatternRule {
        id: "class-get-constructors",
        owner: "java/lang/Class",
        name: "getConstructors",
        descriptor: "()[Ljava/lang/reflect/Constructor;",
        inputs: PatternInputs {
            name: InputPlace::Receiver,
            owner: None,
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Method,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("class-get-constructors", "1"),
        source: "JDK javadoc: Class.getConstructors()",
        not_claimed: &[
            "no target is derived from this overload: the call enumerates every public constructor \
             and names none",
        ],
        support: PatternSupport::Unsupported {
            reason: "the overload takes no name: it enumerates the receiver's constructors, so \
                     there is no constant input to read",
        },
    },
    PatternRule {
        id: "class-get-declared-fields",
        owner: "java/lang/Class",
        name: "getDeclaredFields",
        descriptor: "()[Ljava/lang/reflect/Field;",
        inputs: PatternInputs {
            name: InputPlace::Receiver,
            owner: None,
        },
        target: ReflectedTargetKind::Member {
            member: ReflectedMemberKind::Field,
        },
        loader: LoaderAssumptionKind::CallerDefiningLoader,
        rule: RuleVersion::new("class-get-declared-fields", "1"),
        source: "JDK javadoc: Class.getDeclaredFields()",
        not_claimed: &[
            "no target is derived from this overload: the call enumerates every declared field and \
             names none",
        ],
        support: PatternSupport::Unsupported {
            reason: "the overload takes no name: it enumerates the receiver's fields, so there is \
                     no constant input to read",
        },
    },
];

/// Every registered pattern, in registry order.
pub const fn patterns() -> &'static [PatternRule] {
    PATTERNS
}

/// The rule one call site answers, or `None` when this registry holds no such overload.
///
/// The comparison is byte-exact on all three dimensions: an owner, name or descriptor this table
/// does not hold is a call site this plane makes no claim about, which is a different statement
/// from an overload registered as unsupported.
pub fn pattern_for(owner: &[u8], name: &[u8], descriptor: &[u8]) -> Option<&'static PatternRule> {
    PATTERNS.iter().find(|rule| {
        rule.owner.as_bytes() == owner
            && rule.name.as_bytes() == name
            && rule.descriptor.as_bytes() == descriptor
    })
}

/// One X3 scan request: an explicit environment and the physical scope whose classes are read.
///
/// The caller's own load domain is the request's caller loader for every class of the scope (the
/// closure's caller demand), which is why no second field names a caller: the assumption
/// [`LoaderAssumptionKind::CallerDefiningLoader`] states *is* that domain.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectionPatternRequest {
    pub environment: ResolutionEnvironment,
    pub scope: PhysicalScope,
    /// `0` means "no item limit". An item limit is not a cursor: a truncated scan replays nothing.
    pub max_items: u64,
}

/// The API overload one call site names, as the constant pool of its own class spells it.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Overload {
    pub owner: JvmBytes,
    pub name: JvmBytes,
    pub descriptor: JvmBytes,
}

/// Where the value of one input came from, when it is not a constant this slice proves.
///
/// The variants are the *definitions* the SSA table states, so a reader sees why the input is
/// Unknown instead of reading a guess: a parameter and a receiver are entry state (no instruction
/// produces them), a merge is several paths meeting, a caught reference arrives from a throw site,
/// and an instruction of any other shape — a call result, a concatenation step, an array element, a
/// field whose value this slice does not prove — is named by its BCI and opcode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case", tag = "origin")]
pub enum DynamicOrigin {
    /// The method's own entry state: a parameter, `this`, or a value its caller hands it.
    Entry,
    /// A merge point: the value of several paths, none of which this slice folds.
    Merge,
    /// A caught reference.
    Caught,
    /// An instruction this slice does not treat as a constant source for this input.
    Instruction {
        /// Bytecode index of the defining instruction.
        bci: u32,
        /// Its opcode.
        opcode: u8,
    },
    /// The value table of this body records no definition for the input's place.
    NoDefinition,
    /// The value's replacement chain exceeded this slice's bound, so nothing is claimed about it.
    ChainBeyondBound,
}

/// Which input of a rule kept a site from being inferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternInput {
    /// The constant the target is named by.
    TargetName,
    /// The class literal a member is looked up on.
    OwnerClass,
}

/// Why one site is not an inferred target.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PatternUnknown {
    /// The input's value is not a constant this slice proves; the definition that produced it is
    /// named, so the answer is a checked "no" rather than a guess.
    DynamicInput {
        input: PatternInput,
        origin: DynamicOrigin,
    },
    /// The body could not be analysed: the method-analysis run that would name its values stopped
    /// under this code. Nothing about the site's inputs is claimed.
    AnalysisStopped {
        /// The code the run stopped under.
        code: String,
    },
    /// The call site names an overload this registry holds and this slice deliberately derives no
    /// target from.
    PatternNotSupported {
        /// The registered rule id.
        rule: &'static str,
        /// Why it is registered as unsupported.
        reason: &'static str,
    },
}

/// The one instruction this slice proved an input's value comes from.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConstantSource {
    /// The proving instruction's bytecode index.
    pub bci: u32,
    /// What kind of constant it produced.
    pub kind: ConstantSourceKind,
    /// The assumption that makes the source a value *at the call site*, when the source is not a
    /// literal of this method's own bytes. `None` for `ldc`, which cannot change.
    pub assumption: Option<&'static str>,
}

/// The constant forms this slice proves.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ConstantSourceKind {
    /// `ldc`/`ldc_w` of a `CONSTANT_String`.
    LdcString,
    /// `ldc`/`ldc_w` of a `CONSTANT_Class`: a class literal.
    LdcClass,
    /// `getstatic` of a field the call site's own class declares with a `ConstantValue`.
    StaticFieldConstantValue {
        owner: JvmBytes,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
}

/// The bounded propagation scope of one inference.
///
/// "Bounded" is a property of the evidence and not a promise: the value is the SSA value the call
/// site reads in its own body, the path lists every step the proof took inside that body, and no
/// step crosses a call — a value a callee returned is an ordinary instruction result and therefore
/// Unknown.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Propagation {
    /// The proving instruction.
    pub source: ConstantSource,
    /// How many stores and loads the proof followed between the source and the call site. `0` means
    /// the call site reads the source's own value.
    pub transfers: u32,
    /// The bytecode indexes the proof walked, in the order the value travelled: the source, one
    /// entry per store or load it was carried through, and the call site's own index last. A reader
    /// can replay exactly these instructions, which is what makes the transfer visible instead of
    /// asserted.
    pub path: Vec<u32>,
    /// The budget dimension whose limits bound this flow — the method-analysis run's own charge —
    /// so a reader sees which limit would stop it.
    pub budget_dimension: &'static str,
}

/// What the snapshot's own order states about the name a proven constant spells.
///
/// This plane does not resolve the target: it publishes the inference and this separate fact beside
/// it, so "what the call site asks for" and "what the snapshot provides" are never one claim. A name
/// the order does not provide is a missing dependency (2.2's record), not the statement that the
/// class does not exist.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum PatternTargetState {
    /// The named class resolved to one definition of the declared order.
    Resolved {
        loader: LoaderId,
        definition: PhysicalDefinitionId,
    },
    /// No position of the order provides the name.
    NotInSnapshot { loader: LoaderId },
    /// Positions of the order hold the name and cannot be told apart.
    Ambiguous { loader: LoaderId },
    /// No order was searched for the name, and why.
    NotDemanded { reason: &'static str },
}

/// One inferred reflection target: the five elements a reviewer checks and nothing summarised.
///
/// The fields are exactly the requirement's list — the overload's rule, the constant input, the
/// propagation scope, the loader assumption and the rule version — plus what the call site asks for
/// and the snapshot's own answer about it. This is an **inference**: unlike an X1 structural edge (a
/// byte the class file really holds) and unlike an X2 resolution (a declaration the order really
/// selected), it is what this engine *derives* from a bounded constant and a registered rule, and it
/// says so.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PatternInference {
    /// The registered rule that answered, by id.
    pub rule: &'static str,
    /// The rule version.
    pub rule_version: RuleVersion,
    /// The constant input, as the class file spells it: the actual value, not a reference to it.
    pub constant_input: JvmBytes,
    /// What the call site asks for.
    pub target: ReflectedTarget,
    /// The bounded propagation that proved the constant.
    pub propagation: Propagation,
    /// The loader assumption the conclusion is stated under.
    pub loader: LoaderAssumption,
    /// What the snapshot's own order states about the named class.
    pub resolution: PatternTargetState,
}

/// The loader assumption of one inference, with the loader itself when the assumption is the
/// request's own caller domain.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoaderAssumption {
    /// Which loader the conclusion is stated under.
    pub kind: LoaderAssumptionKind,
    /// The loader the name is looked up in, present exactly for
    /// [`LoaderAssumptionKind::CallerDefiningLoader`].
    pub loader: Option<LoaderId>,
    /// The assumption in one sentence, as the result states it.
    pub statement: &'static str,
}

/// What one inferred call site asks for.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ReflectedTarget {
    /// A class, by binary name.
    Type { name: JvmBytes },
    /// A service interface, by binary name.
    ServiceInterface { name: JvmBytes },
    /// One member of one class, by member name. The descriptor is deliberately not part of this
    /// statement, because it is not propagated.
    Member {
        owner: JvmBytes,
        name: JvmBytes,
        member: ReflectedMemberKind,
    },
}

/// The state of one call site this plane answers.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum ReflectionSiteState {
    /// The target was inferred from a proven constant under a registered rule. An inference, and
    /// never a fact about what the runtime will do.
    PatternInferredTarget(Box<PatternInference>),
    /// Nothing is claimed about the target, and why.
    Unknown { reason: PatternUnknown },
}

/// One answered call site: where it is, what overload it names, which rule answered it, and what.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectionSite {
    /// The class file the call site was read from.
    pub definition: PhysicalDefinitionId,
    /// The member that holds the call site.
    pub method: PhysicalMethodId,
    /// The call site's bytecode index.
    pub bci: u32,
    /// The overload, as the constant pool spells it.
    pub overload: Overload,
    /// The registered rule the call site answered, by id.
    pub rule: &'static str,
    /// The inference, or the reason there is none.
    pub state: ReflectionSiteState,
}

/// What one X3 scan produced.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectionPatternReport {
    pub environment_identity: EnvironmentIdentity,
    pub environment_problems: Vec<EnvironmentProblem>,
    pub scope: PhysicalScope,
    /// Whether the scan ran at all. A rejected environment never yields a definition, so no name is
    /// demanded and the honest state is `NotPerformed`.
    pub analysis: ResolutionAnalysis,
    /// The sites this scan answered, in range order: class, member declaration order, BCI.
    /// Publishing one costs one `ResultItems`, charged before it is published.
    pub sites: Vec<ReflectionSite>,
    /// Every class of the scope the declared order did not resolve — published by name under the
    /// demand and the loader that searched it — plus every name a proven constant spelled that no
    /// position provides. Uncharged evidence, exactly like 2.2's own plane.
    pub unresolved_dependencies: Vec<UnresolvedDependency>,
    /// Whether the answer is a prefix: the listing, a refused charge, a refusal, an item limit or
    /// range this scan could not read ended it before the end of its scope.
    pub has_more: bool,
    /// How many sites the report publishes.
    pub returned_items: u64,
    /// The scan's own class-name reads, then the read records of each analysed body's own run, in
    /// run order. Each run is its own request-scoped read, so one `(definition, loader)` binding may
    /// appear once per run — the totals are the runs' own, not one request's.
    pub reads: Vec<HeaderRead>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    /// The environment's own problems (one diagnostic each), one diagnostic per body whose analysis
    /// stopped, and the stop that ended the scan. A refused charge keeps the diagnostics already
    /// published and stops.
    pub diagnostics: Vec<Diagnostic>,
}

impl ReflectionPatternReport {
    /// The sites this scan inferred a target for, in report order.
    pub fn inferred_sites(&self) -> impl Iterator<Item = &ReflectionSite> {
        self.sites
            .iter()
            .filter(|site| matches!(site.state, ReflectionSiteState::PatternInferredTarget(_)))
    }

    /// The sites this scan could not answer, in report order.
    pub fn unknown_sites(&self) -> impl Iterator<Item = &ReflectionSite> {
        self.sites
            .iter()
            .filter(|site| matches!(site.state, ReflectionSiteState::Unknown { .. }))
    }
}

/// Request-level checks of one X3 scan: shape only, no artifact access.
///
/// The scope rule is the P1/2.4/2.5 one: the only root a fresh snapshot establishes is its own root
/// container, so a scope that names another one cannot describe this snapshot and is refused
/// instead of being silently ignored.
pub(crate) fn validate_request(
    content: &[ArtifactSnapshot],
    request: &ReflectionPatternRequest,
) -> Result<()> {
    require_content_snapshot(content, &request.environment.runtime.physical.snapshot)?;
    if let PhysicalScope::ArtifactTree { root_container } = &request.scope
        && root_container.0 != "root"
    {
        return Err(Error::invalid_input(
            "query_artifact_tree_root_mismatch",
            "pattern scan tree root does not match the snapshot root container",
        ));
    }
    Ok(())
}

/// The code of the diagnostic a scan publishes for a class whose bodies it did not analyse.
const PATTERN_BODY_NOT_ANALYSABLE: &str = "resolution_pattern_body_not_analysable";

/// The label of the artifact-structural range this scan publishes.
const CLASS_RANGE_LABEL: &str = "pattern_scan_class_name";

/// The budget dimension that bounds one site's value flow, as the report states it.
const FLOW_BOUND: &str = "analysis_steps";

/// The assumption a `getstatic` source carries: a static field is process state.
const STATIC_FIELD_ASSUMPTION: &str = "the declaring class's `ConstantValue` is the field's initial \
                                       value, and a static field is process state: a writer this \
                                       snapshot does not hold is not excluded";

/// How many replacement links a read's value may stand in before this slice claims nothing.
///
/// SSA folds a trivial phi into its operand and ties the two together with `replaced_by`; a chain
/// longer than this bound is a shape this slice does not follow, and the answer is Unknown rather
/// than a guess about where the chain ends.
const REPLACEMENT_CHASE_BOUND: usize = 8;

/// How many local loads the proof follows before it claims nothing about the value.
///
/// A value reaches its use through the local slots it was stored in, and each load that hands it
/// on is one step of the same body's value flow. The bound is this slice's own: a chain longer than
/// it is a shape this plane does not follow, and the answer is Unknown rather than a guess about
/// where the chain ends.
const ALIAS_CHASE_BOUND: usize = 8;

/// Whether one opcode loads a local, and so hands the value that slot holds on to the next reader.
///
/// The forms are the four typed loads and their implicit-slot shapes (JVMS 6.5): `iload`…`aload`
/// and `iload_<n>`…`aload_<n>`. An array load (`aaload`, `iaload`, …) reads an element of a runtime
/// array and is deliberately not one: its value is not a local's.
const fn is_local_load(opcode: u8) -> bool {
    matches!(opcode, 0x15..=0x2d)
}

/// Whether one opcode stores into a local, and so carries the value it popped on to whoever loads
/// that slot back.
///
/// The forms are the five typed stores and their implicit-slot shapes (JVMS 6.5): `istore`…`astore`
/// and `istore_<n>`…`astore_<n>`. An array store (`aastore`, `iastore`, …) writes a runtime array
/// and is deliberately not one: what it holds is not a local this plane follows.
const fn is_local_store(opcode: u8) -> bool {
    matches!(opcode, 0x36..=0x4f)
}

/// `ldc`.
const LDC: u8 = 0x12;
/// `ldc_w`.
const LDC_W: u8 = 0x13;
/// `getstatic`.
const GETSTATIC: u8 = 0xb2;
/// `invokestatic`.
const INVOKESTATIC: u8 = 0xb8;

/// Runs one X3 scan.
///
/// A request whose environment the validator rejected performs nothing: it demands no class, reads
/// no body and publishes the honest unavailable state with the environment's own problems, exactly
/// like every other report of this engine.
pub(crate) fn reflection_patterns(
    content: &[ArtifactSnapshot],
    request: &ReflectionPatternRequest,
    budget: &mut Budget,
) -> Result<ReflectionPatternReport> {
    let (problems, environment_identity) = validate_environment(content, &request.environment);
    let mut diagnostics = environment_diagnostics(&problems);
    if !problems.is_empty() {
        diagnostics.push(unavailable_diagnostic(
            RESOLUTION_NOT_IMPLEMENTED,
            "bounded reflection pattern inference",
        ));
        return Ok(ReflectionPatternReport {
            environment_identity,
            environment_problems: problems,
            scope: request.scope.clone(),
            analysis: ResolutionAnalysis::NotPerformed,
            sites: Vec::new(),
            unresolved_dependencies: Vec::new(),
            has_more: false,
            returned_items: 0,
            reads: Vec::new(),
            coverage: Coverage::not_requested(),
            execution: ExecutionReport::Failed {
                reason: TerminationReason::Unsupported {
                    code: RESOLUTION_NOT_IMPLEMENTED.to_string(),
                },
                usage: budget.usage(),
            },
            diagnostics,
        });
    }

    let Some(snapshot) = content
        .iter()
        .find(|candidate| candidate.id() == &request.environment.runtime.physical.snapshot)
    else {
        // Unreachable from the facade, which refuses a snapshot the content does not provide; a
        // scan that cannot see its own range claims nothing.
        return Err(Error::invalid_input(
            "resolution_snapshot_mismatch",
            "the pattern scan names a snapshot the request content does not provide",
        ));
    };

    let mut scan = Scan {
        content,
        request,
        budget,
        environment_identity,
        environment_problems: problems,
        closure: HeaderClosure::new(content, &request.environment),
        sites: Vec::new(),
        unresolved: Vec::new(),
        diagnostics,
        run_reads: Vec::new(),
        stop: None,
        truncated: false,
        examined: 0,
        total: 0,
    };
    scan.run(snapshot);
    Ok(scan.finish())
}

/// The state of one running scan: what it published, what it read and what stopped it.
struct Scan<'a> {
    content: &'a [ArtifactSnapshot],
    request: &'a ReflectionPatternRequest,
    budget: &'a mut Budget,
    environment_identity: EnvironmentIdentity,
    environment_problems: Vec<EnvironmentProblem>,
    closure: HeaderClosure<'a>,
    sites: Vec<ReflectionSite>,
    unresolved: Vec<UnresolvedDependency>,
    diagnostics: Vec<Diagnostic>,
    run_reads: Vec<HeaderRead>,
    /// The stop that ended the scan: the execution the report publishes and its explanation.
    stop: Option<ScanStop>,
    /// Whether the answer is a prefix for a reason other than a stop: range this scan could not
    /// read, a body it could not analyse, or the caller's item limit.
    truncated: bool,
    /// Classes of the scope this scan really demanded and read a header for.
    examined: u64,
    /// Names the scope lists.
    total: u64,
}

impl Scan<'_> {
    /// Walks the scope's class names in listing order.
    fn run(&mut self, snapshot: &ArtifactSnapshot) {
        let range = match enumerate_range(snapshot, &self.request.scope, self.budget) {
            Ok(range) => range,
            Err(error) => {
                self.stop_now(&error);
                return;
            }
        };
        // The listing's own truncation is the first stop this scan knows: a later refusal is a
        // consequence of the same exhausted budget and must not replace it.
        if let Some(truncation) = &range.truncation {
            self.stop = Some(ScanStop {
                execution: truncation.clone(),
                diagnostic: truncation_diagnostic(truncation),
            });
        }
        self.total = u64::try_from(range.names.len()).unwrap_or(u64::MAX);
        for name in range.names.clone() {
            if self.stop.is_some() {
                return;
            }
            self.one_class(&name);
        }
    }

    /// Scans one name of the range: its own position, then the bodies of its members.
    fn one_class(&mut self, name: &JvmBytes) {
        let answer =
            self.closure
                .demand_from_caller(&name.0, HeaderDemand::PatternScan, self.budget);
        let handle = match answer.decision {
            Ok(handle) => handle,
            Err(error) => {
                self.stop_now(&error);
                return;
            }
        };
        let loader = self.closure.caller_loader().clone();
        let resolution = self.closure.resolution(handle);
        if let Some(gap) = unread_gap(resolution.lookup.state) {
            // No position of the order provides this name: the scope holds a class this scan could
            // not read, so its name is published instead of being read as an absence.
            self.unresolved.push(UnresolvedDependency {
                name: name.clone(),
                loader,
                reason: ReadReason::PatternScan,
                declared_by: None,
                gap,
            });
            self.truncated = true;
            return;
        }
        let location = resolution
            .lookup
            .location
            .as_ref()
            .expect("a found lookup publishes its position")
            .clone();
        let header = resolution
            .lookup
            .header
            .as_ref()
            .expect("a found lookup publishes its header");
        let facts = header.facts.clone();
        if !covered_by_scope(
            &self.request.environment.runtime.physical.snapshot,
            &self.request.scope,
            &location.definition,
        ) {
            // The order selected a definition outside the requested scope: the scope's own position
            // is undecided for this request, never an exclusion.
            self.truncated = true;
            return;
        }
        self.examined += 1;
        if !names_a_pattern(&facts.constant_pool) {
            // The class's own constant pool names no registered overload, so no body of it can hold
            // a site of this plane and none is read.
            return;
        }
        if location.loader != *self.closure.caller_loader() {
            // The class is provided by another loader than the request's own caller domain, and a
            // body analysis of this engine always runs under the caller's loader: nothing is
            // claimed for the bodies of this class, and the scan says which class and why.
            let name_bytes = facts.this_class.raw().0.clone();
            let message = format!(
                "the class `{}` is provided by the loader `{}` and not by the request's caller \
                 loader `{}`: a body analysis runs under the caller's own loader, so no body of \
                 this class was read and no site of it is claimed",
                String::from_utf8_lossy(&name_bytes),
                location.loader.0,
                self.closure.caller_loader().0
            );
            self.domain_diagnostic(PATTERN_BODY_NOT_ANALYSABLE, message);
            self.truncated = true;
            return;
        }
        // The class's own bytes are read once. A `getstatic` constant is proved from the field's
        // `ConstantValue` attribute, and that layer reads the attribute content out of the bytes
        // of this very read: the header facts were already paid for by the demand above, so this
        // charges the class bytes and their attributes and nothing else.
        let read = match self.closure.read_own_definition(
            &location.loader,
            &location.definition,
            HeaderDemand::PatternScan,
            self.budget,
        ) {
            Ok(read) => read,
            Err(error) => {
                self.stop_now(&error);
                return;
            }
        };
        let constants = match own_string_constants(&facts, read.read.bytes(), self.budget) {
            Ok(constants) => constants,
            Err(error) => {
                self.stop_now(&error);
                return;
            }
        };
        self.bodies(&facts, &location.definition, &constants);
    }

    /// One method-analysis run per member with a body, and the sites of that run's own decode.
    fn bodies(
        &mut self,
        facts: &ClassFacts,
        definition: &PhysicalDefinitionId,
        constants: &[OwnConstant],
    ) {
        let members: Vec<MemberHeader> = facts
            .methods
            .iter()
            .filter(|member| crate::engine::has_code_attribute(member))
            .cloned()
            .collect();
        for member in members {
            if self.stop.is_some() {
                return;
            }
            let identity = PhysicalMethodId {
                owner: definition.clone(),
                name: member.name.raw().clone(),
                descriptor: member.descriptor.raw().clone(),
            };
            let request = MethodAnalysisRequest {
                environment: self.request.environment.clone(),
                method: identity.clone(),
                stages: AnalysisStage::ALL.to_vec(),
            };
            let analyzed =
                match crate::engine::analyze_method_ir(self.content, &request, self.budget) {
                    Ok(analyzed) => analyzed,
                    Err(error) => {
                        self.stop_now(&error);
                        return;
                    }
                };
            // The run's own reads are this scan's cost and this report's evidence: they are the
            // bodies it paid for, one request-scoped closure per run.
            self.run_reads
                .extend(analyzed.report().reads.iter().cloned());
            let Some(code) = analyzed.ir().code() else {
                // The read produced no decode: the call sites of this body were never seen, so
                // they are unread range and no site of it is guessed at.
                let message = format!(
                    "the body of `{}` `{}` was not decoded by its own analysis run, so the call \
                     sites it holds are unread and no site of it is claimed",
                    String::from_utf8_lossy(&identity.name.0),
                    String::from_utf8_lossy(&identity.descriptor.0)
                );
                self.domain_diagnostic(PATTERN_BODY_NOT_ANALYSABLE, message);
                self.truncated = true;
                continue;
            };
            let sites = pattern_sites(code, analyzed.ir().constant_pool());
            if sites.is_empty() {
                continue;
            }
            match stop_code(analyzed.report().execution.clone()) {
                Some(code_of_stop) => {
                    // The body decoded, so its call sites are known, and the run that would have
                    // named their values stopped: every site is Unknown with the run's own code.
                    let message = format!(
                        "the analysis of the body of `{}` `{}` stopped under `{code_of_stop}`: the \
                         call sites of this body are published as Unknown",
                        String::from_utf8_lossy(&identity.name.0),
                        String::from_utf8_lossy(&identity.descriptor.0)
                    );
                    self.domain_diagnostic(&code_of_stop, message);
                    // The values of this body were never named, so the answer over it is a prefix.
                    self.truncated = true;
                    for site in sites {
                        if self.stop.is_some() {
                            return;
                        }
                        if self.limit_reached() {
                            self.truncated = true;
                            return;
                        }
                        let published = ReflectionSite {
                            definition: definition.clone(),
                            method: identity.clone(),
                            bci: site.bci,
                            overload: site.overload.clone(),
                            rule: site.rule.id,
                            state: ReflectionSiteState::Unknown {
                                reason: PatternUnknown::AnalysisStopped {
                                    code: code_of_stop.clone(),
                                },
                            },
                        };
                        if !self.publish(published) {
                            return;
                        }
                    }
                }
                None => {
                    let Some(ssa) = analyzed.ir().ssa() else {
                        let message = format!(
                            "the body of `{}` `{}` published no value table: its call sites are \
                             published as Unknown",
                            String::from_utf8_lossy(&identity.name.0),
                            String::from_utf8_lossy(&identity.descriptor.0)
                        );
                        self.domain_diagnostic("ir_ssa_not_published", message);
                        self.truncated = true;
                        for site in sites {
                            if self.stop.is_some() {
                                return;
                            }
                            if self.limit_reached() {
                                self.truncated = true;
                                return;
                            }
                            let published = ReflectionSite {
                                definition: definition.clone(),
                                method: identity.clone(),
                                bci: site.bci,
                                overload: site.overload.clone(),
                                rule: site.rule.id,
                                state: ReflectionSiteState::Unknown {
                                    reason: PatternUnknown::AnalysisStopped {
                                        code: "ir_ssa_not_published".to_string(),
                                    },
                                },
                            };
                            if !self.publish(published) {
                                return;
                            }
                        }
                        continue;
                    };
                    for site in sites {
                        if self.stop.is_some() {
                            return;
                        }
                        if self.limit_reached() {
                            self.truncated = true;
                            return;
                        }
                        let body = Body {
                            code,
                            pool: analyzed.ir().constant_pool(),
                            ssa,
                            facts,
                            constants,
                        };
                        let Some(state) = self.site_state(&site, &body) else {
                            // The scan stopped while deciding this site: a site behind a stop is
                            // neither published nor counted.
                            return;
                        };
                        let published = ReflectionSite {
                            definition: definition.clone(),
                            method: identity.clone(),
                            bci: site.bci,
                            overload: site.overload.clone(),
                            rule: site.rule.id,
                            state,
                        };
                        if !self.publish(published) {
                            return;
                        }
                    }
                }
            }
        }
    }

    /// The state of one call site, or `None` when deciding it stopped the scan.
    fn site_state(&mut self, site: &CallSite, body: &Body<'_>) -> Option<ReflectionSiteState> {
        let PatternSupport::Supported = site.rule.support else {
            let PatternSupport::Unsupported { reason } = site.rule.support else {
                unreachable!("the two states above are exhaustive");
            };
            return Some(ReflectionSiteState::Unknown {
                reason: PatternUnknown::PatternNotSupported {
                    rule: site.rule.id,
                    reason,
                },
            });
        };
        let name = match read_input(site, body, site.rule.inputs.name, PatternInput::TargetName) {
            Decided::Proven(proven) => proven,
            Decided::Unknown(reason) => return Some(ReflectionSiteState::Unknown { reason }),
        };
        let owner = match site.rule.inputs.owner {
            None => None,
            Some(place) => match read_input(site, body, place, PatternInput::OwnerClass) {
                Decided::Proven(proven) => Some(proven),
                Decided::Unknown(reason) => {
                    return Some(ReflectionSiteState::Unknown { reason });
                }
            },
        };
        let target = match site.rule.target {
            ReflectedTargetKind::TypeName => ReflectedTarget::Type {
                name: name.value.clone(),
            },
            ReflectedTargetKind::ServiceInterface => ReflectedTarget::ServiceInterface {
                name: name.value.clone(),
            },
            ReflectedTargetKind::Member { member } => ReflectedTarget::Member {
                owner: owner
                    .as_ref()
                    .expect("a member rule names the class it looks the member up on")
                    .value
                    .clone(),
                name: name.value.clone(),
                member,
            },
        };
        let class_name = body.facts.this_class.raw().0.clone();
        let resolution = self.target_state(site.rule, &name.value, &class_name)?;
        // The proof ends at the call site itself, which is the last step it walked.
        let mut path = name.path;
        path.push(site.bci);
        let transfers = u32::try_from(path.len().saturating_sub(2)).unwrap_or(u32::MAX);
        let propagation = Propagation {
            source: name.source,
            transfers,
            path,
            budget_dimension: FLOW_BOUND,
        };
        let constant_input = name.value.clone();
        Some(ReflectionSiteState::PatternInferredTarget(Box::new(
            PatternInference {
                rule: site.rule.id,
                rule_version: site.rule.rule,
                constant_input,
                target,
                propagation,
                loader: loader_assumption(site.rule.loader, &self.request.environment),
                resolution,
            },
        )))
    }

    /// What the snapshot's own order states about one inferred name.
    ///
    /// The demand happens only where the rule *states* an order this request can search: the
    /// caller's own loader. A loader the snapshot does not identify (a thread context loader, a
    /// loader argument), a member target (whose identity the descriptor would decide) and a
    /// constant that is not a binary name are all `NotDemanded` with the reason — the one answer
    /// that is not available is a search order nobody named.
    fn target_state(
        &mut self,
        rule: &'static PatternRule,
        name: &JvmBytes,
        class_name: &[u8],
    ) -> Option<PatternTargetState> {
        match (rule.target, rule.loader) {
            (ReflectedTargetKind::Member { .. }, _) => {
                return Some(PatternTargetState::NotDemanded {
                    reason: "the member is named and not identified: the parameter types (or the \
                             field type) are not propagated, so no member is looked up and no \
                             declaration is claimed",
                });
            }
            (_, LoaderAssumptionKind::ThreadContextLoader) => {
                return Some(PatternTargetState::NotDemanded {
                    reason: LoaderAssumptionKind::ThreadContextLoader.statement(),
                });
            }
            (_, LoaderAssumptionKind::ExplicitLoaderArgument) => {
                return Some(PatternTargetState::NotDemanded {
                    reason: LoaderAssumptionKind::ExplicitLoaderArgument.statement(),
                });
            }
            (_, LoaderAssumptionKind::CallerDefiningLoader) => {}
        }
        if !is_binary_name(&name.0) {
            return Some(PatternTargetState::NotDemanded {
                reason: "the constant is not a binary name in the sense of JVMS 4.2.1, so no order \
                         is searched for it; its bytes are published as the class file holds them",
            });
        }
        let answer =
            self.closure
                .demand_from_caller(&name.0, HeaderDemand::PatternTarget, self.budget);
        let handle = match answer.decision {
            Ok(handle) => handle,
            Err(error) => {
                self.stop_now(&error);
                return None;
            }
        };
        let loader = self.closure.caller_loader().clone();
        let resolution = self.closure.resolution(handle);
        let state = resolution.lookup.state;
        let location = resolution.lookup.location.clone();
        let declared_by = Some(JvmBytes(class_name.to_vec()));
        match state {
            HeaderLookupState::Found => {
                let location = location.expect("a found lookup publishes its position");
                Some(PatternTargetState::Resolved {
                    loader: location.loader,
                    definition: location.definition,
                })
            }
            HeaderLookupState::Missing => {
                self.unresolved.push(UnresolvedDependency {
                    name: name.clone(),
                    loader: loader.clone(),
                    reason: ReadReason::PatternTarget,
                    declared_by,
                    gap: DependencyGap::Missing,
                });
                Some(PatternTargetState::NotInSnapshot { loader })
            }
            HeaderLookupState::Ambiguous => {
                self.unresolved.push(UnresolvedDependency {
                    name: name.clone(),
                    loader: loader.clone(),
                    reason: ReadReason::PatternTarget,
                    declared_by,
                    gap: DependencyGap::Ambiguous,
                });
                Some(PatternTargetState::Ambiguous { loader })
            }
        }
    }

    /// Whether the caller's item limit stops the scan.
    fn limit_reached(&self) -> bool {
        self.request.max_items != 0
            && u64::try_from(self.sites.len()).unwrap_or(u64::MAX) >= self.request.max_items
    }

    /// Publishes one site, charging one `ResultItems` before it enters the report.
    fn publish(&mut self, site: ReflectionSite) -> bool {
        match self.budget.charge(CountedBudgetDimension::ResultItems, 1) {
            Ok(()) => {
                self.sites.push(site);
                true
            }
            Err(error) => {
                // A refused charge keeps the sites already published and ends the scan: the site
                // that could not be paid for is neither published nor counted.
                self.stop_now(&error);
                false
            }
        }
    }

    /// Publishes one domain diagnostic, charged like every result of this engine.
    fn domain_diagnostic(&mut self, code: &str, message: String) {
        let diagnostic = Diagnostic {
            code: code.to_string(),
            severity: DiagnosticSeverity::Warning,
            message,
            provenance: None,
        };
        match self.budget.charge(CountedBudgetDimension::ResultItems, 1) {
            Ok(()) => self.diagnostics.push(diagnostic),
            Err(error) => self.stop_now(&error),
        }
    }

    /// Ends the scan under one refusal: the earliest stop governs, and the prefix stays.
    fn stop_now(&mut self, error: &Error) {
        let stop = refused(error, self.budget.usage());
        if self.stop.is_none() {
            self.stop = Some(stop);
        }
        self.truncated = true;
    }

    /// Builds the report of this scan.
    fn finish(mut self) -> ReflectionPatternReport {
        let complete = self.stop.is_none() && !self.truncated && self.examined == self.total;
        // The stop that ended the scan is explained once, after the results it kept, and it is
        // control metadata: like the environment plane, it is not charged.
        if let Some(stop) = &self.stop {
            self.diagnostics.push(stop.diagnostic.clone());
        }
        let mut reads = published_reads(&self.closure);
        reads.extend(self.run_reads.iter().cloned());
        let extent = self.closure.searched_extent();
        // The closure's class-name searches become the runtime-resolution plane through the one
        // projection every report of this engine reads them with, and this plane's own artifact
        // dimension is the class names of the scope it examined. `dynamic_analysis` stays
        // `not_requested`: nothing here executes, loads or observes a runtime.
        let coverage = search_coverage_with_artifact(
            extent.examined,
            extent.positions,
            self.stop.is_none(),
            complete,
            artifact_coverage(self.examined, self.total, complete),
        );
        let execution = with_usage(
            self.stop.as_ref().map_or(
                ExecutionReport::Complete {
                    usage: self.budget.usage(),
                },
                |stop| stop.execution.clone(),
            ),
            self.budget.usage(),
        );
        let returned_items = u64::try_from(self.sites.len()).unwrap_or(u64::MAX);
        ReflectionPatternReport {
            environment_identity: self.environment_identity,
            environment_problems: self.environment_problems,
            scope: self.request.scope.clone(),
            analysis: ResolutionAnalysis::Performed,
            sites: self.sites,
            unresolved_dependencies: self.unresolved,
            has_more: !complete,
            returned_items,
            reads,
            coverage,
            execution,
            diagnostics: self.diagnostics,
        }
    }
}

/// The stop that ended a scan: the execution the report publishes and the diagnostic that explains
/// it.
struct ScanStop {
    execution: ExecutionReport,
    diagnostic: Diagnostic,
}

/// The same refusal mapping every report of this engine uses: a budget stop is a partial execution
/// that names its dimension, a cancellation is a cancellation, and a structural failure keeps the
/// reader's or the listing's own code.
fn refused(error: &Error, usage: jarde_reader::budget::UsageSnapshot) -> ScanStop {
    let (execution, diagnostic) = crate::ir::terminal(error, usage);
    ScanStop {
        execution,
        diagnostic,
    }
}

/// The explanation of a listing that stopped: the listing's own execution, named as what it is.
fn truncation_diagnostic(execution: &ExecutionReport) -> Diagnostic {
    Diagnostic {
        code: stop_code(execution.clone())
            .unwrap_or_else(|| "resolution_pattern_listing".to_string()),
        severity: DiagnosticSeverity::Warning,
        message: "the class listing of this scope stopped before its end: the names it did list \
                  were scanned, and the range behind the listed prefix was never read"
            .to_string(),
        provenance: None,
    }
}

/// The artifact-structural plane of a scan: the class names of the scope it examined.
fn artifact_coverage(examined: u64, total: u64, complete: bool) -> CoverageDimension {
    let mut scanned = Vec::new();
    let mut skipped = Vec::new();
    if examined > 0 {
        scanned.push(CoverageRange {
            label: CLASS_RANGE_LABEL.to_string(),
            start: 0,
            end: examined,
        });
    }
    if examined < total {
        skipped.push(CoverageRange {
            label: CLASS_RANGE_LABEL.to_string(),
            start: examined,
            end: total,
        });
    }
    CoverageDimension {
        state: if complete {
            CoverageState::CompleteWithinSchema
        } else {
            CoverageState::Partial
        },
        scanned,
        skipped,
        uninterpreted_extensions: Vec::new(),
    }
}

/// One `static final String` constant one scanned class declares.
///
/// The value is the class file's own `ConstantValue`, decoded from the field's attributes by the
/// reader's attribute layer over the bytes of that same class read; the assumption the report
/// states beside it (a static field is process state) is what keeps it from being read as a
/// literal of the method.
struct OwnConstant {
    name: JvmBytes,
    descriptor: JvmBytes,
    value: JvmBytes,
}

/// The `String` constants one class declares, read from its fields' `ConstantValue` attributes.
///
/// Only a field the class itself declares, whose descriptor is `Ljava/lang/String;` and whose
/// shell list holds the attribute, is read: a field without one is initialized by code this plane
/// does not run, and another class's field would need that class's bytes.
fn own_string_constants(
    facts: &ClassFacts,
    bytes: &[u8],
    budget: &mut Budget,
) -> Result<Vec<OwnConstant>> {
    let mut constants = Vec::new();
    for field in &facts.fields {
        if field.descriptor.raw().0 != b"Ljava/lang/String;" {
            continue;
        }
        if !field
            .attributes
            .iter()
            .any(|shell| shell.name.raw().0 == b"ConstantValue")
        {
            continue;
        }
        let attributes = attribute_facts(bytes, &field.attributes, &facts.constant_pool, budget)?;
        let Some(index) = attributes.constant_value else {
            continue;
        };
        let Ok(entry) = cp_entry(&facts.constant_pool, index.0) else {
            continue;
        };
        let CpEntryKind::String { value, .. } = &entry.kind else {
            continue;
        };
        constants.push(OwnConstant {
            name: field.name.raw().clone(),
            descriptor: field.descriptor.raw().clone(),
            value: value.clone(),
        });
    }
    Ok(constants)
}

/// Whether one lookup state states a gap for the name that was demanded.
fn unread_gap(state: HeaderLookupState) -> Option<DependencyGap> {
    match state {
        HeaderLookupState::Found => None,
        HeaderLookupState::Missing => Some(DependencyGap::Missing),
        HeaderLookupState::Ambiguous => Some(DependencyGap::Ambiguous),
    }
}

/// Whether one class's constant pool names any registered overload.
///
/// A class that names none can hold no site of this plane, so its bodies are not read at all: the
/// pre-filter is one pass over facts this scan already holds.
fn names_a_pattern(pool: &[CpEntryFacts]) -> bool {
    pool.iter().any(|entry| match &entry.kind {
        CpEntryKind::MethodRef {
            owner,
            name,
            descriptor,
            ..
        }
        | CpEntryKind::InterfaceMethodRef {
            owner,
            name,
            descriptor,
            ..
        } => pattern_for(&owner.0, &name.0, &descriptor.0).is_some(),
        _ => false,
    })
}

/// The code one execution stopped under, if it stopped.
fn stop_code(execution: ExecutionReport) -> Option<String> {
    match execution {
        ExecutionReport::Complete { .. } => None,
        ExecutionReport::Cancelled { .. } => Some("resolution_pattern_cancelled".to_string()),
        ExecutionReport::Partial { reason, .. } | ExecutionReport::Failed { reason, .. } => {
            Some(match reason {
                TerminationReason::Error { code } | TerminationReason::Unsupported { code } => code,
                TerminationReason::BudgetExceeded { dimension } => {
                    format!("budget_exceeded_{}", budget_dimension_code(dimension))
                }
            })
        }
    }
}

/// Whether one byte string is a binary class name in the sense of JVMS 4.2.1.
///
/// A binary name is a non-empty sequence of non-empty identifiers separated by `/`, and no
/// identifier contains `.`, `;` or `[`. The check states what this slice will search an order for: a
/// constant that is not a binary name is published as the call site's input and *no* order is
/// searched for it, instead of turning a malformed string into a "missing class".
fn is_binary_name(name: &[u8]) -> bool {
    !name.is_empty()
        && name.split(|byte| *byte == b'/').all(|identifier| {
            !identifier.is_empty()
                && !identifier
                    .iter()
                    .any(|byte| matches!(byte, b'.' | b';' | b'['))
        })
}

/// The assumption text of one loader kind, with the loader itself when the assumption is the
/// request's own caller domain.
fn loader_assumption(
    kind: LoaderAssumptionKind,
    environment: &ResolutionEnvironment,
) -> LoaderAssumption {
    LoaderAssumption {
        kind,
        loader: matches!(kind, LoaderAssumptionKind::CallerDefiningLoader)
            .then(|| environment.runtime.load_domain.loader.clone()),
        statement: kind.statement(),
    }
}

/// One call site that answered a registered rule.
struct CallSite {
    bci: u32,
    opcode: u8,
    rule: &'static PatternRule,
    overload: Overload,
}

/// The sites one decode holds, so one rule decides them all.
///
/// The decode is the analysed run's own (`MethodCodeFacts` and the pool of the same header read),
/// and an overload this registry does not hold produces no site at all: the plane makes no claim
/// about it.
fn pattern_sites(code: &MethodCodeFacts, pool: &[CpEntryFacts]) -> Vec<CallSite> {
    let mut sites = Vec::new();
    for (position, instruction) in code.instructions.iter().enumerate() {
        let Some(operands) = code.operands().get(position) else {
            continue;
        };
        let opcode = operands.effective_opcode;
        if !matches!(opcode, 0xb6..=0xb9) {
            continue;
        }
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = cp_entry(pool, index) else {
            continue;
        };
        let (owner, name, descriptor) = match &entry.kind {
            CpEntryKind::MethodRef {
                owner,
                name,
                descriptor,
                ..
            }
            | CpEntryKind::InterfaceMethodRef {
                owner,
                name,
                descriptor,
                ..
            } => (owner, name, descriptor),
            _ => continue,
        };
        let Some(rule) = pattern_for(&owner.0, &name.0, &descriptor.0) else {
            continue;
        };
        sites.push(CallSite {
            bci: instruction.bci,
            opcode,
            rule,
            overload: Overload {
                owner: owner.clone(),
                name: name.clone(),
                descriptor: descriptor.clone(),
            },
        });
    }
    sites
}

/// One body a site was read from: the run's own decode, the pool of the same class read, the value
/// table that run published, the class's own facts and the constants that class declares.
///
/// The parts travel together because they are one read of one body: an instruction's constant-pool
/// index belongs to the pool the decode came with, and a value belongs to the table of the same
/// run. Nothing here is read a second time by the classifier.
struct Body<'a> {
    code: &'a MethodCodeFacts,
    pool: &'a [CpEntryFacts],
    ssa: &'a SsaTable,
    facts: &'a ClassFacts,
    constants: &'a [OwnConstant],
}

/// One input of one call site this slice really proved.
struct Proven {
    value: JvmBytes,
    source: ConstantSource,
    /// The instructions the proof followed, from the use back to the source.
    path: Vec<u32>,
}

/// What one input of one call site turned out to be.
enum Decided {
    /// A constant this slice proved.
    Proven(Proven),
    /// Nothing is claimed about it, and why.
    Unknown(PatternUnknown),
}

/// Reads one input of one call site against the constant kind its rule requires.
fn read_input(site: &CallSite, body: &Body<'_>, place: InputPlace, input: PatternInput) -> Decided {
    let wanted = wanted_constant(site.rule, input);
    let value = match input_value(site, body.ssa, place) {
        Ok(value) => value,
        Err(origin) => {
            return Decided::Unknown(PatternUnknown::DynamicInput { input, origin });
        }
    };
    match classify(value, body, wanted) {
        InputValue::Dynamic(origin) => {
            Decided::Unknown(PatternUnknown::DynamicInput { input, origin })
        }
        InputValue::Constant {
            value,
            source,
            path,
        } => Decided::Proven(Proven {
            value,
            source,
            path,
        }),
    }
}

/// The constant one input of one rule has to be.
const fn wanted_constant(rule: &PatternRule, input: PatternInput) -> WantedConstant {
    match (rule.target, input) {
        (ReflectedTargetKind::ServiceInterface, _) => WantedConstant::Class,
        (_, PatternInput::TargetName) => WantedConstant::String,
        (_, PatternInput::OwnerClass) => WantedConstant::Class,
    }
}

/// The constant kind one rule requires.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WantedConstant {
    /// A `String` constant: a name.
    String,
    /// A `Class` literal: a class the site names directly.
    Class,
}

/// The SSA instruction of one body at one bytecode index, if the table names it.
///
/// The decode and the value table are two reads of one body: the decode states what the bytes are,
/// the table states which values the instruction read and wrote. Every classification this slice
/// makes asks the table through this one lookup.
fn ssa_instruction(ssa: &SsaTable, bci: u32) -> Option<&SsaInstruction> {
    ssa.blocks()
        .iter()
        .flat_map(|block| block.instructions().iter())
        .find(|instruction| instruction.bci() == bci)
}

/// The value one input of one call site reads, as the SSA table of its body names it.
///
/// The operand stack holds the receiver (for an instance call) and then the arguments left to
/// right, and the SSA table records one read per popped slot, so sorting the reads of this
/// instruction by slot gives exactly that layout. A place the table does not record is
/// [`DynamicOrigin::NoDefinition`]: the shape this slice assumes is not there, and it says so.
fn input_value(
    site: &CallSite,
    ssa: &SsaTable,
    place: InputPlace,
) -> std::result::Result<ValueId, DynamicOrigin> {
    let instruction = ssa_instruction(ssa, site.bci).ok_or(DynamicOrigin::NoDefinition)?;
    let mut stack: Vec<(u32, ValueId)> = instruction
        .reads()
        .iter()
        .filter_map(|(slot, value)| match slot {
            Slot::Stack(depth) => Some((*depth, *value)),
            Slot::Local(_) => None,
        })
        .collect();
    stack.sort_unstable_by_key(|(depth, _)| *depth);
    let index = match place {
        InputPlace::Receiver => {
            if site.opcode == INVOKESTATIC {
                return Err(DynamicOrigin::NoDefinition);
            }
            0
        }
        InputPlace::Argument(position) => {
            let receiver = usize::from(site.opcode != INVOKESTATIC);
            receiver + usize::from(position)
        }
    };
    stack
        .get(index)
        .map(|(_, value)| *value)
        .ok_or(DynamicOrigin::NoDefinition)
}

/// What one SSA value turned out to be.
enum InputValue {
    /// A constant this slice proved, with the path the proof walked from it to the use.
    Constant {
        value: JvmBytes,
        source: ConstantSource,
        path: Vec<u32>,
    },
    /// Nothing is claimed about it, and why.
    Dynamic(DynamicOrigin),
}

/// Classifies one SSA value against the constant kind a rule requires, following the bounded alias
/// chain the body's own value flow states.
///
/// The classification reads four things and nothing else: the values of this body's own flow (each
/// definition, chased through the bounded replacement chain SSA records for a folded phi and
/// through the local loads that carry one value on to its next reader), the decode of this body,
/// the call site's own class facts and the constants that class declares. A `getstatic` of a field
/// *another* class declares, and one of a field with no `ConstantValue`, are deliberately not proven
/// sources: the first would need that class's bytes, and the second is initialized by code this
/// plane does not run.
fn classify(value: ValueId, body: &Body<'_>, wanted: WantedConstant) -> InputValue {
    let Body {
        code,
        pool,
        ssa,
        facts,
        constants,
    } = *body;
    // The path is collected from the use back to the source and reversed at the end, so the report
    // publishes it in the order the value travelled.
    let mut path: Vec<u32> = Vec::new();
    let mut current = value;
    let mut transfers = 0_usize;
    loop {
        for _ in 0..REPLACEMENT_CHASE_BOUND {
            match ssa.value(current).replaced_by() {
                Some(next) => current = next,
                None => break,
            }
        }
        let defined = ssa.value(current);
        if defined.replaced_by().is_some() {
            // The chain is longer than this slice's bound: the value stands in a shape it does not
            // follow, and nothing is claimed about it.
            return InputValue::Dynamic(DynamicOrigin::ChainBeyondBound);
        }
        let Definition::Instruction { bci, .. } = defined.def() else {
            return InputValue::Dynamic(match defined.def() {
                Definition::Entry { .. } => DynamicOrigin::Entry,
                Definition::Phi { .. } => DynamicOrigin::Merge,
                Definition::Caught { .. } => DynamicOrigin::Caught,
                Definition::Instruction { .. } => DynamicOrigin::NoDefinition,
            });
        };
        let bci = *bci;
        let Some(position) = code
            .instructions
            .iter()
            .position(|instruction| instruction.bci == bci)
        else {
            return InputValue::Dynamic(DynamicOrigin::NoDefinition);
        };
        let instruction = &code.instructions[position];
        let opcode = code
            .operands()
            .get(position)
            .map_or(instruction.opcode, |operands| operands.effective_opcode);
        path.push(bci);
        match opcode {
            LDC | LDC_W => {
                let Some(index) = instruction.constant_pool_index else {
                    return InputValue::Dynamic(DynamicOrigin::Instruction { bci, opcode });
                };
                let Ok(entry) = cp_entry(pool, index) else {
                    return InputValue::Dynamic(DynamicOrigin::NoDefinition);
                };
                let proven = match (&entry.kind, wanted) {
                    (CpEntryKind::String { value, .. }, WantedConstant::String) => {
                        Some((value.clone(), ConstantSourceKind::LdcString))
                    }
                    (CpEntryKind::Class { name, .. }, WantedConstant::Class) => {
                        Some((name.clone(), ConstantSourceKind::LdcClass))
                    }
                    // A constant of a kind the registered overload's own parameter type does not
                    // state is not a proven input: this slice states Unknown rather than coercing.
                    _ => None,
                };
                let Some((value, kind)) = proven else {
                    return InputValue::Dynamic(DynamicOrigin::Instruction { bci, opcode });
                };
                path.reverse();
                return InputValue::Constant {
                    value,
                    source: ConstantSource {
                        bci,
                        kind,
                        assumption: None,
                    },
                    path,
                };
            }
            GETSTATIC => {
                if wanted != WantedConstant::String {
                    return InputValue::Dynamic(DynamicOrigin::Instruction { bci, opcode });
                }
                let Some(index) = instruction.constant_pool_index else {
                    return InputValue::Dynamic(DynamicOrigin::Instruction { bci, opcode });
                };
                let Ok(entry) = cp_entry(pool, index) else {
                    return InputValue::Dynamic(DynamicOrigin::NoDefinition);
                };
                let CpEntryKind::FieldRef {
                    owner,
                    name,
                    descriptor,
                    ..
                } = &entry.kind
                else {
                    return InputValue::Dynamic(DynamicOrigin::Instruction { bci, opcode });
                };
                // The field has to be the call site's own class's and has to hold a `ConstantValue`
                // that class's own attributes state: proving another class's field would read that
                // class, which is a bound this slice states instead of crossing silently, and a
                // field without a `ConstantValue` is initialized by code this plane does not run.
                if owner.0 != facts.this_class.raw().0 || descriptor.0 != b"Ljava/lang/String;" {
                    return InputValue::Dynamic(DynamicOrigin::Instruction { bci, opcode });
                }
                let Some(constant) = constants.iter().find(|constant| {
                    constant.name.0 == name.0 && constant.descriptor.0 == descriptor.0
                }) else {
                    return InputValue::Dynamic(DynamicOrigin::Instruction { bci, opcode });
                };
                path.reverse();
                return InputValue::Constant {
                    value: constant.value.clone(),
                    source: ConstantSource {
                        bci,
                        kind: ConstantSourceKind::StaticFieldConstantValue {
                            owner: owner.clone(),
                            name: name.clone(),
                            descriptor: descriptor.clone(),
                        },
                        assumption: Some(STATIC_FIELD_ASSUMPTION),
                    },
                    path,
                };
            }
            // A store writes the value it popped into a local, and a load reads that slot back: the
            // body's own value flow names the value on both sides, so the proof follows it, within
            // this slice's own bound.
            _ if is_local_load(opcode) && transfers < ALIAS_CHASE_BOUND => {
                let Some(handing_on) = ssa_instruction(ssa, bci) else {
                    return InputValue::Dynamic(DynamicOrigin::NoDefinition);
                };
                let Some(next) = handing_on
                    .reads()
                    .iter()
                    .find_map(|(slot, value)| match slot {
                        Slot::Local(_) => Some(*value),
                        Slot::Stack(_) => None,
                    })
                else {
                    return InputValue::Dynamic(DynamicOrigin::NoDefinition);
                };
                current = next;
                transfers += 1;
            }
            _ if is_local_store(opcode) && transfers < ALIAS_CHASE_BOUND => {
                let Some(handing_on) = ssa_instruction(ssa, bci) else {
                    return InputValue::Dynamic(DynamicOrigin::NoDefinition);
                };
                let Some(next) = handing_on
                    .reads()
                    .iter()
                    .find_map(|(slot, value)| match slot {
                        Slot::Stack(_) => Some(*value),
                        Slot::Local(_) => None,
                    })
                else {
                    return InputValue::Dynamic(DynamicOrigin::NoDefinition);
                };
                current = next;
                transfers += 1;
            }
            _ => return InputValue::Dynamic(DynamicOrigin::Instruction { bci, opcode }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The parameter type strings of one descriptor, in order.
    ///
    /// A test-side reader of the descriptor grammar, deliberately independent of the slice it
    /// checks: the implementation never parses a descriptor, it reads the operand stack, so this
    /// is what pins the assumption the implementation makes.
    fn parameters(descriptor: &str) -> Vec<&str> {
        let descriptor = descriptor.as_bytes();
        assert_eq!(
            descriptor.first(),
            Some(&b'('),
            "{} starts a method descriptor",
            String::from_utf8_lossy(descriptor)
        );
        let mut rest = &descriptor[1..];
        let mut parameters = Vec::new();
        while let Some((&first, tail)) = rest.split_first() {
            if first == b')' {
                break;
            }
            let (parameter, remainder) = match first {
                b'L' => {
                    let end = tail
                        .iter()
                        .position(|byte| *byte == b';')
                        .expect("a class type ends")
                        + 1;
                    (&rest[..=end], &rest[end + 1..])
                }
                b'[' => {
                    let end = rest
                        .iter()
                        .skip(1)
                        .position(|byte| *byte == b';')
                        .expect("an array type ends")
                        + 2;
                    (&rest[..end], &rest[end..])
                }
                _ => (&rest[..1], tail),
            };
            parameters.push(std::str::from_utf8(parameter).expect("fixture descriptors are ASCII"));
            rest = remainder;
        }
        parameters
    }

    /// How many slot units one parameter type occupies.
    fn slots(parameter: &str) -> usize {
        match parameter {
            "J" | "D" => 2,
            _ => 1,
        }
    }

    #[test]
    fn every_registered_rule_states_what_it_does_not_claim() {
        for rule in patterns() {
            assert!(
                !rule.not_claimed.is_empty(),
                "`{}` claims something without stating its boundary",
                rule.id
            );
            assert!(
                rule.not_claimed.iter().all(|note| !note.trim().is_empty()),
                "`{}` holds an empty boundary note",
                rule.id
            );
            assert!(
                !rule.id.is_empty()
                    && !rule.rule.name().is_empty()
                    && !rule.rule.version().is_empty(),
                "`{}` states an incomplete rule version",
                rule.id
            );
            assert!(
                !rule.source.trim().is_empty(),
                "`{}` cites no source",
                rule.id
            );
        }
    }

    #[test]
    fn two_overloads_are_two_rules_and_no_overload_is_registered_twice() {
        for (index, rule) in patterns().iter().enumerate() {
            for other in &patterns()[index + 1..] {
                assert_ne!(rule.id, other.id, "two rules share one id");
                assert!(
                    !(rule.owner == other.owner
                        && rule.name == other.name
                        && rule.descriptor == other.descriptor),
                    "`{}` and `{}` register the same overload",
                    rule.id,
                    other.id
                );
            }
        }
        let one = pattern_for(
            b"java/lang/Class",
            b"forName",
            b"(Ljava/lang/String;)Ljava/lang/Class;",
        )
        .expect("the one-argument overload is registered");
        let three = pattern_for(
            b"java/lang/Class",
            b"forName",
            b"(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
        )
        .expect("the three-argument overload is registered");
        assert_ne!(one.id, three.id);
        assert_eq!(one.loader, LoaderAssumptionKind::CallerDefiningLoader);
        assert_eq!(three.loader, LoaderAssumptionKind::ExplicitLoaderArgument);
    }

    #[test]
    fn every_registered_input_names_a_single_slot_parameter_of_its_own_descriptor() {
        for rule in patterns() {
            let parameters = parameters(rule.descriptor);
            for place in [Some(rule.inputs.name), rule.inputs.owner]
                .into_iter()
                .flatten()
            {
                match place {
                    InputPlace::Argument(position) => {
                        let index = usize::from(position);
                        assert!(
                            index < parameters.len(),
                            "`{}` names parameter {position} of `{}`",
                            rule.id,
                            rule.descriptor
                        );
                        let found = parameters[index];
                        assert!(
                            !found.starts_with('[') && slots(found) == 1,
                            "`{}` names `{found}` as an input, and this slice's operand-stack \
                             reading assumes one slot per input",
                            rule.id
                        );
                    }
                    InputPlace::Receiver => {}
                }
            }
            // The name of a target is always a constant one argument spells; a receiver is the
            // class a member is looked up on and never a name. The enumeration overloads this
            // slice derives nothing from carry a receiver where a name would be, which is exactly
            // why they are registered as unsupported.
            assert!(
                matches!(rule.inputs.name, InputPlace::Argument(_))
                    || matches!(rule.support, PatternSupport::Unsupported { .. }),
                "`{}` reads a target name off a receiver",
                rule.id
            );
            if let PatternSupport::Supported = rule.support {
                match rule.target {
                    ReflectedTargetKind::Member { .. } => assert!(
                        rule.inputs.owner.is_some(),
                        "`{}` infers a member without naming the class it is looked up on",
                        rule.id
                    ),
                    ReflectedTargetKind::TypeName | ReflectedTargetKind::ServiceInterface => {
                        assert!(
                            rule.inputs.owner.is_none(),
                            "`{}` names a class it is not looked up on",
                            rule.id
                        )
                    }
                }
            }
        }
    }

    #[test]
    fn the_required_families_are_registered_and_supported() {
        for (owner, name, descriptor, expected) in [
            (
                &b"java/lang/Class"[..],
                &b"forName"[..],
                &b"(Ljava/lang/String;)Ljava/lang/Class;"[..],
                "class-for-name",
            ),
            (
                &b"java/lang/Class"[..],
                &b"getMethod"[..],
                &b"(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;"[..],
                "class-get-method",
            ),
            (
                &b"java/lang/Class"[..],
                &b"getDeclaredField"[..],
                &b"(Ljava/lang/String;)Ljava/lang/reflect/Field;"[..],
                "class-get-declared-field",
            ),
            (
                &b"java/lang/invoke/MethodHandles$Lookup"[..],
                &b"findStatic"[..],
                &b"(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;"[..],
                "methodhandles-find-static",
            ),
            (
                &b"java/lang/invoke/MethodHandles$Lookup"[..],
                &b"findGetter"[..],
                &b"(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;"[..],
                "methodhandles-find-getter",
            ),
            (
                &b"java/util/ServiceLoader"[..],
                &b"load"[..],
                &b"(Ljava/lang/Class;)Ljava/util/ServiceLoader;"[..],
                "service-loader-load",
            ),
        ] {
            let rule = pattern_for(owner, name, descriptor)
                .unwrap_or_else(|| panic!("`{expected}` is registered"));
            assert_eq!(rule.id, expected);
            assert!(
                matches!(rule.support, PatternSupport::Supported),
                "`{expected}` infers a target in this slice"
            );
        }
    }

    #[test]
    fn an_unsupported_rule_states_a_reason_and_is_still_registered() {
        let mut unsupported = 0;
        for rule in patterns() {
            if let PatternSupport::Unsupported { reason } = rule.support {
                unsupported += 1;
                assert!(
                    !reason.trim().is_empty(),
                    "`{}` is unsupported without a reason",
                    rule.id
                );
                assert_eq!(
                    pattern_for(
                        rule.owner.as_bytes(),
                        rule.name.as_bytes(),
                        rule.descriptor.as_bytes()
                    )
                    .map(|found| found.id),
                    Some(rule.id),
                    "`{}` is registered as unsupported, so the registry holds it",
                    rule.id
                );
            }
        }
        assert!(
            unsupported > 0,
            "this slice derives no target from some overloads, and they are registered"
        );
    }

    #[test]
    fn an_overload_the_registry_does_not_hold_is_not_a_subject() {
        for (owner, name, descriptor) in [
            (
                &b"java/lang/Class"[..],
                &b"forName"[..],
                &b"(Ljava/lang/Class;)Ljava/lang/Class;"[..],
            ),
            (
                &b"java/lang/Class"[..],
                &b"getSuperclass"[..],
                &b"()Ljava/lang/Class;"[..],
            ),
            (
                &b"java/lang/invoke/MethodHandles"[..],
                &b"lookup"[..],
                &b"()Ljava/lang/invoke/MethodHandles$Lookup;"[..],
            ),
        ] {
            assert_eq!(
                pattern_for(owner, name, descriptor),
                None,
                "`{}` makes no claim about this overload",
                String::from_utf8_lossy(name)
            );
        }
    }

    #[test]
    fn a_binary_name_is_read_the_way_jvms_4_2_1_states_it() {
        assert!(is_binary_name(b"p/Target"));
        assert!(is_binary_name(b"java/util/ServiceLoader"));
        assert!(
            is_binary_name(b"p/*"),
            "a wildcard byte is an identifier byte"
        );
        assert!(!is_binary_name(b""));
        assert!(!is_binary_name(b"p/.Hidden"));
        assert!(!is_binary_name(b"[Lp/Target;"));
        assert!(!is_binary_name(b"p//Target"));
        assert!(!is_binary_name(b"p/;"));
    }
}
