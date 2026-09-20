//! Explicit runtime environment for P2 resolution and method analysis.
//!
//! An environment is a *declaration*: physical view, loader domains, the caller's domain
//! and the header providers a request runs under. Nothing in this module reads artifact
//! bytes — the entry-provided `content` snapshots are matched by [`SnapshotId`] only — and
//! nothing here guesses platform content, a classpath or a delegation outcome. P1 physical
//! X0/X1 requests carry no environment, which is why they can never start the resolver
//! (A17).
//!
//! [`validate_environment`] decides everything that the declarations alone can decide and
//! returns every violation it found; [`validate_environment_with_caller`] adds the one check
//! that needs the request's own `CallerContext` as well, because that identity is not part of
//! the environment. The problem codes are a closed set: a validation failure never turns into a
//! broken-down "best effort" search order, it stays a reportable environment problem that keeps
//! the original symbols and the unperformed range.

use jarde_reader::artifact::ArtifactSnapshot;
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    Diagnostic, DiagnosticSeverity, PhysicalMethodId, SnapshotId, SymbolRef,
};
use jarde_reader::view::{
    DelegationPolicy, LoadDomain, LoadRoot, LoaderId, ModuleMode, RuntimeView,
};
use serde::{Deserialize, Serialize};

/// Name of one declared provider of readable headers.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderId(pub String);

/// Readable content declared by name.
///
/// A provider carries no order, priority or delegation: search order belongs to
/// [`LoadDomain::roots`] alone, and a provider root only names content the entry has
/// already provided. A provider must not add a search position that no domain lists.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeaderProvider {
    pub id: ProviderId,
    pub roots: Vec<LoadRoot>,
}

/// Runtime view, participating domains and header providers of one request.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionEnvironment {
    /// Identity: physical view, runtime profile and the caller's own domain.
    pub runtime: RuntimeView,
    /// Participating domains, declaration order; each loader owns exactly one entry and
    /// exactly one entry is equal to `runtime.load_domain`.
    pub domains: Vec<LoadDomain>,
    /// May be empty. An empty list means "no additional content source", never "guess".
    pub providers: Vec<HeaderProvider>,
}

/// Where a resolution request is issued from.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallerContext {
    pub loader: LoaderId,
    /// Method that contains the use site; a declaration query may omit it.
    pub enclosing: Option<PhysicalMethodId>,
}

/// Closed set of environment problems, serialized as `snake_case`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentProblemCode {
    DuplicateLoader,
    CallerDomainMismatch,
    /// The request's `CallerContext` and the environment's caller domain name different
    /// loaders; both claim to be the caller's identity.
    CallerLoaderMismatch,
    MissingParent,
    ParentCycle,
    UnsupportedPolicy,
    UnreadableRoot,
    ContentNotProvided,
    ProviderRootUnbound,
    /// A container root's prefix is neither empty nor closed by `/`, so the declaration does not
    /// name a byte boundary inside its container and this engine refuses to guess one.
    InvalidRootPrefix,
}

impl EnvironmentProblemCode {
    /// The closed set in declaration order.
    pub const ALL: [Self; 10] = [
        Self::DuplicateLoader,
        Self::CallerDomainMismatch,
        Self::CallerLoaderMismatch,
        Self::MissingParent,
        Self::ParentCycle,
        Self::UnsupportedPolicy,
        Self::UnreadableRoot,
        Self::ContentNotProvided,
        Self::ProviderRootUnbound,
        Self::InvalidRootPrefix,
    ];

    /// Diagnostic code of this problem: the same `snake_case` name serde writes.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateLoader => "duplicate_loader",
            Self::CallerDomainMismatch => "caller_domain_mismatch",
            Self::CallerLoaderMismatch => "caller_loader_mismatch",
            Self::MissingParent => "missing_parent",
            Self::ParentCycle => "parent_cycle",
            Self::UnsupportedPolicy => "unsupported_policy",
            Self::UnreadableRoot => "unreadable_root",
            Self::ContentNotProvided => "content_not_provided",
            Self::ProviderRootUnbound => "provider_root_unbound",
            Self::InvalidRootPrefix => "invalid_root_prefix",
        }
    }
}

/// Locator of an environment problem: the declaration that has to change.
///
/// The variants are newtype (or struct) payloads, and serde cannot serialize newtype
/// variants under an internal tag, so this enum uses the default externally tagged shape
/// with the `snake_case` variant names of the rest of the schema
/// (`{"loader":"app"}`, `{"root":{"loader":"app","index":0}}`). The variant names,
/// payloads and field names are the designed ones.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentSubject {
    Loader(LoaderId),
    Provider(ProviderId),
    /// Root position inside one domain, by declaration order.
    Root {
        loader: LoaderId,
        index: u32,
    },
    Symbol(SymbolRef),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentProblem {
    pub code: EnvironmentProblemCode,
    pub subject: EnvironmentSubject,
    pub message: String,
}

/// Environment identity a report points back at.
///
/// Order is declaration order throughout. Digest and hash of the environment are added
/// only when a consumer needs them.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentIdentity {
    pub runtime: RuntimeView,
    pub domain_loaders: Vec<LoaderId>,
    pub providers: Vec<ProviderId>,
    /// Snapshots the entry actually provided, in argument order.
    pub content: Vec<SnapshotId>,
}

/// Decides whether the environment declarations are usable, without reading one byte.
///
/// The checks are decided from declarations and from the identity of the provided
/// snapshots only. An unreadable or unprovided root is a problem, not an instruction to
/// guess: the caller sees exactly which declaration is unusable and still gets the
/// identity of the environment that was rejected.
///
/// The request's own `CallerContext` is not part of the environment, so this entry cannot
/// decide the one check that needs both: it stays the declaration-only decision, and the
/// entries that carry a caller use [`validate_environment_with_caller`] instead.
pub fn validate_environment(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
) -> (Vec<EnvironmentProblem>, EnvironmentIdentity) {
    decide_environment(content, environment, None)
}

/// The same decision, with the request's caller identity included.
///
/// An environment is a declaration of the caller's domain, and a request carries a
/// `CallerContext` of its own; the two name the same identity, so the closed set has one code
/// for their disagreement (`CallerLoaderMismatch`). A declaration-only check cannot see it: the
/// mismatch exists between the environment and the request, not inside either one.
pub(crate) fn validate_environment_with_caller(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    caller: &CallerContext,
) -> (Vec<EnvironmentProblem>, EnvironmentIdentity) {
    decide_environment(content, environment, Some(caller))
}

/// The one decision both entries share; `caller` adds the check that needs the request.
fn decide_environment(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    caller: Option<&CallerContext>,
) -> (Vec<EnvironmentProblem>, EnvironmentIdentity) {
    let mut problems = Vec::new();
    validate_domain_loaders(environment, &mut problems);
    validate_caller_domain(environment, &mut problems);
    if let Some(caller) = caller {
        validate_caller_context(environment, caller, &mut problems);
    }
    for domain in &environment.domains {
        validate_domain_policy(domain, &mut problems);
        validate_domain_parent(domain, environment, &mut problems);
        validate_domain_roots(domain, content, &mut problems);
    }
    validate_parent_cycles(environment, &mut problems);
    validate_providers(environment, &mut problems);
    let identity = EnvironmentIdentity {
        runtime: environment.runtime.clone(),
        domain_loaders: environment
            .domains
            .iter()
            .map(|domain| domain.loader.clone())
            .collect(),
        providers: environment
            .providers
            .iter()
            .map(|provider| provider.id.clone())
            .collect(),
        content: content
            .iter()
            .map(|snapshot| snapshot.id().clone())
            .collect(),
    };
    (problems, identity)
}

/// Request-level check: the request's runtime snapshot must be one of the entry-provided
/// snapshots.
///
/// This is a caller mismatch rather than an environment problem, so it surfaces as an
/// input error and the caller gets no report at all. It reads no bytes: snapshots are
/// identified by their id.
pub(crate) fn require_content_snapshot(
    content: &[ArtifactSnapshot],
    snapshot: &SnapshotId,
) -> Result<()> {
    if content.iter().any(|candidate| candidate.id() == snapshot) {
        Ok(())
    } else {
        Err(Error::invalid_input(
            "resolution_snapshot_mismatch",
            format!(
                "request snapshot `{}` is not provided by the request content",
                snapshot.0
            ),
        ))
    }
}

/// Environment problems are reported twice: structured in `environment_problems` and as
/// stable diagnostics under the same closed-set code.
pub(crate) fn environment_diagnostics(problems: &[EnvironmentProblem]) -> Vec<Diagnostic> {
    problems
        .iter()
        .map(|problem| Diagnostic {
            code: problem.code.as_str().to_string(),
            severity: DiagnosticSeverity::Error,
            message: problem.message.clone(),
            provenance: None,
        })
        .collect()
}

/// Diagnostic of a capability that this engine slice does not implement yet.
///
/// The code is the capability name and disappears together with the capability; the
/// diagnostic explains that the request and its environment were validated, that no
/// artifact byte was read and that no result was produced.
pub(crate) fn unavailable_diagnostic(code: &str, capability: &str) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Error,
        message: format!(
            "{capability} is not implemented in this engine slice: the request shape and \
             its environment were validated, no artifact byte was read, and no result was \
             produced"
        ),
        provenance: None,
    }
}

fn problem(
    code: EnvironmentProblemCode,
    subject: EnvironmentSubject,
    message: String,
) -> EnvironmentProblem {
    EnvironmentProblem {
        code,
        subject,
        message,
    }
}

fn domain_for_loader<'a>(
    environment: &'a ResolutionEnvironment,
    loader: &LoaderId,
) -> Option<&'a LoadDomain> {
    environment
        .domains
        .iter()
        .find(|domain| &domain.loader == loader)
}

/// Every loader owns exactly one domain: a repeated loader could silence a root list.
fn validate_domain_loaders(
    environment: &ResolutionEnvironment,
    problems: &mut Vec<EnvironmentProblem>,
) {
    let mut seen: Vec<&LoaderId> = Vec::new();
    for domain in &environment.domains {
        if seen.contains(&&domain.loader) {
            let declared = environment
                .domains
                .iter()
                .filter(|candidate| candidate.loader == domain.loader)
                .count();
            problems.push(problem(
                EnvironmentProblemCode::DuplicateLoader,
                EnvironmentSubject::Loader(domain.loader.clone()),
                format!(
                    "loader `{}` is declared {} times in `domains`; one loader must own \
                     exactly one domain",
                    domain.loader.0, declared
                ),
            ));
        } else {
            seen.push(&domain.loader);
        }
    }
}

/// The caller's domain is the runtime identity that drives delegation: it must be present
/// exactly once and the two declarations must be equal.
///
/// A missing caller domain is a uniqueness violation like a repeated one, so both are
/// reported under `DuplicateLoader`; only a present-but-different caller domain is
/// `CallerDomainMismatch`. The messages name which of the two happened.
fn validate_caller_domain(
    environment: &ResolutionEnvironment,
    problems: &mut Vec<EnvironmentProblem>,
) {
    let caller = &environment.runtime.load_domain;
    let subject = EnvironmentSubject::Loader(caller.loader.clone());
    let matches = environment
        .domains
        .iter()
        .filter(|domain| domain.loader == caller.loader)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => problems.push(problem(
            EnvironmentProblemCode::DuplicateLoader,
            subject,
            format!(
                "caller loader `{}` has no domain in `domains`; exactly one entry must be \
                 equal to `runtime.load_domain`",
                caller.loader.0
            ),
        )),
        [found] if *found == caller => {}
        [_] => problems.push(problem(
            EnvironmentProblemCode::CallerDomainMismatch,
            subject,
            format!(
                "domain `{}` differs from `runtime.load_domain` under the same loader; the \
                 request may not search a domain other than the one it declares",
                caller.loader.0
            ),
        )),
        repeated => problems.push(problem(
            EnvironmentProblemCode::DuplicateLoader,
            subject,
            format!(
                "caller loader `{}` is declared {} times in `domains`; the caller identity \
                 is ambiguous",
                caller.loader.0,
                repeated.len()
            ),
        )),
    }
}

/// The request's caller identity and the environment's caller domain must name one loader.
///
/// Both declarations claim to be the caller: the domain whose roots the search starts from and
/// the context the use site was found in. A search that started from one while reporting the
/// other would publish two caller identities in one report, so the disagreement is a problem
/// instead of a silently picked side.
fn validate_caller_context(
    environment: &ResolutionEnvironment,
    caller: &CallerContext,
    problems: &mut Vec<EnvironmentProblem>,
) {
    let declared = &environment.runtime.load_domain.loader;
    if &caller.loader == declared {
        return;
    }
    problems.push(problem(
        EnvironmentProblemCode::CallerLoaderMismatch,
        EnvironmentSubject::Loader(caller.loader.clone()),
        format!(
            "the request names caller loader `{}` while `runtime.load_domain` is loader `{}`; \
             the two declarations of the caller identity must be equal",
            caller.loader.0, declared.0
        ),
    ));
}

/// Policies this slice cannot execute must be refused, not flattened into one classpath.
fn validate_domain_policy(domain: &LoadDomain, problems: &mut Vec<EnvironmentProblem>) {
    match &domain.module_mode {
        ModuleMode::ClassPath => {}
        ModuleMode::ModulePath => unsupported_policy(problems, domain, "module mode `module_path`"),
        ModuleMode::Hybrid => unsupported_policy(problems, domain, "module mode `hybrid`"),
        ModuleMode::Custom { id } => {
            unsupported_policy(problems, domain, &format!("module mode `custom` ({id})"));
        }
        ModuleMode::Unknown => unsupported_policy(problems, domain, "unknown module mode"),
    }
    match &domain.delegation {
        DelegationPolicy::ParentFirst | DelegationPolicy::ChildFirst => {}
        DelegationPolicy::Custom { id } => unsupported_policy(
            problems,
            domain,
            &format!("custom delegation policy `{id}`"),
        ),
        DelegationPolicy::Unknown => {
            unsupported_policy(problems, domain, "unknown delegation policy")
        }
    }
}

fn unsupported_policy(
    problems: &mut Vec<EnvironmentProblem>,
    domain: &LoadDomain,
    declaration: &str,
) {
    problems.push(problem(
        EnvironmentProblemCode::UnsupportedPolicy,
        EnvironmentSubject::Loader(domain.loader.clone()),
        format!(
            "loader `{}` declares {declaration}; this slice resolves only `class_path` \
             domains with `parent_first` or `child_first` delegation",
            domain.loader.0
        ),
    ));
}

fn validate_domain_parent(
    domain: &LoadDomain,
    environment: &ResolutionEnvironment,
    problems: &mut Vec<EnvironmentProblem>,
) {
    let Some(parent) = &domain.parent_loader else {
        return;
    };
    if domain_for_loader(environment, parent).is_none() {
        problems.push(problem(
            EnvironmentProblemCode::MissingParent,
            EnvironmentSubject::Loader(domain.loader.clone()),
            format!(
                "loader `{}` declares parent `{}` but `domains` has no domain for it",
                domain.loader.0, parent.0
            ),
        ));
    }
}

/// Roots are readable only when the entry provided their content, and a container root's prefix
/// has to name a byte boundary; `External` stays an unreadable declaration.
///
/// The prefix check is decided from the declaration alone and is the only new refusal this
/// shape adds: a prefix that is neither empty nor closed by `/` names no boundary inside its
/// container, and guessing one (adding the separator, trimming, decoding or folding bytes) would
/// bind a class the caller never declared. Everything else about a container root — that the
/// origin really derives from its snapshot and that the container's directory is complete — is a
/// physical fact the search checks when the position is really searched, because it needs the
/// artifact's bytes and this validator reads none.
fn validate_domain_roots(
    domain: &LoadDomain,
    content: &[ArtifactSnapshot],
    problems: &mut Vec<EnvironmentProblem>,
) {
    for (position, root) in domain.roots.iter().enumerate() {
        let index = u32::try_from(position).unwrap_or(u32::MAX);
        let subject = EnvironmentSubject::Root {
            loader: domain.loader.clone(),
            index,
        };
        match root {
            LoadRoot::External { id } => problems.push(problem(
                EnvironmentProblemCode::UnreadableRoot,
                subject,
                format!(
                    "root {index} of loader `{}` is the external declaration `{id}`; an \
                     external root is declared but not readable and resolves nothing",
                    domain.loader.0
                ),
            )),
            LoadRoot::StandaloneClass { snapshot } => {
                if !content.iter().any(|candidate| candidate.id() == snapshot) {
                    problems.push(problem(
                        EnvironmentProblemCode::ContentNotProvided,
                        subject,
                        format!(
                            "root {index} of loader `{}` names the standalone CLASS snapshot \
                             `{}`, which the request content does not provide",
                            domain.loader.0, snapshot.0
                        ),
                    ));
                }
            }
            LoadRoot::Container { origin, prefix } => {
                if !prefix.0.is_empty() && prefix.0.last() != Some(&b'/') {
                    problems.push(problem(
                        EnvironmentProblemCode::InvalidRootPrefix,
                        subject.clone(),
                        format!(
                            "root {index} of loader `{}` declares prefix \"{}\" in container \
                             `{}` of snapshot `{}`; a prefix is empty or ends with `/`, and this \
                             engine does not guess the missing boundary",
                            domain.loader.0,
                            crate::providers::escaped(&prefix.0),
                            origin.current_container().0,
                            origin.snapshot.0
                        ),
                    ));
                }
                if !content
                    .iter()
                    .any(|candidate| candidate.id() == &origin.snapshot)
                {
                    problems.push(problem(
                        EnvironmentProblemCode::ContentNotProvided,
                        subject,
                        format!(
                            "root {index} of loader `{}` names container `{}` of snapshot `{}`, \
                             which the request content does not provide",
                            domain.loader.0,
                            origin.current_container().0,
                            origin.snapshot.0
                        ),
                    ));
                }
            }
        }
    }
}

/// A parent graph with a cycle cannot be walked to a root, so every domain whose parent
/// chain re-enters itself is reported once, with the repeating path.
fn validate_parent_cycles(
    environment: &ResolutionEnvironment,
    problems: &mut Vec<EnvironmentProblem>,
) {
    for domain in &environment.domains {
        let mut path: Vec<&LoaderId> = vec![&domain.loader];
        let mut current = domain.parent_loader.as_ref();
        while let Some(loader) = current {
            if path.contains(&loader) {
                let mut chain = path
                    .iter()
                    .map(|loader| loader.0.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                chain.push_str(" -> ");
                chain.push_str(&loader.0);
                problems.push(problem(
                    EnvironmentProblemCode::ParentCycle,
                    EnvironmentSubject::Loader(domain.loader.clone()),
                    format!(
                        "parent chain from loader `{}` re-enters `{}` ({chain})",
                        domain.loader.0, loader.0
                    ),
                ));
                break;
            }
            path.push(loader);
            current = domain_for_loader(environment, loader).and_then(|d| d.parent_loader.as_ref());
        }
    }
}

/// A provider names content; it must not add a search position.
///
/// Each provider root is compared for equality against the roots of every participating
/// domain, so a provider cannot introduce a loader, an order or a snapshot the
/// environment never declared.
fn validate_providers(environment: &ResolutionEnvironment, problems: &mut Vec<EnvironmentProblem>) {
    for provider in &environment.providers {
        let unbound = provider
            .roots
            .iter()
            .enumerate()
            .filter(|(_, root)| {
                !environment
                    .domains
                    .iter()
                    .any(|domain| domain.roots.contains(root))
            })
            .map(|(position, _)| u32::try_from(position).unwrap_or(u32::MAX))
            .collect::<Vec<_>>();
        if !unbound.is_empty() {
            problems.push(problem(
                EnvironmentProblemCode::ProviderRootUnbound,
                EnvironmentSubject::Provider(provider.id.clone()),
                format!(
                    "provider `{}` declares roots {unbound:?} that no participating domain \
                     lists in `roots`; a provider names content, it does not add search \
                     positions",
                    provider.id.0
                ),
            ));
        }
    }
}
