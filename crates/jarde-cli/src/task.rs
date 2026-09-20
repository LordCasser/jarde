//! The task-oriented commands: friendly parameters over the library's own entries.
//!
//! Five commands, each a parameter layer over the library entry that owns the work — the two
//! evidence levels of the class listing, the member listing, the reference scan and its grouping,
//! the class view, and the task-oriented recovery. What this module owns is exactly three things:
//! the mapping from a friendly parameter to a library request, the two renderings of the one
//! report that came back, and the exit status that report's own planes state.
//!
//! The three are deliberately narrow:
//!
//! * **the library's document is the JSON document.** A command writes the report's own
//!   serialization — the bytes `serde_json` produces for the library value, envelope-free, with no
//!   renamed field and no plane of its own — so a caller can compare it field by field with what
//!   the library answers for the same request. The adapter adds nothing to a report, which is also
//!   why it reads nothing for one: every identity, count and diagnostic in the document is the
//!   library's.
//! * **the text rendering is a projection of that document.** Every line names the JSON field path
//!   it renders (`items.3.identity = {...}`), so "the text says what the JSON says" is a property
//!   of how it is written rather than a claim to be trusted. The one body a report carries — the
//!   recovered Java text — is written verbatim, because the field it comes from *is* the text.
//! * **diagnostics never travel with the body.** In text mode the report's content goes to standard
//!   output (or to `--output`) and its bookkeeping planes — `limits`, `usage`, `coverage`,
//!   `execution`, `diagnostics` — go to standard error; in JSON mode they are all fields of the one
//!   document. A code body is therefore never a place a diagnostic can end up in.
//!
//! The exit status is the report's own classification rather than a re-derivation of it: a report
//! whose execution planes are all `Complete` exits 0, an `Ambiguous` outcome (the library bound
//! several identities and ran nothing) exits 3, an `Incomplete` outcome (the name search did not
//! finish, so nothing was bound and nothing ran) exits 4 like any other stopped report, and a
//! `Performed` report that stopped — reliable prefix and all — exits 4 instead of pretending to be
//! a success. A parameter or request-level refusal exits 2, and a document that could not be
//! delivered exits 1.
//!
//! Where the chain hands an identity on, the option takes the *inner* identity type the library
//! publishes (`PhysicalDefinitionId`, `PhysicalMethodId`), so the value a listing printed is
//! consumed verbatim rather than reassembled from a name, an ordinal and a digest.

use crate::ErrorResponse;
use clap::{ArgAction, Args, Subcommand, ValueEnum};
use jarde::{
    ArtifactInput, ArtifactSnapshot, BodyRef, Budget, BudgetOverride, ClassNameQuery, ClassRef,
    ClassViewReport, ClassViewRequest, ConsumerKind, ConsumerSchema, CountedBudgetDimension,
    Engine, EnvironmentPolicy, EnvironmentRequest, Error, ExecutionReport, JvmBytes, LayoutMode,
    LoadRoot, LoaderId, MethodOperationRequest, MethodRecoveryReport, MethodRef,
    MultiReleasePolicy, OperationOutcome, PhysicalDefinitionId, PhysicalScope, PhysicalView,
    QueryRelation, QueryRequest, QueryTarget, ReferenceGrouping, ReferenceSource, RuntimeProfile,
    SymbolRef, UsageSnapshot, task_budget,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::slice;

/// The exit status of a command whose report executed completely.
pub const EXIT_COMPLETE: u8 = 0;
/// The exit status of a command whose document could not be delivered.
pub const EXIT_DELIVERY: u8 = 1;
/// The exit status of a usage or request-level input error.
pub const EXIT_USAGE: u8 = 2;
/// The exit status of a friendly name that bound several physical identities.
pub const EXIT_AMBIGUOUS: u8 = 3;
/// The exit status of a report whose execution stopped before its scope did — a performed report
/// that stopped, or a name selection the search could not finish.
pub const EXIT_INCOMPLETE: u8 = 4;

/// The JSON path of the one text body a task report carries: the recovery's own Java text.
///
/// It is a *field path*, not a second rendering: the text mode writes the value this path holds,
/// byte for byte, so `--format text --output A.java` writes exactly what the JSON document holds
/// under `recovered.recovery.text` — no header, no diagnostic and no added newline.
const RECOVERED_BODY: &[&str] = &["recovered", "recovery", "text"];

/// The consumer schema version a reference scan declares.
///
/// It is the version this engine serves: `jarde-query` keeps the served version to itself and
/// answers a request that names another with its own `query_consumer_schema_version` error, so a
/// build serving a different version refuses the request explicitly instead of scanning under a
/// schema the caller never named. The report echoes the schema it ran under either way.
const CONSUMER_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Subcommand)]
/// One task command.
///
/// The variants are the whole surface: `main` hands one of them to [`run`], and nothing else in
/// this crate reaches the library's task entries.
pub(crate) enum Command {
    /// List the classes a physical scope holds.
    ///
    /// Library entry: `Engine::list_class_declarations` — every candidate's own header really read,
    /// each item carrying the physical definition identity a later command consumes — or
    /// `Engine::list_class_candidates` for the same partition read from entry names alone.
    ListClasses(ListClasses),
    /// List one class definition's own members.
    ///
    /// Library entry: `Engine::list_members`, addressed by the definition identity a listing
    /// printed. No body is read and no analysis runs; a member that declares a `Code` attribute
    /// states exactly that, at its position.
    ListMembers(ListMembers),
    /// Look at the references to one named symbol over a physical scope.
    ///
    /// Library entries: `Engine::query` for the scan and `ReferenceGrouping::from_query` for the
    /// grouping the scan's own planes are moved into. The adapter picks no relation of its own, no
    /// consumer category and no page size: those are the request's statement.
    References(References),
    /// Open one class's code: its declaration, its members, and the method bodies asked for.
    ///
    /// Library entry: `Engine::class_view`. A body is decoded out of the same one class read the
    /// members came from, one `method_bodies` attempt each, and a body nobody asked for is not read
    /// at all.
    ClassView(ClassView),
    /// Recover one method's Java presentation.
    ///
    /// Library entry: `Engine::recover_target` — one analysis run under the operation's own stage
    /// table and the presentation of that very run's payload, beside the environment the caller
    /// declared and the identities the library bound.
    Recover(Recover),
}

/// The parameters every task command carries.
///
/// The five are the request's own statements, not the adapter's: which artifact to open, which
/// physical scope to scan, which dimensions of the bounded default budget to replace, which
/// rendering to write, and where it goes. Nothing here selects a target or reads a byte.
#[derive(Debug, Args)]
struct Common {
    /// The artifact to open: an archive or a standalone class file.
    #[arg(long, value_name = "PATH")]
    input: PathBuf,
    /// The physical scope to scan, as a `PhysicalScope` document (or `@FILE`); defaults to
    /// `{"kind":"snapshot_all"}`, the whole opened snapshot.
    ///
    /// `{"kind":"artifact_tree","root_container":…}` scans one container the caller names — the
    /// identity an `enumerate_artifact_tree` request printed. No root is inferred from a path here.
    #[arg(long, value_name = "JSON")]
    scope: Option<String>,
    /// One budget override, `dimension=limit`, repeatable.
    ///
    /// The five dimensions a task request may override are `output_bytes`, `elapsed_millis`,
    /// `result_items`, `class_headers` and `method_bodies`; every other dimension keeps its bounded
    /// default. An unknown dimension or a zero limit is a usage error, never a silent default.
    #[arg(long = "budget", value_name = "DIMENSION=LIMIT", action = ArgAction::Append)]
    budget: Vec<String>,
    /// The output format: `text` renders the report's content to standard output and its
    /// bookkeeping planes to standard error, `json` writes the library's own document.
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Write the document to FILE instead of standard output.
    ///
    /// The document is serialized and checked against `output_bytes` before the file is touched, so
    /// a refused charge leaves no file behind — and it is the same serialization, the same check and
    /// the same bytes standard output would have received.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Format {
    Text,
    Json,
}

/// Which evidence level of the class listing the command asks for.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Evidence {
    /// Every candidate's own header is read: one `class_headers` attempt each, the declaration it
    /// states, and the physical definition identity a later command is addressed by.
    Declarations,
    /// The partition of the scope read from entry names alone: no header, no class byte, and no
    /// claim that an entry holds a class.
    Candidates,
}

#[derive(Debug, Args)]
pub(crate) struct ListClasses {
    #[command(flatten)]
    common: Common,
    /// The evidence level this listing reads at.
    #[arg(long, value_enum, default_value = "declarations")]
    evidence: Evidence,
}

#[derive(Debug, Args)]
pub(crate) struct ListMembers {
    #[command(flatten)]
    common: Common,
    /// The class's physical definition identity, exactly as a listing printed it (JSON, or
    /// `@FILE`).
    ///
    /// The value is the library's own `PhysicalDefinitionId` document, consumed verbatim: its
    /// location, class-bytes digest and variant are verified against the snapshot this command
    /// opened, and a definition of another snapshot is an input error rather than a same-named
    /// substitute.
    #[arg(long, value_name = "JSON")]
    definition: String,
}

#[derive(Debug, Args)]
pub(crate) struct References {
    #[command(flatten)]
    common: Common,
    /// The class the referenced symbol's owner is named by: `a/b/C` (a class file's internal name)
    /// or `a.b.C` (source-style dotted).
    #[arg(long, value_name = "NAME")]
    class_name: String,
    /// What kind of symbol is referenced.
    #[arg(long, value_enum, default_value = "class")]
    member: MemberKind,
    /// The member's raw name, as the listing printed it.
    #[arg(long, value_name = "NAME")]
    member_name: Option<String>,
    /// The member's raw descriptor, byte for byte.
    #[arg(long, value_name = "DESCRIPTOR")]
    descriptor: Option<String>,
    /// The relation to scan for.
    ///
    /// The `literal_value` relation is not offered here: its target is a literal, and this command
    /// names symbols. The legacy `query` operation carries that one, and a cursor with it.
    #[arg(long, value_enum, default_value = "mentions-symbol")]
    relation: Relation,
    /// One consumer category the scan declares, by the library's own snake_case name (for example
    /// `invocation`, `field`, `type`); repeatable.
    ///
    /// At least one is required: the engine serves no implicit category set, and a scan that named
    /// none would be answered with a coverage claim nobody asked for.
    #[arg(long = "consumer", value_name = "KIND", action = ArgAction::Append)]
    consumers: Vec<String>,
    /// The page item limit; `0` means "no page limit".
    ///
    /// A task request carries no cursor: continuing a paged answer stays the legacy `query`
    /// operation's, where the cursor identity is replayed verbatim.
    #[arg(long = "max-items", default_value_t = 0)]
    max_items: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum MemberKind {
    Class,
    Method,
    Field,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Relation {
    MentionsSymbol,
    ConstantPoolContains,
    ReferencesDefinition,
    MayDispatchTo,
}

impl Relation {
    /// The library relation this parameter names.
    ///
    /// The mapping is spelled out rather than derived through `Serialize`: a `ValueEnum` spelling
    /// and a wire spelling that happened to agree today are two facts, and a rename of one would
    /// otherwise silently change which relation the caller asked for.
    fn into_query(self) -> QueryRelation {
        match self {
            Self::MentionsSymbol => QueryRelation::MentionsSymbol,
            Self::ConstantPoolContains => QueryRelation::ConstantPoolContains,
            Self::ReferencesDefinition => QueryRelation::ReferencesDefinition,
            Self::MayDispatchTo => QueryRelation::MayDispatchTo,
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct ClassView {
    #[command(flatten)]
    common: Common,
    /// The class to view, by the name the caller spells: `a/b/C` or `a.b.C`.
    #[arg(long, value_name = "NAME")]
    class_name: Option<String>,
    /// The class to view, by the physical definition identity a listing printed (JSON, or `@FILE`).
    #[arg(long, value_name = "JSON")]
    definition: Option<String>,
    /// One method body to read, as `name` or `name(descriptor)`; repeatable.
    ///
    /// A name without a descriptor is the overload case: every declared descriptor is a candidate,
    /// and the library answers with them instead of reading a body it cannot tell apart.
    #[arg(long = "body", value_name = "NAME[(DESCRIPTOR)]", action = ArgAction::Append)]
    body: Vec<String>,
    /// One method body to read, by the physical method identity a member listing printed (JSON, or
    /// `@FILE`); repeatable.
    #[arg(long = "body-method", value_name = "JSON", action = ArgAction::Append)]
    body_method: Vec<String>,
}

#[derive(Debug, Args)]
pub(crate) struct Recover {
    #[command(flatten)]
    common: Common,
    /// The method to recover, by the physical method identity a member listing printed (JSON, or
    /// `@FILE`).
    #[arg(long, value_name = "JSON")]
    method: Option<String>,
    /// The method's declaring class, by the name the caller spells: `a/b/C` or `a.b.C`.
    #[arg(long, value_name = "NAME")]
    class_name: Option<String>,
    /// The method's raw name, as the listing printed it.
    #[arg(long, value_name = "NAME")]
    method_name: Option<String>,
    /// The method's raw descriptor, byte for byte.
    ///
    /// Optional: a name declared with several descriptors is answered with those candidates instead
    /// of an elected overload.
    #[arg(long, value_name = "DESCRIPTOR")]
    descriptor: Option<String>,
    #[command(flatten)]
    environment: EnvironmentArgs,
}

/// How the request's own load positions are declared.
///
/// The three provided policies build the declaration a caller would write by hand for the same
/// shape, in the library's one place (`EnvironmentRequest::build`): nothing here reads a Manifest,
/// activates a nested library or organizes a layout into roots.
#[derive(Debug, Args)]
struct EnvironmentArgs {
    /// The environment policy: `plain-jar` roots the snapshot's own root container at its own root,
    /// `single-class` roots one whole class file, and `explicit-classpath` takes the caller's own
    /// roots in the caller's own order.
    #[arg(long, value_enum, default_value = "plain-jar")]
    policy: Policy,
    /// One load position as a `LoadRoot` document (or `@FILE`), for `--policy explicit-classpath`;
    /// repeatable and ordered. A tree enumeration's own identity is what a root names.
    #[arg(long = "root", value_name = "JSON", action = ArgAction::Append)]
    roots: Vec<String>,
    /// The JDK release the run targets.
    #[arg(long, default_value_t = 8)]
    release: u16,
    /// Whether the profile admits multi-release variants.
    #[arg(long = "multi-release", value_enum, default_value = "disabled")]
    multi_release: MultiRelease,
    /// The layout mode the profile declares.
    #[arg(long, value_enum, default_value = "generic")]
    layout: Layout,
    /// The runtime profile as a `RuntimeProfile` document (or `@FILE`), for a state the three
    /// friendly declarations cannot spell (`custom`, `unknown`).
    #[arg(
        long,
        value_name = "JSON",
        conflicts_with_all = ["release", "multi_release", "layout"]
    )]
    profile: Option<String>,
    /// The loader that owns the request's one domain.
    #[arg(long, default_value = "app")]
    loader: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Policy {
    SingleClass,
    PlainJar,
    ExplicitClasspath,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum MultiRelease {
    Disabled,
    Enabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Layout {
    Generic,
    War,
    SpringBoot,
}

/// Runs one task command and returns the process exit status.
pub fn run(command: Command) -> ExitCode {
    let (answer, session) = match dispatch(command) {
        Ok(dispatched) => dispatched,
        Err(failure) => return failure.exit(),
    };
    match session.deliver(&answer) {
        Ok(status) => status,
        Err(failure) => failure.exit(),
    }
}

/// One failed command: the status it exits with, and the document it writes to standard error.
///
/// A failure is not a report, so it never shares standard output with one: the exit status carries
/// the classification, and the document — the library's own `Error` beside the usage of the budget
/// the request ran under — says which code, message and dimension it was and what the request had
/// already cost.
struct Failure {
    status: ExitCode,
    /// Boxed so a `Result` that carries a failure in its `Err` arm stays small: the error is the
    /// one part whose size is the library's, and a failure is written once and then dropped.
    error: Box<Error>,
    /// Boxed for the same reason: the usage snapshot is eighteen counts.
    usage: Box<UsageSnapshot>,
}

impl Failure {
    /// A refusal the CLI itself makes, with no cost to state: exit 2.
    fn refused_default(error: Error) -> Self {
        Self::refused(error, UsageSnapshot::default())
    }

    /// A request-level refusal: exit 2, beside whatever the request had cost.
    fn refused(error: Error, usage: UsageSnapshot) -> Self {
        Self {
            status: ExitCode::from(EXIT_USAGE),
            error: Box::new(error),
            usage: Box::new(usage),
        }
    }

    /// The document itself could not be written: exit 1.
    fn delivery(operation: &str, error: io::Error) -> Self {
        Self {
            status: ExitCode::from(EXIT_DELIVERY),
            error: Box::new(Error::Io {
                operation: operation.to_owned(),
                message: error.to_string(),
            }),
            usage: Box::new(UsageSnapshot::default()),
        }
    }

    /// Writes the failure document to standard error and returns the classification.
    fn exit(self) -> ExitCode {
        let response = ErrorResponse {
            status: "error",
            error: &self.error,
            usage: &self.usage,
        };
        let mut document = match serde_json::to_vec(&response) {
            Ok(document) => document,
            Err(_) => br#"{"status":"error"}"#.to_vec(),
        };
        document.push(b'\n');
        if let Err(error) = io::stderr().lock().write_all(&document) {
            eprintln!("jarde-cli: failed to write the error document to standard error: {error}");
        }
        self.status
    }
}

impl From<Error> for Failure {
    /// A refusal reported before a budget exists — a parameter document, a usage error.
    fn from(error: Error) -> Self {
        Self::refused(error, UsageSnapshot::default())
    }
}

/// One command's finished answer: the library's own document, the classification its own planes
/// state, and the JSON path of the text body it carries when it carries one.
struct Answer {
    document: Value,
    plane: Plane,
    body: Option<&'static [&'static str]>,
}

/// What the report's own execution planes state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Plane {
    /// Every plane the report publishes is `Complete`.
    Complete,
    /// The library bound several physical identities and ran nothing.
    Ambiguous,
    /// The report stopped: `Partial`, `Cancelled` or `Failed`, with whatever prefix it published.
    Incomplete,
}

impl Answer {
    /// The answer of a command whose library value is the report itself.
    fn report<T: Serialize>(report: &T, plane: Plane) -> Result<Self, Failure> {
        Ok(Self {
            document: document(report)?,
            plane,
            body: None,
        })
    }

    /// The answer of a command whose library value is an `OperationOutcome`.
    ///
    /// The document is the outcome's own serialization — the `outcome` tag included — so an
    /// ambiguity and an unfinished selection are reports a caller reads rather than error
    /// envelopes it has to decode.
    fn outcome<T: Serialize>(
        outcome: &OperationOutcome<T>,
        plane: impl FnOnce(&T) -> Plane,
    ) -> Result<Self, Failure> {
        let plane = match outcome {
            OperationOutcome::Performed(report) => plane(report),
            OperationOutcome::Ambiguous(_) => Plane::Ambiguous,
            OperationOutcome::Incomplete(_) => Plane::Incomplete,
        };
        Ok(Self {
            document: document(outcome)?,
            plane,
            body: None,
        })
    }

    fn exit(&self) -> ExitCode {
        match self.plane {
            Plane::Complete => ExitCode::from(EXIT_COMPLETE),
            Plane::Ambiguous => ExitCode::from(EXIT_AMBIGUOUS),
            Plane::Incomplete => ExitCode::from(EXIT_INCOMPLETE),
        }
    }
}

/// One opened request: the artifact, the scope, the budget and the caller's own parameters.
struct Session {
    engine: Engine,
    snapshot: ArtifactSnapshot,
    scope: PhysicalScope,
    budget: Budget,
    common: Common,
}

impl Session {
    /// Opens one artifact under the task budget the parameters declare.
    ///
    /// This is the one place the adapter binds content: it opens `--input` and hands *that*
    /// snapshot to the library, exactly as the legacy operations do. Nothing else about a request is
    /// derived here — the scope is the caller's, and every target identity is the library's own.
    fn open(common: Common) -> Result<Self, Failure> {
        let overrides = budget_overrides(&common.budget)?;
        let mut budget = task_budget(&overrides)?;
        let engine = Engine::new();
        let opened = engine.open(ArtifactInput::Path(common.input.clone()), &mut budget);
        let snapshot = opened.map_err(|error| Failure::refused(error, budget.usage()))?;
        let scope = match common.scope.as_deref() {
            Some(document) => parse_argument(document, "cli_scope_json", "the physical scope")?,
            None => PhysicalScope::SnapshotAll,
        };
        Ok(Self {
            engine,
            snapshot,
            scope,
            budget,
            common,
        })
    }

    /// One library call, with the budget's own usage attached to a refusal.
    ///
    /// A refusal that happens after the artifact was opened is stated beside what the request had
    /// already cost, exactly as the legacy path's failures are: the error document then says both
    /// what went wrong and what it had spent.
    fn call<T>(
        &mut self,
        call: impl FnOnce(&Engine, &ArtifactSnapshot, &mut Budget) -> jarde::Result<T>,
    ) -> Result<T, Failure> {
        let result = call(&self.engine, &self.snapshot, &mut self.budget);
        result.map_err(|error| Failure::refused(error, self.budget.usage()))
    }

    /// Renders, funds and writes this command's document.
    ///
    /// The document is serialized once, funded once and written once, whichever destination carries
    /// it: a refused charge leaves both standard output and a named output file untouched, so a
    /// budget stop is visible in the report's own planes rather than in a half-written document.
    fn deliver(mut self, answer: &Answer) -> Result<ExitCode, Failure> {
        let rendering = match self.common.format {
            Format::Json => Rendering::Json(document_bytes(&answer.document)?),
            Format::Text => render_text(&answer.document, answer.body),
        };
        let rendered = rendering.document();
        let length = u64::try_from(rendered.len()).map_err(|_| {
            Failure::refused_default(Error::invalid_input(
                "cli_document_length",
                "the rendered document does not fit a budget count",
            ))
        })?;
        let funded = self
            .budget
            .check(CountedBudgetDimension::OutputBytes, length)
            .and_then(|()| {
                self.budget
                    .charge(CountedBudgetDimension::OutputBytes, length)
            });
        if let Err(error) = funded {
            return Err(Failure::refused(error, self.budget.usage()));
        }
        match &self.common.output {
            Some(path) => write_file(path, rendered)?,
            None => write_stream(&mut io::stdout().lock(), "cli_write_stdout", rendered)?,
        }
        if let Rendering::Text { bookkeeping, .. } = &rendering {
            write_stream(&mut io::stderr().lock(), "cli_write_stderr", bookkeeping)?;
        }
        Ok(answer.exit())
    }
}

/// The document one command writes, before it is written.
enum Rendering {
    Json(Vec<u8>),
    Text {
        content: Vec<u8>,
        bookkeeping: Vec<u8>,
    },
}

impl Rendering {
    /// The bytes the named destination — standard output or `--output` — receives.
    fn document(&self) -> &[u8] {
        match self {
            Self::Json(bytes) | Self::Text { content: bytes, .. } => bytes,
        }
    }
}

fn dispatch(command: Command) -> Result<(Answer, Session), Failure> {
    match command {
        Command::ListClasses(args) => list_classes(args),
        Command::ListMembers(args) => list_members(args),
        Command::References(args) => references(args),
        Command::ClassView(args) => class_view(args),
        Command::Recover(args) => recover(args),
    }
}

fn list_classes(args: ListClasses) -> Result<(Answer, Session), Failure> {
    let ListClasses { common, evidence } = args;
    let mut session = Session::open(common)?;
    let answer = match evidence {
        Evidence::Candidates => {
            let scope = session.scope.clone();
            let report = session.call(|engine, snapshot, budget| {
                engine.list_class_candidates(snapshot, &scope, budget)
            })?;
            Answer::report(&report, plane_of(&report.execution))?
        }
        Evidence::Declarations => {
            let scope = session.scope.clone();
            let report = session.call(|engine, snapshot, budget| {
                engine.list_class_declarations(snapshot, &scope, budget)
            })?;
            Answer::report(&report, plane_of(&report.execution))?
        }
    };
    Ok((answer, session))
}

fn list_members(args: ListMembers) -> Result<(Answer, Session), Failure> {
    let ListMembers { common, definition } = args;
    // The parameter is read before the artifact is opened: a request that names its target in a
    // document this adapter cannot read never becomes a read of anything.
    let definition: PhysicalDefinitionId = parse_argument(
        &definition,
        "cli_definition_json",
        "the definition identity",
    )?;
    let mut session = Session::open(common)?;
    let report = session
        .call(|engine, snapshot, budget| engine.list_members(snapshot, &definition, budget))?;
    let answer = Answer::report(&report, plane_of(&report.execution))?;
    Ok((answer, session))
}

fn references(args: References) -> Result<(Answer, Session), Failure> {
    let References {
        common,
        class_name,
        member,
        member_name,
        descriptor,
        relation,
        consumers,
        max_items,
    } = args;
    let mut session = Session::open(common)?;
    let request = QueryRequest {
        relation: relation.into_query(),
        target: QueryTarget::Symbol {
            value: symbol(
                &class_name,
                member,
                member_name.as_deref(),
                descriptor.as_deref(),
            )?,
        },
        // The adapter binds the one thing it owns — the snapshot it opened — and the caller declares
        // the scope it wants scanned. Validation, relation dispatch, paging and coverage all stay in
        // the library, so no result is filtered or re-scanned here.
        physical: PhysicalView {
            snapshot: session.snapshot.id().clone(),
            scope: session.scope.clone(),
        },
        consumers: consumer_schema(&consumers)?,
        max_items,
        cursor: None,
    };
    let report =
        session.call(|engine, snapshot, budget| engine.query(snapshot, &request, budget))?;
    // The grouping is the library's own reorganization of that one report: its planes stay the
    // scan's, and no finding is renamed, dropped or completed by this adapter.
    let grouping = ReferenceGrouping::from_query(report);
    let plane = match &grouping.source {
        ReferenceSource::Query { execution, .. }
        | ReferenceSource::Declaration { execution, .. } => plane_of(execution),
    };
    let answer = Answer::report(&grouping, plane)?;
    Ok((answer, session))
}

fn class_view(args: ClassView) -> Result<(Answer, Session), Failure> {
    let ClassView {
        common,
        class_name,
        definition,
        body,
        body_method,
    } = args;
    let class = match (class_name, definition) {
        (Some(name), None) => ClassRef::Name {
            class: class_name_query(&name),
        },
        (None, Some(definition)) => ClassRef::Definition {
            definition: parse_argument(
                &definition,
                "cli_definition_json",
                "the definition identity",
            )?,
        },
        (Some(_), Some(_)) => {
            return Err(Failure::refused_default(Error::invalid_input(
                "cli_target_conflict",
                "name the class with `--class-name` or with `--definition`, not both",
            )));
        }
        (None, None) => {
            return Err(Failure::refused_default(Error::invalid_input(
                "cli_target_missing",
                "name the class with `--class-name` or with `--definition`",
            )));
        }
    };
    let mut bodies = Vec::new();
    for spec in &body {
        bodies.push(body_spec(spec));
    }
    for identity in &body_method {
        bodies.push(BodyRef::Method {
            method: parse_argument(identity, "cli_body_json", "the body identity")?,
        });
    }
    let request = ClassViewRequest { class, bodies };
    let mut session = Session::open(common)?;
    let scope = session.scope.clone();
    let outcome = session
        .call(|engine, snapshot, budget| engine.class_view(snapshot, &scope, &request, budget))?;
    let answer = Answer::outcome(&outcome, class_view_plane)?;
    Ok((answer, session))
}

fn recover(args: Recover) -> Result<(Answer, Session), Failure> {
    let Recover {
        common,
        method,
        class_name,
        method_name,
        descriptor,
        environment,
    } = args;
    let method = match (method, class_name, method_name) {
        (Some(identity), None, None) => MethodRef::Method {
            method: parse_argument(&identity, "cli_method_json", "the method identity")?,
        },
        (None, Some(class), Some(name)) => MethodRef::Name {
            class: class_name_query(&class),
            name: jvm_bytes(&name),
            descriptor: descriptor.as_deref().map(jvm_bytes),
        },
        (Some(_), _, _) => {
            return Err(Failure::refused_default(Error::invalid_input(
                "cli_target_conflict",
                "name the method with `--method` or with `--class-name`/`--method-name`, not both",
            )));
        }
        (None, _, _) => {
            return Err(Failure::refused_default(Error::invalid_input(
                "cli_target_missing",
                "name the method with `--method`, or with `--class-name` and `--method-name`",
            )));
        }
    };
    let declaration = environment.declaration()?;
    let mut session = Session::open(common)?;
    let request = MethodOperationRequest {
        method,
        environment: EnvironmentRequest {
            // The snapshot is the one this invocation opened, like every other content binding this
            // adapter makes; the scope, the policy, the profile and the loader are the caller's.
            snapshot: session.snapshot.id().clone(),
            scope: session.scope.clone(),
            policy: declaration.policy,
            profile: declaration.profile,
            loader: declaration.loader,
        },
    };
    let outcome = session.call(|engine, snapshot, budget| {
        engine.recover_target(slice::from_ref(snapshot), &request, budget)
    })?;
    let mut answer = Answer::outcome(&outcome, method_recovery_plane)?;
    answer.body = Some(RECOVERED_BODY);
    Ok((answer, session))
}

/// One budget override named by its `dimension=limit` spelling.
///
/// The library owns the closed dimension set and the zero check, so an override this adapter would
/// have to re-decide is never re-decided here: an unknown name or a zero limit is refused with the
/// library's own code and message.
fn budget_overrides(values: &[String]) -> Result<Vec<BudgetOverride>, Error> {
    let mut overrides = Vec::new();
    for value in values {
        let Some((dimension, limit)) = value.split_once('=') else {
            return Err(Error::invalid_input(
                "cli_budget_override",
                format!("`{value}` is not a `dimension=limit` override"),
            ));
        };
        let limit: u64 = limit.trim().parse().map_err(|error| {
            Error::invalid_input("cli_budget_override", format!("`{value}`: {error}"))
        })?;
        overrides.push(BudgetOverride::new(dimension.trim(), limit)?);
    }
    Ok(overrides)
}

/// One JSON-valued parameter: the document itself, or `@FILE` naming a file that holds it.
fn read_argument(value: &str, what: &str) -> Result<String, Error> {
    match value.strip_prefix('@') {
        None => Ok(value.to_owned()),
        Some(path) => std::fs::read_to_string(path).map_err(|error| Error::Io {
            operation: "cli_read_argument".to_owned(),
            message: format!("{what}: `{path}`: {error}"),
        }),
    }
}

fn parse_argument<T: DeserializeOwned>(value: &str, code: &str, what: &str) -> Result<T, Error> {
    let text = read_argument(value, what)?;
    serde_json::from_str(&text).map_err(|error| {
        Error::invalid_input(
            code,
            format!("{what} is not the document this parameter takes: {error}"),
        )
    })
}

/// One class name in the spelling the caller used.
///
/// A name containing `/` is the class file's own internal name; anything else is source-style
/// dotted. The rule is input parsing, not a second name model: both spellings hand the library the
/// `ClassNameQuery` it defines, and the query's own `internal_name` is what a scan matches bytes
/// against.
fn class_name_query(spelling: &str) -> ClassNameQuery {
    if spelling.contains('/') {
        ClassNameQuery::internal(spelling)
    } else {
        ClassNameQuery::dotted(spelling)
    }
}

fn jvm_bytes(text: &str) -> JvmBytes {
    JvmBytes(text.as_bytes().to_vec())
}

/// One body reference the `--body` spelling states: `name`, or `name(descriptor)`.
fn body_spec(spec: &str) -> BodyRef {
    match spec.split_once('(') {
        None => BodyRef::Name {
            name: jvm_bytes(spec),
            descriptor: None,
        },
        Some((name, descriptor)) => BodyRef::Name {
            name: jvm_bytes(name),
            descriptor: Some(jvm_bytes(&format!("({descriptor}"))),
        },
    }
}

/// The symbol one reference request names.
fn symbol(
    class_name: &str,
    member: MemberKind,
    name: Option<&str>,
    descriptor: Option<&str>,
) -> Result<SymbolRef, Error> {
    let owner = class_name_query(class_name).internal_name().clone();
    let member_bytes = |what: &str| -> Result<(JvmBytes, JvmBytes), Error> {
        match (name, descriptor) {
            (Some(name), Some(descriptor)) => Ok((jvm_bytes(name), jvm_bytes(descriptor))),
            _ => Err(Error::invalid_input(
                "cli_member_target_missing",
                format!(
                    "a {what} target names `--member-name` and `--descriptor`: the scan matches the \
                     symbol's own bytes, and a missing descriptor is not a wildcard"
                ),
            )),
        }
    };
    match member {
        MemberKind::Class => {
            if name.is_some() || descriptor.is_some() {
                return Err(Error::invalid_input(
                    "cli_member_target_conflict",
                    "a class target names `--class-name` alone: drop `--member-name` and \
                     `--descriptor`",
                ));
            }
            Ok(SymbolRef::Class { owner })
        }
        MemberKind::Method => {
            let (name, descriptor) = member_bytes("method")?;
            Ok(SymbolRef::Method {
                owner,
                name,
                descriptor,
            })
        }
        MemberKind::Field => {
            let (name, descriptor) = member_bytes("field")?;
            Ok(SymbolRef::Field {
                owner,
                name,
                descriptor,
            })
        }
    }
}

/// The consumer schema one reference scan declares from its `--consumer` categories.
fn consumer_schema(kinds: &[String]) -> Result<ConsumerSchema, Error> {
    if kinds.is_empty() {
        return Err(Error::invalid_input(
            "cli_consumer_kinds_missing",
            "a reference scan declares at least one `--consumer` category: the engine serves no \
             implicit set, and a scan that named none would be a coverage claim nobody asked for",
        ));
    }
    let mut parsed = Vec::new();
    for kind in kinds {
        // The category names are the library's own serde spelling, so no second vocabulary of
        // consumer kinds exists here to drift from the engine's.
        let kind: ConsumerKind =
            serde_json::from_value(Value::String(kind.clone())).map_err(|_| {
                Error::invalid_input(
                    "cli_consumer_kind_unknown",
                    format!("`{kind}` is not a consumer category this engine names"),
                )
            })?;
        parsed.push(kind);
    }
    Ok(ConsumerSchema::new(CONSUMER_SCHEMA_VERSION, parsed))
}

/// The environment declaration one recovery request carries, as the caller's parameters state it.
///
/// Parsing the declaration and binding the opened snapshot are two steps: this one reads no
/// artifact at all, so a parameter this adapter cannot read never becomes a read of anything.
struct Declaration {
    policy: EnvironmentPolicy,
    profile: RuntimeProfile,
    loader: LoaderId,
}

impl EnvironmentArgs {
    /// The declaration these parameters state.
    ///
    /// The adapter states what the caller stated — a policy, its roots, a profile and a loader — and
    /// hands the declaration to the library, which builds the environment and owns every check a
    /// policy has (a kind mismatch, a root the request did not provide). Nothing here reads a
    /// Manifest, activates a nested library or organizes a layout into roots.
    fn declaration(&self) -> Result<Declaration, Error> {
        let policy = match self.policy {
            Policy::SingleClass => EnvironmentPolicy::SingleClass,
            Policy::PlainJar => EnvironmentPolicy::PlainJar,
            Policy::ExplicitClasspath => {
                let mut roots: Vec<LoadRoot> = Vec::new();
                for root in &self.roots {
                    roots.push(parse_argument(root, "cli_root_json", "a load position")?);
                }
                EnvironmentPolicy::ExplicitClasspath { roots }
            }
        };
        let profile = match &self.profile {
            Some(document) => parse_argument(document, "cli_profile_json", "the runtime profile")?,
            None => RuntimeProfile {
                java_release: self.release,
                multi_release: match self.multi_release {
                    MultiRelease::Disabled => MultiReleasePolicy::Disabled,
                    MultiRelease::Enabled => MultiReleasePolicy::Enabled,
                },
                layout: match self.layout {
                    Layout::Generic => LayoutMode::Generic,
                    Layout::War => LayoutMode::War,
                    Layout::SpringBoot => LayoutMode::SpringBoot,
                },
            },
        };
        Ok(Declaration {
            policy,
            profile,
            loader: LoaderId(self.loader.clone()),
        })
    }
}

/// What one execution plane states.
fn plane_of(execution: &ExecutionReport) -> Plane {
    match execution {
        ExecutionReport::Complete { .. } => Plane::Complete,
        ExecutionReport::Partial { .. }
        | ExecutionReport::Cancelled { .. }
        | ExecutionReport::Failed { .. } => Plane::Incomplete,
    }
}

/// The plane one class view ran under: the report's own execution.
///
/// The library merges the search, the class read and every requested body's stop into that one
/// plane, so a view with a stopped body is already non-`Complete` there. The adapter reads it and
/// derives nothing: walking `bodies` here would be a second implementation of the library's merge,
/// and the two would be free to disagree.
fn class_view_plane(report: &ClassViewReport) -> Plane {
    plane_of(&report.execution)
}

/// The plane one recovery presentation ran under.
fn method_recovery_plane(report: &MethodRecoveryReport) -> Plane {
    plane_of(&report.presentation.execution)
}

fn document<T: Serialize>(value: &T) -> Result<Value, Failure> {
    serde_json::to_value(value)
        .map_err(|error| Failure::delivery("cli_document_json", io::Error::other(error)))
}

fn document_bytes(value: &Value) -> Result<Vec<u8>, Failure> {
    let mut bytes = serde_json::to_vec(value)
        .map_err(|error| Failure::delivery("cli_document_json", io::Error::other(error)))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// The two standard streams one report's text mode writes.
///
/// Every line is a `path = value` rendering of one field of the very document the JSON mode writes,
/// so the correspondence between the renderings is a property of how this is written. The one
/// exception is a report that carries a text body: that field *is* the body, so it is written
/// verbatim to standard output and the whole document beside it becomes bookkeeping.
fn render_text(document: &Value, body: Option<&[&str]>) -> Rendering {
    let mut content = String::new();
    let mut bookkeeping = String::new();
    match body.and_then(|path| lookup_text(document, path)) {
        Some((path, text)) => {
            content.push_str(text);
            project(document, Some(path.as_str()), &mut |_, line| {
                bookkeeping.push_str(&line);
            });
        }
        None => project(document, None, &mut |is_bookkeeping, line| {
            if is_bookkeeping {
                bookkeeping.push_str(&line);
            } else {
                content.push_str(&line);
            }
        }),
    }
    Rendering::Text {
        content: content.into_bytes(),
        bookkeeping: bookkeeping.into_bytes(),
    }
}

/// Renders one JSON document as `path = value` lines, skipping one field path when asked to.
fn project(value: &Value, skip: Option<&str>, emit: &mut impl FnMut(bool, String)) {
    render_field(value, "", skip, emit);
}

fn render_field(
    value: &Value,
    path: &str,
    skip: Option<&str>,
    emit: &mut impl FnMut(bool, String),
) {
    match value {
        Value::Object(fields) => {
            for (key, child) in fields {
                let child_path = join(path, key);
                if skip != Some(child_path.as_str()) {
                    render_field(child, &child_path, skip, emit);
                }
            }
        }
        Value::Array(items) if !items.is_empty() && items.iter().all(Value::is_object) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = format!("{path}.{index}");
                if skip != Some(child_path.as_str()) {
                    render_field(child, &child_path, skip, emit);
                }
            }
        }
        leaf => emit(bookkeeping(path), format!("{path} = {leaf}\n")),
    }
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

/// Whether one field path belongs to a report's bookkeeping planes rather than to its content.
///
/// The five names are the vocabulary every entry point publishes its bookkeeping under: what a
/// request limited, what it consumed, what it looked at, how far it got, and what it observed. A
/// content field is not named one of them, so this is a path-segment test rather than a list of the
/// fields this adapter happens to know about — and a plane a later report adds under the same
/// vocabulary is separated without the adapter being told about it.
fn bookkeeping(path: &str) -> bool {
    const PLANES: [&str; 5] = ["limits", "usage", "coverage", "execution", "diagnostics"];
    path.split('.').any(|segment| PLANES.contains(&segment))
}

/// The text one report holds at `path`, when it holds a string there.
fn lookup_text<'a>(document: &'a Value, path: &[&str]) -> Option<(String, &'a str)> {
    let text = lookup(document, path)?.as_str()?;
    Some((path.join("."), text))
}

fn lookup<'a>(document: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = document;
    for segment in path {
        current = current.as_object()?.get(*segment)?;
    }
    Some(current)
}

fn write_stream(stream: &mut impl Write, operation: &str, bytes: &[u8]) -> Result<(), Failure> {
    stream
        .write_all(bytes)
        .map_err(|error| Failure::delivery(operation, error))
}

/// Writes the document to the named file.
///
/// The file is created only after the document was serialized and funded: a refused `output_bytes`
/// charge never leaves a file behind, and the bytes written are the ones standard output would have
/// received.
fn write_file(path: &Path, bytes: &[u8]) -> Result<(), Failure> {
    let mut file =
        File::create(path).map_err(|error| Failure::delivery("cli_create_output", error))?;
    file.write_all(bytes)
        .map_err(|error| Failure::delivery("cli_write_output", error))
}
