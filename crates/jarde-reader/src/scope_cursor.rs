//! Incremental physical traversal: one class candidate at a time, in the scope's own order.
//!
//! A bulk operation dispatches classes a window at a time, so discovery must be able to hand over
//! one class at a time rather than build a whole package's class list, read every class header or
//! decode every method first (change `add-parallel-bulk-recovery`, decision 2). [`ScopeCursor`] is
//! that handover: it walks the containers a scope holds depth first, yields the class candidates of
//! each container in central-directory order, and is pulled by the caller.
//!
//! # Order
//!
//! Containers are walked in central-directory entry order, a child container is visited at its own
//! entry (depth first), and a container's entries are visited in declaration order. A standalone
//! `CLASS` snapshot yields exactly one item — its own root — because that is the one candidate a
//! snapshot with no containers has.
//!
//! # What it does not build
//!
//! The cursor reads one container's directory at a time, through the same directed access every
//! other lookup uses, and never collects the classes it found: there is no package-wide class list,
//! no batch of class headers and no `MethodIr` anywhere on this path. The directory of the container
//! being walked is the one product the walk holds, and it is the same verified container product
//! every other read of that container shares.
//!
//! # Charging
//!
//! The account is the one `ArtifactSnapshot::enumerate_artifact_tree` keeps: one `ArchiveEntries`
//! charge per entry the directory parse examines, `check_nested_depth` per container the cursor
//! descends into (the depth check the ancestor walk applies), and the container reads' own charges.
//!
//! # Failure, cancellation and what "complete" means
//!
//! * The **root** container failing is an `Err`: there is no scope to walk and no prefix to report.
//! * A **child** container failing records a [`Diagnostic`] carrying the physical position of the
//!   entry it hangs from and leaves that subtree **unknown** — not walked, not counted, and never
//!   reported as a complete denominator ([`ScopeCursor::coverage_state`] states it, and the caller
//!   must not treat the items it got as the whole scope).
//! * The **request's own cancellation is observed before every item**, the standalone root and the
//!   first entry of a container included: [`Budget::poll`] runs before the step is looked at, so a
//!   cancelled request yields no candidate at all and the first `next_class` answers
//!   [`Error::Cancelled`]. A budget stop or a cancellation ends the walk with `Err` as soon as it is
//!   observed; the cursor then stays stopped, so a later call yields nothing.
//! * [`ScopeCursor::coverage_state`] is `CompleteWithinSchema` only once the walk really reached the
//!   **end of the scope** with nothing left unknown. A cursor that has yielded a prefix of a scope it
//!   has not exhausted reports `Partial`, because "the denominator is not known yet" is exactly what a
//!   progressive consumer has to be able to read from the cursor itself rather than infer from a
//!   caller's own bookkeeping.

use crate::artifact::{
    ArtifactKind, ArtifactSnapshot, ContainerFacts, ContainerFactsHandle, NestedArchiveState,
    child_container_facts, root_origin, tree_diagnostic,
};
use crate::budget::{Budget, BudgetDimension};
use crate::error::{Error, Result};
use crate::model::{CoverageState, Diagnostic, PhysicalClassLocation, PhysicalEntryId, SnapshotId};
use crate::view::PhysicalScope;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// The raw-name rule a class candidate is selected by: a case-sensitive `.class` suffix.
///
/// The rule decides *candidacy* on the entry's own bytes and derives nothing: an entry that matches
/// it has not been shown to hold a class, and the read that follows is what turns it into a
/// declaration. It is byte-for-byte the rule the facade's class listing and the query layer's class
/// scan apply, so one entry is one candidate wherever it is listed.
const CLASS_FILE_SUFFIX: &[u8] = b".class";

/// Whether one raw archive name is a class candidate by that rule.
///
/// This is the one spelling of the rule above, published so that a walk which yields *every* entry
/// (not only the class candidates) classifies them by the same bytes the class-only walk filters
/// them with. It is a pure name predicate: it reads no record and claims nothing about the entry's
/// content.
pub fn is_class_candidate_name(raw_name: &[u8]) -> bool {
    raw_name.ends_with(CLASS_FILE_SUFFIX)
}

/// One class candidate the cursor yielded.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScopeClass {
    /// Where the candidate is: an archive entry, or the root of a standalone snapshot.
    pub location: PhysicalClassLocation,
    /// The entry the candidate is, or `None` for a standalone snapshot's root.
    pub entry: Option<PhysicalEntryId>,
    /// How deep the container holding it is: 0 for a snapshot's root, 1 for a direct child, …
    pub depth: u64,
}

/// One container the cursor is walking, in physical order.
struct ContainerWalk {
    facts: Arc<ContainerFacts>,
    /// The next entry this walk examines.
    position: usize,
}

/// One scope walked incrementally, one class candidate at a time.
///
/// The cursor is created by [`ArtifactSnapshot::scope_cursor`] and is pulled with
/// [`ScopeCursor::next_class`] until it answers `Ok(None)`. Once it has stopped — because the walk
/// finished, or because a failure ended it — later calls keep answering `Ok(None)`.
pub struct ScopeCursor {
    snapshot: ArtifactSnapshot,
    /// Whether the scope descends into nested containers: the artifact-tree scope does, the
    /// whole-snapshot scope is the root container's own entries.
    descend: bool,
    /// The stack of containers being walked, outermost first; the last one is the current one.
    stack: Vec<ContainerWalk>,
    /// The one item a standalone `CLASS` snapshot yields, until it has been yielded.
    standalone_pending: bool,
    /// Whether the root container has been opened already.
    started: bool,
    /// Every failure that ended a subtree, with the physical position of the entry it hangs from.
    diagnostics: Vec<Diagnostic>,
    /// Whether every container the scope holds was walked *and* nothing of it was left unknown.
    complete: bool,
    /// Whether the walk reached the end of the scope: the standalone root was yielded and the scope
    /// found empty behind it, or the last container was left behind.
    ///
    /// A prefix of a scope that has not been exhausted cannot report a complete denominator, so this
    /// flag is what [`ScopeCursor::coverage_state`] requires beside [`Self::complete`]. It is never
    /// set for a walk that stopped — a stop leaves the scope unexhausted, which is the honest state.
    exhausted: bool,
    /// Whether the walk has stopped; a stopped cursor yields nothing further.
    stopped: bool,
}

impl ScopeCursor {
    /// Opens one scope for incremental traversal.
    ///
    /// The scope's *shape* is checked here, before any read: a standalone `CLASS` snapshot has no
    /// containers, so an artifact-tree scope on one is an input error, and an artifact-tree scope
    /// whose root container is not this snapshot's own root is one too. Opening reads nothing — the
    /// root container is opened by the first [`ScopeCursor::next_class`].
    pub(crate) fn new(snapshot: ArtifactSnapshot, scope: &PhysicalScope) -> Result<Self> {
        let root_container = root_origin(snapshot.id()).root_container;
        let (descend, standalone_pending) = match (snapshot.kind(), scope) {
            (ArtifactKind::StandaloneClass, PhysicalScope::SnapshotAll) => (false, true),
            (ArtifactKind::StandaloneClass, PhysicalScope::ArtifactTree { .. }) => {
                return Err(Error::invalid_input(
                    "navigation_not_zip",
                    "an artifact-tree scope requires a ZIP snapshot; a standalone CLASS snapshot has no containers",
                ));
            }
            (ArtifactKind::Zip, PhysicalScope::SnapshotAll) => (false, false),
            (
                ArtifactKind::Zip,
                PhysicalScope::ArtifactTree {
                    root_container: named,
                },
            ) => {
                if *named != root_container {
                    return Err(Error::invalid_input(
                        "navigation_root_container_mismatch",
                        "the tree scope's root container is not this snapshot's root container",
                    ));
                }
                (true, false)
            }
        };
        Ok(Self {
            snapshot,
            descend,
            stack: Vec::new(),
            standalone_pending,
            started: false,
            diagnostics: Vec::new(),
            complete: true,
            exhausted: false,
            stopped: false,
        })
    }

    /// The snapshot this cursor walks.
    pub fn snapshot(&self) -> &SnapshotId {
        self.snapshot.id()
    }

    /// The verified facts of the container the cursor is currently walking, as an **active handle**
    /// the caller may keep.
    ///
    /// The walk holds one container at a time — the one whose entries it is yielding — and this hands
    /// that product out as the same strong reference the walk itself keeps
    /// ([`crate::artifact::ContainerFactsHandle`]). A caller that dispatches the candidate elsewhere
    /// (a class task that runs after the cursor has moved on) holds this handle for as long as it
    /// needs the container, and every read of that container — the class entry's own verified read
    /// and the loader binding query of a method request — is then answered from the same product
    /// instead of parsing the directory again, whatever the caller's facts cache is doing.
    ///
    /// `None` two ways, and both are "this walk has no container facts to hand over": the cursor has
    /// not opened a container yet (`next_class` opens the root on its first call), and a standalone
    /// `CLASS` snapshot has no container at all.
    pub fn container_facts(&self) -> Option<ContainerFactsHandle> {
        self.stack.last().map(|walk| ContainerFactsHandle {
            facts: Arc::clone(&walk.facts),
        })
    }

    /// Every failure that ended a subtree, each carrying the physical position it hangs from.
    ///
    /// A diagnostic here means the walk could not read a container the scope holds, so the classes
    /// it would have held are **unknown**: the caller must not report the items it received as a
    /// complete denominator.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Whether every container this scope holds was walked, to the end.
    ///
    /// `CompleteWithinSchema` is the walk reaching the **end of the scope** with no subtree left
    /// unread; `Partial` is everything else, and there are three ways to be there: the walk has not
    /// gone far enough yet (a cursor that yielded one candidate of a scope it has not exhausted), a
    /// subtree was skipped or a failure ended the walk, and the standalone root before the scope was
    /// found empty behind it. It is the state a caller has to check before treating its item count as
    /// a scope-wide denominator.
    pub fn coverage_state(&self) -> CoverageState {
        if self.exhausted && self.complete {
            CoverageState::CompleteWithinSchema
        } else {
            CoverageState::Partial
        }
    }

    /// The next class candidate in the scope's own physical order, or `None` when the walk is over.
    ///
    /// See the module documentation for the order, the charging and what a failure means.
    pub fn next_class(&mut self, budget: &mut Budget) -> Result<Option<ScopeClass>> {
        if self.stopped {
            return Ok(None);
        }
        match self.advance(budget) {
            Ok(item) => Ok(item),
            // A stop ends the walk for good: the items already yielded are a prefix, and no later
            // call continues a walk that was refused, cancelled or run out of budget.
            Err(error) => {
                self.complete = false;
                self.stopped = true;
                Err(error)
            }
        }
    }

    /// One step of the walk: the next candidate, or `None` when the scope is exhausted.
    ///
    /// The request is polled **before** the step is looked at, so every item — the standalone root
    /// and the first entry of a container included — is subject to the cancellation and the deadline
    /// the caller's budget already holds. Reaching the end of the scope is what sets
    /// [`ScopeCursor::coverage_state`]'s completeness; a stop or a failure leaves it unexhausted.
    fn advance(&mut self, budget: &mut Budget) -> Result<Option<ScopeClass>> {
        budget.poll()?;
        if self.standalone_pending {
            self.standalone_pending = false;
            return Ok(Some(ScopeClass {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: self.snapshot.id().clone(),
                },
                entry: None,
                depth: 0,
            }));
        }
        if !self.started {
            self.started = true;
            if self.snapshot.kind() != ArtifactKind::Zip {
                // A standalone CLASS snapshot holds exactly its root, which the arm above yielded:
                // the scope is exhausted behind the one item it has.
                self.exhausted = true;
                return Ok(None);
            }
            // The root container's own failure is an `Err`: the scope has no readable prefix at
            // all, and the directed access refuses an incomplete directory rather than reporting a
            // prefix as the container's contents.
            let root = root_origin(self.snapshot.id());
            let facts = self.snapshot.container_facts(&root, budget)?;
            self.snapshot.hold_container_facts(&facts);
            self.stack.push(ContainerWalk { facts, position: 0 });
        }
        loop {
            // Cancellation is observed at every entry: the poll happens before the entry at the
            // cursor is even looked at, so a cancelled request neither examines nor yields it.
            budget.poll()?;
            let Some(index) = self.stack.len().checked_sub(1) else {
                // The outermost container was left behind and nothing is above it: the walk reached
                // the end of the scope, and only now can it state a complete denominator.
                self.exhausted = true;
                return Ok(None);
            };
            let position = self.stack[index].position;
            // The step is decided from an immutable borrow of the container at the cursor, so it
            // ends before anything is pushed, popped or recorded.
            let step = step_of(&self.stack[index], position, self.descend);
            match step {
                CursorStep::Exhausted => {
                    self.stack.pop();
                }
                CursorStep::Skip => {
                    self.stack[index].position = position + 1;
                }
                CursorStep::Yield => {
                    self.stack[index].position = position + 1;
                    let class = class_candidate(&self.stack[index], position)?;
                    return Ok(Some(class));
                }
                CursorStep::Descend => {
                    self.stack[index].position = position + 1;
                    // The child is reached from the parent this walk already holds, so the parent's
                    // complete directory is parsed once for this request rather than once per
                    // container that hangs from it.
                    let child = child_container_facts(&self.stack[index].facts, position, budget);
                    match child {
                        Ok(facts) => {
                            // The walk keeps the child's verified facts for as long as it is inside
                            // that container, so a consumer of this walk — a class read, a binding
                            // query — reaches the same product by origin instead of parsing the
                            // directory again.
                            self.snapshot.hold_container_facts(&facts);
                            self.stack.push(ContainerWalk { facts, position: 0 })
                        }
                        Err(error) if walk_stops(&error) => return Err(error),
                        Err(error) => {
                            // The subtree is left unknown and named where it hangs from; the rest
                            // of the scope keeps being walked, exactly like the tree walk's own
                            // record of a container it could not read.
                            self.complete = false;
                            let parent = self.stack[index].facts.entries().get(position);
                            self.diagnostics.push(tree_diagnostic(&error, parent));
                        }
                    }
                }
            }
        }
    }
}

/// What one examined entry does next.
enum CursorStep {
    /// The container has no entry left at the cursor.
    Exhausted,
    /// An entry the scope does not present as a class candidate.
    Skip,
    /// A class candidate to yield.
    Yield,
    /// A nested archive candidate the scope descends into.
    Descend,
}

/// Classifies the entry at `position` of one container.
///
/// The rule is physical and total: an entry is either a nested archive candidate the scope descends
/// into (the artifact-tree scope only — the whole-snapshot scope is the root container's own
/// entries), a class candidate (a case-sensitive `.class` raw name), or neither. Nothing here reads
/// the entry's bytes: candidacy is a claim about a name, and the class read that follows is what
/// turns it into a declaration.
fn step_of(walk: &ContainerWalk, position: usize, descend: bool) -> CursorStep {
    match walk.facts.entries().get(position) {
        None => CursorStep::Exhausted,
        Some(record)
            if descend && record.nested_archive == NestedArchiveState::CandidateNotScanned =>
        {
            CursorStep::Descend
        }
        Some(record) if is_class_candidate_name(&record.id.raw_name.0) => CursorStep::Yield,
        Some(_) => CursorStep::Skip,
    }
}

/// The one class candidate at `position` of one container, as the walk hands it over.
fn class_candidate(walk: &ContainerWalk, position: usize) -> Result<ScopeClass> {
    let record = walk.facts.entries().get(position).ok_or_else(|| {
        Error::invalid_input(
            "entry_not_found",
            "entry ordinal is absent from the container",
        )
    })?;
    let depth = u64::try_from(walk.facts.origin().steps.len()).map_err(|_| {
        Error::invalid_input("nested_depth_overflow", "container depth exceeds u64")
    })?;
    Ok(ScopeClass {
        location: PhysicalClassLocation::ArchiveEntry {
            entry: record.id.clone(),
        },
        entry: Some(record.id.clone()),
        depth,
    })
}

/// Whether one failure ends the whole walk rather than one subtree.
///
/// The rule is the artifact-tree walk's own (`tree_must_stop`) with one addition: a cancellation, an
/// exhausted budget other than the nested-depth dimension and an I/O failure end the walk — the
/// first two because no read may start after a stop, the third because an infrastructure failure is
/// not damage in one container — while structure damage (an unreadable directory, an entry that is
/// not an archive) and a refused nested depth leave that subtree unread and the walk running.
///
/// The entry walk ([`crate::entry_cursor::EntryCursor`]) applies the same rule, so a subtree that
/// could not be read has one meaning wherever a walk reports it.
pub(crate) fn walk_stops(error: &Error) -> bool {
    match error {
        Error::Cancelled { .. } | Error::Io { .. } => true,
        Error::BudgetExceeded { dimension, .. } => *dimension != BudgetDimension::NestedDepth,
        Error::InvalidInput { .. } | Error::Unsupported { .. } => false,
    }
}
