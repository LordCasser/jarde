//! The class's own members a presented body's call sites named, read **on demand** (P3 3.2).
//!
//! # Why this read exists, and why it is not a wider one
//!
//! A presentation of one body can present a call to a synthetic accessor as the field access the
//! accessor forwards — but only if it holds that member's own declaration and body, which the
//! presented body's payload cannot contain (it is one method's). The recovery layer above cannot
//! read them: it has no artifact, no loader and no budget, and a layer that read its own evidence
//! would be a second reader of the same bytes with a second idea of what a read costs.
//!
//! So this entry reads them here, and it reads **exactly the candidates it is given**: the call
//! sites one recovery run's own decode named (`jarde-java`'s `accessor@1`, the rule that decides
//! from them). Nothing enumerates the class's members to find them, nothing reads a body no
//! candidate named, and a class with forty members costs one header read and one body attempt per
//! candidate. That is A16's boundary applied to this read: the demand is named before it is paid
//! for.
//!
//! # What one read is bound to
//!
//! The request names the class **by physical definition** — the very definition the presented body
//! was read from — and not by a name a caller could point elsewhere. So:
//!
//! * every member this entry returns is declared in that one class file, and its
//!   [`PhysicalMethodId`] states that definition: the member's name and descriptor alone are a
//!   claim about *some* class, and a same-named member of another class file is never this
//!   member;
//! * a body is decoded against the constant pool of the same header read those bytes came from, so
//!   every constant-pool index the callers of this evidence see is an index in **that** pool;
//! * the header read is the same identity read the driver's own read was (`this_class` bound under
//!   the declared loader, bytes checked against the definition's digest and length), which is why a
//!   candidate whose owner is not the name that definition declares for itself is refused rather
//!   than read from a same-named member of the class that happens to be here.
//!
//! # Cost, and what a failure is
//!
//! One header read (`ClassHeaders`, a second read of the definition the presented body came from —
//! the payload carries that body's tables, not its bytes) and, per distinct candidate member the
//! class really declares with a body, one pre-charged `MethodBodies` attempt and the body's own
//! `CodeBytes`/`AttributeBytes` reads, exactly as the driver's own body read is charged. A budget
//! stop and a cancellation are the budget's own errors, raised from this entry like every other
//! read of this engine; a *shape* failure — a candidate naming another class, one the class does
//! not declare, a member the class declares without a body — is a stated refusal in the report, with
//! the call site's BCI in it, because "there is no such member" is an answer the caller must be able
//! to read rather than an error that hides the rest of the answer.
//!
//! The report is read-only evidence: the members are borrowed through accessors, the body facts are
//! the reader's own type, and nothing here hands out a mutable internal of the analysis engine.
//!
//! # Two entries, one read
//!
//! [`read_callees`] performs the class read itself. [`read_prepared_callees`] is the same read from a
//! class the caller already prepared (bulk task 2.3): same refusals, same member order, same charges
//! per member, same read record — minus the read of the class definition itself and its
//! `ClassHeaders` charge, which the preparation paid once for every method of that class. A bulk
//! worker holds the prepared class
//! beside the run it presents, so the members a presented body's call sites name come from the very
//! member table, pool and decoder that run read its body with.

use serde::Serialize;

use jarde_reader::budget::{Budget, CountedBudgetDimension, UsageSnapshot};
use jarde_reader::classfile::{MemberHeader, MethodCodeFacts};
use jarde_reader::error::Result;
use jarde_reader::model::{JvmBytes, PhysicalDefinitionId, PhysicalMethodId};
use jarde_reader::prepared::{MethodOrdinal, PreparedClass};

use crate::environment::ResolutionEnvironment;
use crate::providers::{DefinitionContent, HeaderClosure, HeaderDemand};
use crate::resolver::{HeaderRead, published_reads};

/// One call site of a presented body: the member the call reaches, on the class it names, at the
/// bytecode index of the call.
///
/// The identity is the presented body's own decode of its `invoke*` — the owner, name and descriptor
/// the class file's constant pool spells — so a candidate is a fact of the run that decided to read
/// it and never a second look at the bytes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CalleeCandidate {
    call_site: u32,
    owner: JvmBytes,
    name: JvmBytes,
    descriptor: JvmBytes,
}

impl CalleeCandidate {
    /// One candidate call site.
    pub fn new(
        call_site: u32,
        owner: impl Into<Vec<u8>>,
        name: impl Into<Vec<u8>>,
        descriptor: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            call_site,
            owner: JvmBytes(owner.into()),
            name: JvmBytes(name.into()),
            descriptor: JvmBytes(descriptor.into()),
        }
    }

    /// The BCI of the call site in the presented body.
    pub fn call_site(&self) -> u32 {
        self.call_site
    }

    /// The owner the call named, in internal form, as the class file spells it.
    pub fn owner(&self) -> &JvmBytes {
        &self.owner
    }

    /// The member name the call named.
    pub fn name(&self) -> &JvmBytes {
        &self.name
    }

    /// The member descriptor the call named.
    pub fn descriptor(&self) -> &JvmBytes {
        &self.descriptor
    }
}

/// The members to read, and the one class definition they may come from.
#[derive(Clone, Debug)]
pub struct CalleeReadRequest<'a> {
    environment: &'a ResolutionEnvironment,
    definition: &'a PhysicalDefinitionId,
    candidates: &'a [CalleeCandidate],
}

impl<'a> CalleeReadRequest<'a> {
    /// One read of the members `candidates` name, from `definition` alone.
    pub fn new(
        environment: &'a ResolutionEnvironment,
        definition: &'a PhysicalDefinitionId,
        candidates: &'a [CalleeCandidate],
    ) -> Self {
        Self {
            environment,
            definition,
            candidates,
        }
    }
}

/// The decoded body of one member this read produced.
///
/// The caller that decides a shape from this member reads the facts through
/// [`CalleeBody::facts`]. The report's own document states it as the two numbers that describe the
/// body — how many instructions it holds and how many code bytes — because no product schema of this
/// engine restates a decode: the analysis report publishes a body's coverage and its state, not its
/// instructions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CalleeBody {
    instructions: usize,
    code_bytes: usize,
    #[serde(skip)]
    facts: MethodCodeFacts,
}

impl CalleeBody {
    fn new(facts: MethodCodeFacts) -> Self {
        Self {
            instructions: facts.instructions.len(),
            code_bytes: usize::try_from(facts.code_span.length).unwrap_or(usize::MAX),
            facts,
        }
    }

    /// The decoded body, as the read of the class file produced it.
    pub fn facts(&self) -> &MethodCodeFacts {
        &self.facts
    }

    /// The same body by value, for the caller that hands it on.
    pub fn into_facts(self) -> MethodCodeFacts {
        self.facts
    }
}

/// One member of the read class, as its own declaration states it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CalleeMember {
    identity: PhysicalMethodId,
    access_flags: u16,
    body: Option<CalleeBody>,
}

impl CalleeMember {
    /// The member's physical identity: the definition it was read from, and its name and descriptor.
    pub fn identity(&self) -> &PhysicalMethodId {
        &self.identity
    }

    /// The member's access flags, as the class declares them.
    pub fn access_flags(&self) -> u16 {
        self.access_flags
    }

    /// The member's decoded body, or `None` when the class declares it without one (abstract or
    /// native): the declaration is evidence, and no `Code` attribute was there to read. Such a
    /// member costs no `MethodBodies` attempt.
    pub fn body(&self) -> Option<&CalleeBody> {
        self.body.as_ref()
    }
}

/// One candidate this read could not answer as a member of the class it read.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CalleeRefusal {
    candidate: CalleeCandidate,
    code: &'static str,
    message: String,
}

impl CalleeRefusal {
    /// The call site this refusal is about.
    pub fn candidate(&self) -> &CalleeCandidate {
        &self.candidate
    }

    /// The refusal's stable code.
    pub fn code(&self) -> &'static str {
        self.code
    }

    /// What could not be read, in one sentence.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// What one callee read produced: the class it read, the members it found, the candidates it
/// refused, the read it performed and what it charged.
#[derive(Clone, Debug, Serialize)]
pub struct CalleeReadReport {
    class: String,
    members: Vec<CalleeMember>,
    refusals: Vec<CalleeRefusal>,
    reads: Vec<HeaderRead>,
    usage: UsageSnapshot,
}

impl CalleeReadReport {
    /// The name the definition declares for itself (`this_class`), in internal form: the class every
    /// member of this report is of, and the name a call site's owner has to be for a member of it to
    /// be the callee that call reaches.
    pub fn class(&self) -> &str {
        &self.class
    }

    /// Every member the class declares among the candidates, in candidate order.
    pub fn members(&self) -> &[CalleeMember] {
        &self.members
    }

    /// Every candidate this read could not answer, in candidate order.
    pub fn refusals(&self) -> &[CalleeRefusal] {
        &self.refusals
    }

    /// The header this read performed, with the reason it was performed under.
    pub fn reads(&self) -> &[HeaderRead] {
        &self.reads
    }

    /// What this read charged, at the moment it finished.
    pub fn usage(&self) -> &UsageSnapshot {
        &self.usage
    }

    /// The same read without the **read detail** the `ReadDetails` evidence category publishes:
    /// every member keeps the identity and the flags the class declared, every refusal and the usage
    /// stay, and the decoded bodies and the header-read proofs are dropped (change
    /// `add-demand-driven-core-results`, D3).
    ///
    /// The two halves answer different questions. What stays is the *binding result*: which member
    /// each candidate named resolved to, which candidates this class does not answer, and what the
    /// read charged — the necessary result of a presentation that read a callee at all, and what the
    /// accessor rule decided from. What goes is the *expansion*: the callee facts (the members'
    /// decoded bodies, which the rule read and no longer needs) and the record of the header reads
    /// that produced them.
    ///
    /// The entry that performed the read publishes this projection when the request did not select
    /// `ReadDetails`; the read itself is unchanged, because the accessor rule's decision reads the
    /// bodies either way.
    pub fn without_read_details(mut self) -> Self {
        for member in &mut self.members {
            member.body = None;
        }
        self.reads.clear();
        self
    }

    /// How many read-detail records this report holds: the decoded bodies it kept and the header-read
    /// proofs it recorded.
    ///
    /// The count an entry states for the `ReadDetails` category — zero for a report this projection
    /// produced, and zero for a read that named no member at all, which is a legal empty result.
    pub fn detail_records(&self) -> u64 {
        let bodies = self
            .members
            .iter()
            .filter(|member| member.body.is_some())
            .count();
        u64::try_from(bodies.saturating_add(self.reads.len())).unwrap_or(u64::MAX)
    }
}

/// Reads the members `request.candidates` name out of the one definition the request names.
///
/// The order of the work is the order of the charges: one header read, then one `MethodBodies`
/// attempt per distinct member the class declares with a body, each before the body is decoded. A
/// candidate that repeats a member already read is answered by that member and charges nothing more.
///
/// This is the read for a caller that holds **no** prepared class; one that does — a bulk operation,
/// which prepared the class once for all of its methods — uses
/// [`read_prepared_callees`] and pays no class read at all.
pub fn read_callees(
    content: &[jarde_reader::artifact::ArtifactSnapshot],
    request: &CalleeReadRequest<'_>,
    budget: &mut Budget,
) -> Result<CalleeReadReport> {
    let mut closure = HeaderClosure::new(content, request.environment);
    let read = closure.read_own_definition(
        &request.environment.runtime.load_domain.loader,
        request.definition,
        HeaderDemand::CalleeMemberBody,
        budget,
    );
    // The read is published before its result is read, exactly like the driver's own read: a refusal
    // by the binding check keeps the record of the bytes it was decided on.
    let reads = published_reads(&closure);
    let read = read?;
    let (class, members, refusals) = callee_members(request, CalleeClass::Read(&read), budget)?;
    // The header's constant pool is deliberately not published: the body facts above were decoded
    // against it by this very read, and the caller above holds the pool of the same definition (the
    // presented run's own), so a second copy of it on this report would be one fact stated twice.
    Ok(CalleeReadReport {
        class,
        members,
        refusals,
        reads,
        usage: budget.usage(),
    })
}

/// The same read, from a class the caller already prepared (bulk task 2.3, callee half).
///
/// A bulk operation prepares one class once and analyses every method that class declares; the
/// presented body's accessor candidates name members of that very class (a candidate of another
/// class is refused by this read, as it always was), so the class's member table, its constant pool
/// and the decoder that resolves a body against them are already in the caller's hands. This entry
/// consumes them:
///
/// * **no second read of the definition and no `ClassHeaders` charge for it**: the members come
///   from the prepared member table
///   and the bodies from [`PreparedClass::method_code`], the same decoder
///   [`jarde_reader::classfile::method_code_facts`] delegates to, so a member body read here and the
///   same body read by [`read_callees`] cannot drift in their facts, their stop position or what
///   they charge;
/// * the **same refusals and the same member order** as [`read_callees`]: a candidate naming another
///   class is `callee_not_this_class`, one the class declares no record for is `callee_not_declared`,
///   and a member the class declares without a `Code` entry is answered as its declaration with no
///   body (and no `MethodBodies` attempt);
/// * the **same charges per member**: one `MethodBodies` attempt per distinct candidate the class
///   declares with a body, before its decode, plus the body's own `AttributeBytes`/`CodeBytes`;
/// * the **same read record**, under [`crate::resolver::ReadReason::CalleeMemberBody`], of the
///   definition the request names — sourced from the preparation instead of a second read of those
///   bytes.
///
/// The prepared class must be the definition the request names (its coordinate and trusted
/// class-bytes identity: [`crate::providers::require_prepared_definition`]) and its member table
/// must have read to its declared end ([`crate::providers::require_prepared_member_table`]; a
/// stopped table cannot state that a class declares no member). The declared loader's binding check
/// is the direct read's own ([`crate::providers::HeaderClosure::bind_definition`]), decided from the
/// prepared facts, so a prepared input is not a way around it.
pub fn read_prepared_callees(
    content: &[jarde_reader::artifact::ArtifactSnapshot],
    prepared: &PreparedClass<'_>,
    request: &CalleeReadRequest<'_>,
    budget: &mut Budget,
) -> Result<CalleeReadReport> {
    crate::providers::require_prepared_definition(prepared, request.definition)?;
    let facts = prepared.facts_handle();
    let mut closure = HeaderClosure::new(content, request.environment);
    let bound = closure.bind_definition(
        &request.environment.runtime.load_domain.loader,
        request.definition,
        facts,
        HeaderDemand::CalleeMemberBody,
        budget,
    );
    let reads = published_reads(&closure);
    bound?;
    crate::providers::require_prepared_member_table(prepared)?;
    let (class, members, refusals) =
        callee_members(request, CalleeClass::Prepared(prepared), budget)?;
    Ok(CalleeReadReport {
        class,
        members,
        refusals,
        reads,
        usage: budget.usage(),
    })
}

/// The class one callee read answers from.
///
/// Both sources answer the same three questions — the name the class declares for itself, which
/// member record declares a raw name and descriptor, and how that record's body is decoded — so the
/// candidate loop ([`callee_members`]) is written once and the two reads differ only here. The
/// refusals, the candidate order and the deduplication that the report publishes are the loop's, not
/// a source's.
enum CalleeClass<'a> {
    /// The header read this entry performed itself: the bytes and member records of that read.
    Read(&'a DefinitionContent),
    /// The class the caller already prepared: its shared facts, locator and decoder.
    Prepared(&'a PreparedClass<'a>),
}

/// One member record a callee read located, with the coordinate its decode is asked with.
#[derive(Clone, Copy)]
struct CalleeRecord<'a> {
    /// The record's own declaration, as its class states it (attribute shells only, no body).
    record: &'a MemberHeader,
    /// The record's ordinal inside its class's declaration order: the prepared locator's own ordinal
    /// for a prepared class, and the position in the member table the read produced for one read
    /// here — the same coordinate, because a class file's method table *is* its declaration order.
    ordinal: MethodOrdinal,
}

impl CalleeClass<'_> {
    /// The class's own internal name (`this_class`), lossily spelled as this report states it.
    fn class_name(&self) -> String {
        let declared = match self {
            Self::Read(read) => read.header.facts.this_class.raw(),
            Self::Prepared(prepared) => prepared.class_facts().this_class.raw(),
        };
        String::from_utf8_lossy(&declared.0).into_owned()
    }

    /// The first member record declaring this raw name and descriptor, in declaration order.
    ///
    /// The first record is the one a caller that names a member gets: two records under one
    /// signature stay two declarations in the class, and which of them this read answers with is
    /// decided by the body read's own ambiguity rule (the shared decoder refuses both when it is
    /// asked for a body) rather than by an election here.
    fn locate(&self, name: &[u8], descriptor: &[u8]) -> Option<CalleeRecord<'_>> {
        match self {
            Self::Read(read) => {
                let position = read.header.facts.methods.iter().position(|member| {
                    member.name.raw().0 == name && member.descriptor.raw().0 == descriptor
                })?;
                Some(CalleeRecord {
                    record: &read.header.facts.methods[position],
                    ordinal: MethodOrdinal(u32::try_from(position).unwrap_or(u32::MAX)),
                })
            }
            Self::Prepared(prepared) => {
                let ordinal = *prepared.locate_method(name, descriptor).first()?;
                let slot = prepared.slot(ordinal)?;
                Some(CalleeRecord {
                    record: &slot.header,
                    ordinal,
                })
            }
        }
    }

    /// Decodes one located body, charging the `MethodBodies` attempt **before** the decode.
    ///
    /// The bytes are the ones the class was read from, the pool is that same read's, and the decode
    /// is the one implementation both sources go through, so a body this entry answers with is the
    /// body the single-method path publishes for it — facts, stop position and charges alike.
    fn decode(&self, located: CalleeRecord<'_>, budget: &mut Budget) -> Result<MethodCodeFacts> {
        budget.charge(CountedBudgetDimension::MethodBodies, 1)?;
        match self {
            Self::Read(read) => jarde_reader::classfile::method_code_facts(
                read.read.bytes(),
                located.record,
                budget,
            ),
            Self::Prepared(prepared) => prepared.method_code(located.ordinal, budget),
        }
    }
}

/// The members `request.candidates` name, read from one class in candidate order.
///
/// The whole policy of one callee read lives here, once, whatever the class was read from: a
/// candidate whose owner is not the name this class declares for itself is refused
/// (`callee_not_this_class`) rather than looked up in these bytes; a repeated candidate is answered
/// by the member the first one read; a candidate the class declares no record for is refused
/// (`callee_not_declared`); a record the class declares without a `Code` entry is answered as its own
/// declaration with no body and no body attempt; and every other candidate costs one `MethodBodies`
/// attempt before its decode.
fn callee_members(
    request: &CalleeReadRequest<'_>,
    class: CalleeClass<'_>,
    budget: &mut Budget,
) -> Result<(String, Vec<CalleeMember>, Vec<CalleeRefusal>)> {
    let class_name = class.class_name();
    let mut members: Vec<CalleeMember> = Vec::new();
    let mut refusals: Vec<CalleeRefusal> = Vec::new();
    let mut seen: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for candidate in request.candidates {
        let owner = String::from_utf8_lossy(&candidate.owner.0).into_owned();
        if owner != class_name {
            refusals.push(CalleeRefusal {
                candidate: candidate.clone(),
                code: "callee_not_this_class",
                message: format!(
                    "the call site at BCI {} names `{owner}.{}`, and this read is of the definition that declares `{class_name}`: a member of another class is not read from these bytes, however its name reads",
                    candidate.call_site,
                    String::from_utf8_lossy(&candidate.name.0)
                ),
            });
            continue;
        }
        let key = (candidate.name.0.clone(), candidate.descriptor.0.clone());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let Some(located) = class.locate(&candidate.name.0, &candidate.descriptor.0) else {
            refusals.push(CalleeRefusal {
                candidate: candidate.clone(),
                code: "callee_not_declared",
                message: format!(
                    "the call site at BCI {} names `{class_name}.{}` `{}`, and the class declares no such member of its own: the call reaches a member this run cannot read",
                    candidate.call_site,
                    String::from_utf8_lossy(&candidate.name.0),
                    String::from_utf8_lossy(&candidate.descriptor.0)
                ),
            });
            continue;
        };
        let identity = PhysicalMethodId {
            owner: request.definition.clone(),
            name: located.record.name.raw().clone(),
            descriptor: located.record.descriptor.raw().clone(),
        };
        if !crate::engine::has_code_attribute(located.record) {
            // A member the class declares without a body is answered as itself: the declaration and
            // its flags are evidence, and no attempt is charged for a body that is not there.
            members.push(CalleeMember {
                identity,
                access_flags: located.record.access_flags,
                body: None,
            });
            continue;
        }
        let facts = class.decode(located, budget)?;
        members.push(CalleeMember {
            identity,
            access_flags: located.record.access_flags,
            body: Some(CalleeBody::new(facts)),
        });
    }
    Ok((class_name, members, refusals))
}
