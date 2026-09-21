//! Incremental entry traversal: every entry of a scope, one at a time, classified.
//!
//! [`ScopeCursor`](crate::scope_cursor::ScopeCursor) hands over the **class candidates** of a
//! scope, which is what a bulk class operation dispatches. A structural query needs the entries a
//! class-only walk filters out as well — a `META-INF/MANIFEST.MF` or a
//! `META-INF/services/…` registration is a resource fact of its own — so this cursor is the same
//! traversal with the filter removed: it yields one entry per pull, in the scope's own physical
//! order, with the classification the caller needs and the container's own verified record.
//!
//! # Order
//!
//! Containers are walked depth first, in central-directory entry order, and a container's entries
//! are yielded in ordinal order before the walk descends into the nested archives that container
//! holds — the order the artifact-tree report enumerates in
//! ([`ArtifactSnapshot::enumerate_artifact_tree`](crate::artifact::ArtifactSnapshot::enumerate_artifact_tree)),
//! so the two orders are comparable entry by entry. The whole-snapshot scope is the root
//! container's own entries: it descends into nothing, and a standalone `CLASS` snapshot has no
//! container to walk at all (that root is the caller's own unit, which it already holds).
//!
//! # What one pull pays for
//!
//! One container's verified facts are the product the directed access always builds — its complete
//! central directory, validated record by record, with the same charges, the same duplicate
//! reporting and the same refusal of an incomplete parse (see
//! [`ArtifactSnapshot`](crate::artifact::ArtifactSnapshot)'s container access) — and they are built
//! once when the walk reaches that container, never for a container behind the caller's stop. They
//! are built **for this walk**: the validation is the work this walk reports, so
//! [`ArtifactSnapshot::walked_container_facts`](crate::artifact::ArtifactSnapshot::walked_container_facts)
//! states why no retention answers it. A nested archive is materialized (decompressed, CRC-checked)
//! only when the walk really *descends* into it, and an entry's own bytes are read only by the
//! caller that consumes that entry. What a pull therefore makes necessary is the directory
//! validation of the containers it reached and nothing else: the containers behind the last pull
//! were never opened, their records were never validated, and their nested archives were never
//! expanded.
//!
//! # Failure, cancellation and completeness
//!
//! * The **root** container failing is an `Err`: there is no scope to walk and no prefix to report.
//! * A **child** container failing (an entry that is not an archive, a directory that does not parse
//!   completely, a refused nested depth) records a [`Diagnostic`] carrying the physical position of
//!   the entry it hangs from and leaves that subtree **unknown** — not walked, not counted, and
//!   never reported as a complete denominator ([`EntryCursor::coverage_state`] states it).
//! * The request's own cancellation is observed before every entry, the first pull included, so a
//!   cancelled request yields no entry at all. A stop ends the walk for good.
//! * [`EntryCursor::coverage_state`] is `CompleteWithinSchema` only once the walk reached the end of
//!   the scope with nothing left unknown. A prefix of a scope that was not exhausted reports
//!   `Partial`, which is what a progressive consumer reads to know that the denominator is not
//!   established yet.

use crate::artifact::{
    ArtifactKind, ArtifactSnapshot, ContainerFacts, NestedArchiveState, PhysicalEntry,
    child_container_facts, derive_child_origin, root_origin, tree_diagnostic,
};
use crate::budget::Budget;
use crate::error::{Error, Result};
use crate::model::{
    ContainerOrigin, CoverageState, Diagnostic, DiagnosticSeverity, ExecutionReport, SnapshotId,
};
use crate::scope_cursor::{is_class_candidate_name, walk_stops};
use crate::view::PhysicalScope;
use std::sync::Arc;

/// What one entry is for a walk over a scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeEntryKind {
    /// The raw name is a class candidate, by the rule the class-only walk filters with
    /// ([`is_class_candidate_name`]). Candidacy is a claim about the name, never about the bytes.
    ClassCandidate,
    /// The record marks a nested archive candidate: the entry a walk descends through when the
    /// scope holds nested containers. It is a kind of its own because it is the one entry whose
    /// *subtree* belongs to the walk, and a consumer that filtered only by the class rule would
    /// otherwise read it as an ordinary resource.
    NestedContainer,
    /// Every other entry: a resource, a directory record, a signature file, …
    ///
    /// The kind states what the entry is not; it never claims the entry holds text, or that its
    /// bytes parse as anything.
    Other,
}

/// One entry the walk yielded, with the container's own verified record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeEntry {
    /// The record this container's complete central directory parsed for this entry, at
    /// [`PhysicalEntry::id`]'s ordinal and raw name. It is the authoritative record, not a
    /// caller-supplied one: the walk looked it up in the directory it just validated.
    pub entry: PhysicalEntry,
    /// How deep the container holding it is: 0 for a snapshot's root container, 1 for a direct
    /// child, …
    pub depth: u64,
    /// What the entry is for this walk.
    pub kind: ScopeEntryKind,
}

/// One container this walk knows about, with what it established for it.
///
/// The count is the denominator a caller may state for that container, and it comes either from the
/// container's own directory (which the walk validated) or from the same directory's declaration
/// (which the walk reached but could not open): it is known exactly for every container the walk
/// knows about — and absent, not zero, for one it never reached at all.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeContainer {
    pub origin: ContainerOrigin,
    /// Entries the container's own directory declared.
    pub entries: u64,
    /// Whether the walk opened this container.
    ///
    /// An opened container's directory was validated in full — every record's ordinal, layout and
    /// cross-check is that validation's product — so a range covering it states work this walk
    /// really did. A container this walk stopped before opening was never examined at all: its
    /// count is what its own directory declares, and every entry of it stays named as work *not*
    /// done.
    pub walked: bool,
}

/// One container the walk is inside, in physical order.
struct ContainerWalk {
    facts: Arc<ContainerFacts>,
    /// The next entry of this container the walk yields.
    position: usize,
    /// The next nested-archive candidate of this container the walk descends through.
    descend_from: usize,
}

/// The entries of one scope, pulled one at a time.
///
/// Created by [`ArtifactSnapshot::entry_cursor`] and pulled with [`EntryCursor::next_entry`] until
/// it answers `Ok(None)`. Once it has stopped — because the walk finished, or because a failure
/// ended it — later calls keep answering `Ok(None)`.
pub struct EntryCursor {
    snapshot: ArtifactSnapshot,
    /// Whether the scope descends into nested containers: the artifact-tree scope does, the
    /// whole-snapshot scope is the root container's own entries.
    descend: bool,
    /// The stack of containers being walked, outermost first; the last one is the current one.
    stack: Vec<ContainerWalk>,
    /// Every container this walk knows about, in the order it reached them.
    containers: Vec<ScopeContainer>,
    /// Every failure that ended a subtree, with the physical position of the entry it hangs from.
    diagnostics: Vec<Diagnostic>,
    /// The state the first subtree failure means, in the tree walk's own vocabulary.
    ///
    /// A subtree that could not be read does not stop the walk, so this is not the walk's own
    /// ending: it is the bound the caller reports beside the entries it did receive.
    issue: Option<ExecutionReport>,
    /// Whether every container the scope holds was walked *and* nothing was left unknown.
    complete: bool,
    /// Whether the walk reached the end of the scope.
    exhausted: bool,
    /// Whether the walk has stopped; a stopped cursor yields nothing further.
    stopped: bool,
}

impl ArtifactSnapshot {
    /// Opens one scope for incremental entry traversal.
    ///
    /// The scope's shape is checked here, before any read: a standalone `CLASS` snapshot has no
    /// container, so there is no entry to walk (its root is the caller's own unit), and an
    /// artifact-tree scope whose root container is not this snapshot's own root is refused.
    /// Opening reads nothing — the root container is opened by the first [`EntryCursor::next_entry`].
    pub fn entry_cursor(&self, scope: &PhysicalScope) -> Result<EntryCursor> {
        EntryCursor::new(self.clone(), scope)
    }
}

impl EntryCursor {
    pub(crate) fn new(snapshot: ArtifactSnapshot, scope: &PhysicalScope) -> Result<Self> {
        let root_container = root_origin(snapshot.id()).root_container;
        let descend = match (snapshot.kind(), scope) {
            (ArtifactKind::StandaloneClass, _) => {
                return Err(Error::invalid_input(
                    "navigation_not_zip",
                    "entry traversal requires a ZIP snapshot; a standalone CLASS snapshot holds no container",
                ));
            }
            (ArtifactKind::Zip, PhysicalScope::SnapshotAll) => false,
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
                true
            }
        };
        Ok(Self {
            snapshot,
            descend,
            stack: Vec::new(),
            containers: Vec::new(),
            diagnostics: Vec::new(),
            issue: None,
            complete: true,
            exhausted: false,
            stopped: false,
        })
    }

    /// The snapshot this cursor walks.
    pub fn snapshot(&self) -> &SnapshotId {
        self.snapshot.id()
    }

    /// Every container this walk knows about, in the order it reached it.
    ///
    /// A container the walk never reached is absent: its entry count was never established, so this
    /// never states a denominator for it. A container the walk reached but could not open is
    /// present with `walked` false — its own directory declares how many entries it holds, and that
    /// declaration is the range this invocation did *not* examine.
    pub fn containers(&self) -> &[ScopeContainer] {
        &self.containers
    }

    /// Every failure that ended a subtree, each carrying the physical position it hangs from.
    ///
    /// A diagnostic here means the walk could not read a container the scope holds, so the entries
    /// it would have held are **unknown**: the caller must not report the entries it received as a
    /// complete denominator.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// The state the first subtree failure means, or `None` when every container was read.
    ///
    /// A subtree that stayed unknown leaves the walk itself running — the entries after it are
    /// still yielded — so this is the bound a caller reports *beside* its entries (a `Partial`
    /// execution, in the walk's own vocabulary), never the walk's own ending state. It is folded
    /// from the same function the artifact-tree walk uses, so one failure has one spelling.
    pub fn issue(&self) -> Option<&ExecutionReport> {
        self.issue.as_ref()
    }

    /// Whether the walk reached the end of the scope with nothing left unknown.
    ///
    /// `CompleteWithinSchema` is every container walked to its end and no subtree left unread;
    /// `Partial` is everything else — a walk that has not gone far enough yet, a subtree that
    /// failed, or a stop.
    pub fn coverage_state(&self) -> CoverageState {
        if self.exhausted && self.complete {
            CoverageState::CompleteWithinSchema
        } else {
            CoverageState::Partial
        }
    }

    /// The next entry in the scope's own physical order, or `None` when the walk is over.
    ///
    /// See the module documentation for the order, what one pull pays for, and what a failure
    /// means.
    pub fn next_entry(&mut self, budget: &mut Budget) -> Result<Option<ScopeEntry>> {
        if self.stopped {
            return Ok(None);
        }
        match self.advance(budget) {
            Ok(item) => Ok(item),
            // A stop ends the walk for good: the entries already yielded are a prefix, and no later
            // call continues a walk that was refused, cancelled or run out of budget.
            Err(error) => {
                self.complete = false;
                self.stopped = true;
                self.declare_unopened_root();
                Err(error)
            }
        }
    }

    /// States the range a walk owes when it stopped before it could open the root container.
    ///
    /// The container's own central directory declares how many entries it holds, and that
    /// declaration is a fact about the bytes rather than about this invocation's work — so a walk
    /// that never opened the container still reports the range it did not examine, instead of
    /// leaving a caller to read "no range" as "nothing there". It is recorded as *not* walked, and
    /// deliberately not read through the budget: a stopped request performs no further work, it
    /// only states what the stopped range was. A walk that opened any container has already made
    /// this statement about each one it knows, so there is nothing to add.
    fn declare_unopened_root(&mut self) {
        if !self.containers.is_empty() {
            return;
        }
        let Some(entries) = self.snapshot.declared_root_entries() else {
            return;
        };
        self.containers.push(ScopeContainer {
            origin: root_origin(self.snapshot.id()),
            entries,
            walked: false,
        });
    }

    /// One step of the walk: the next entry, or `None` when the scope is exhausted.
    ///
    /// The request is polled **before** every step is looked at, so every entry — the first entry
    /// of a container and the descent that opens one included — is subject to the cancellation and
    /// the deadline the caller's budget already holds.
    fn advance(&mut self, budget: &mut Budget) -> Result<Option<ScopeEntry>> {
        budget.poll()?;
        if self.stack.is_empty() && self.exhausted {
            return Ok(None);
        }
        if self.stack.is_empty() {
            // The root container is opened by the first pull. Its failure is an `Err`: the scope
            // has no readable prefix at all, and the directed access refuses an incomplete
            // directory rather than reporting a prefix as the container's contents. The open is
            // this walk's own validation — it is what the coverage ranges of this invocation
            // state — so a store's retention of an earlier request does not answer it.
            let root = root_origin(self.snapshot.id());
            let facts = self.snapshot.walked_container_facts(&root, budget)?;
            self.note_container(&root, &facts)?;
            self.stack.push(ContainerWalk {
                facts,
                position: 0,
                descend_from: 0,
            });
        }
        self.next_of_current(budget)
    }

    /// Opens one child container, or records the reason its subtree stays unknown.
    ///
    /// A failure that ends the walk is returned; structure damage and a refused nested depth are
    /// recorded against the entry the subtree hangs from and the walk continues with the next
    /// container — the rule the class-only walk applies (see
    /// [`walk_stops`](crate::scope_cursor)).
    fn descend(
        &mut self,
        candidate: usize,
        budget: &mut Budget,
    ) -> Result<Option<Arc<ContainerFacts>>> {
        let entry = {
            let top = self.stack.last().expect("a container is open");
            top.facts.entries().get(candidate).cloned().ok_or_else(|| {
                Error::invalid_input(
                    "entry_not_found",
                    "entry ordinal is absent from the container",
                )
            })?
        };
        let parent = self
            .stack
            .last()
            .expect("a container is open")
            .facts
            .origin()
            .clone();
        let origin = derive_child_origin(&parent, &entry.id);
        let opened = {
            let parent_facts = Arc::clone(&self.stack.last().expect("a container is open").facts);
            child_container_facts(&parent_facts, candidate, budget)
        };
        match opened {
            Ok(facts) => {
                self.note_container(&origin, &facts)?;
                Ok(Some(facts))
            }
            Err(error) if walk_stops(&error) => Err(error),
            Err(error) => {
                self.complete = false;
                crate::artifact::merge_tree_error(&mut self.issue, &error, budget);
                self.diagnostics.push(tree_diagnostic(&error, Some(&entry)));
                Ok(None)
            }
        }
    }

    /// Records one verified container product: its declared denominator and its repeated raw
    /// names.
    ///
    /// The walk keeps the product itself (its stack holds it for as long as it is inside that
    /// container) and deliberately does not register it as *held* by this snapshot: a hold is what
    /// lets other readers reach a product a consumer is still using, and this walk's contract is
    /// the opposite one — every container it reports a validated range for is opened by this
    /// request, so no reader may be told that range was validated by an answer it took from
    /// somewhere else.
    fn note_container(
        &mut self,
        origin: &ContainerOrigin,
        facts: &Arc<ContainerFacts>,
    ) -> Result<()> {
        let entries = u64::try_from(facts.entries().len()).map_err(|_| {
            Error::invalid_input(
                "entry_count_overflow",
                "container entry count does not fit u64",
            )
        })?;
        self.containers.push(ScopeContainer {
            origin: origin.clone(),
            entries,
            walked: true,
        });
        for (ordinal, first) in facts.repeated_names() {
            self.diagnostics.push(duplicate_diagnostic(ordinal, first));
        }
        Ok(())
    }

    /// The next entry of the container the walk is inside, or `None` once the scope ends.
    fn next_of_current(&mut self, budget: &mut Budget) -> Result<Option<ScopeEntry>> {
        loop {
            // Cancellation is observed at every entry: the poll happens before the entry at the
            // cursor is even looked at, so a cancelled request neither examines nor yields it.
            budget.poll()?;
            if self.stack.is_empty() {
                // The outermost container was left behind and nothing is above it: the walk reached
                // the end of the scope, and only now can it state a complete denominator.
                self.exhausted = true;
                return Ok(None);
            }
            let position = self.stack.last().expect("a container is open").position;
            let length = self
                .stack
                .last()
                .expect("a container is open")
                .facts
                .entries()
                .len();
            if position < length {
                self.stack.last_mut().expect("a container is open").position = position + 1;
                return Ok(Some(self.entry_at(position)?));
            }
            // The container's own entries are over: the nested archives it holds are walked next,
            // in entry order, before the container is left behind.
            match self.next_candidate() {
                Some(candidate) => {
                    if let Some(facts) = self.descend(candidate, budget)? {
                        self.stack.push(ContainerWalk {
                            facts,
                            position: 0,
                            descend_from: 0,
                        });
                    }
                }
                None => {
                    self.stack.pop();
                }
            }
        }
    }

    /// The next nested-archive candidate of the container the walk is inside, if any.
    fn next_candidate(&mut self) -> Option<usize> {
        if !self.descend {
            return None;
        }
        let top = self.stack.last_mut()?;
        while top.descend_from < top.facts.entries().len() {
            let position = top.descend_from;
            top.descend_from = position + 1;
            if top.facts.entries()[position].nested_archive
                == NestedArchiveState::CandidateNotScanned
            {
                return Some(position);
            }
        }
        None
    }

    /// One entry of the container the walk is inside, as the walk hands it over.
    fn entry_at(&self, position: usize) -> Result<ScopeEntry> {
        let top = self.stack.last().ok_or_else(|| {
            Error::invalid_input(
                "entry_not_found",
                "no container is open to yield an entry from",
            )
        })?;
        let record = top.facts.entries().get(position).ok_or_else(|| {
            Error::invalid_input(
                "entry_not_found",
                "entry ordinal is absent from the container",
            )
        })?;
        let depth = u64::try_from(top.facts.origin().steps.len()).map_err(|_| {
            Error::invalid_input("nested_depth_overflow", "container depth exceeds u64")
        })?;
        let kind = if record.nested_archive == NestedArchiveState::CandidateNotScanned {
            ScopeEntryKind::NestedContainer
        } else if is_class_candidate_name(&record.id.raw_name.0) {
            ScopeEntryKind::ClassCandidate
        } else {
            ScopeEntryKind::Other
        };
        Ok(ScopeEntry {
            entry: record.clone(),
            depth,
            kind,
        })
    }
}

/// The duplicate-name diagnostic one repeated record publishes.
///
/// The code, the message and the severity are the directory parser's own: a container's records are
/// that container's product whether they were enumerated into a report or pulled by a walk, so a
/// name that repeats is one fact with one spelling.
fn duplicate_diagnostic(ordinal: u64, first: u64) -> Diagnostic {
    Diagnostic {
        code: "duplicate_raw_name".into(),
        severity: DiagnosticSeverity::Warning,
        message: format!("entry {ordinal} repeats raw name first seen at ordinal {first}"),
        provenance: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactInput;
    use crate::budget::{Budget, Limits};
    use crate::model::{ContainerId, DiagnosticSeverity};
    use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Write};

    const STORED: u16 = 0;
    const DEFLATED: u16 = 8;

    fn limits() -> Limits {
        Limits {
            input_bytes: 1 << 20,
            archive_entries: 1_000,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 1_000,
            method_bodies: 1_000,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 8,
            dependency_depth: 8,
            elapsed_millis: u64::MAX,
        }
    }

    fn budget() -> Budget {
        Budget::new(limits())
    }

    /// A STORED or DEFLATED archive holding exactly the given entries, in the given order.
    fn zip(entries: &[(&[u8], Vec<u8>, u16)]) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut archive = ZipArchiveWriter::new(&mut output);
            for (name, data, method) in entries {
                let (mut entry, config) = archive
                    .new_file(EntryPath::verbatim(name.to_vec()))
                    .compression_method(CompressionMethod::new(*method))
                    .start()
                    .expect("the fixture entry starts");
                if *method == DEFLATED {
                    let encoder = flate2::write::DeflateEncoder::new(
                        &mut entry,
                        flate2::Compression::default(),
                    );
                    let mut writer = config.wrap(encoder);
                    writer.write_all(data).expect("the fixture payload writes");
                    let (encoder, descriptor) = writer.finish().expect("the payload finishes");
                    encoder.finish().expect("the deflate stream finishes");
                    entry.finish(descriptor).expect("the entry finishes");
                } else {
                    let mut writer = config.wrap(&mut entry);
                    writer.write_all(data).expect("the fixture payload writes");
                    let (_, descriptor) = writer.finish().expect("the payload finishes");
                    entry.finish(descriptor).expect("the entry finishes");
                }
            }
            archive.finish().expect("the fixture archive finishes");
        }
        output.into_inner()
    }

    /// A minimal, complete class file: the shared class magic, one method, no attributes.
    fn class_file() -> Vec<u8> {
        let mut bytes = 0xcafe_babe_u32.to_be_bytes().to_vec();
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // minor
        bytes.extend_from_slice(&52_u16.to_be_bytes()); // major: Java 8
        bytes.extend_from_slice(&5_u16.to_be_bytes()); // pool count
        // #1 Utf8 p/Seed
        bytes.push(1);
        bytes.extend_from_slice(&7_u16.to_be_bytes());
        bytes.extend_from_slice(b"p/Seed");
        // #2 Class #1
        bytes.push(7);
        bytes.extend_from_slice(&1_u16.to_be_bytes());
        // #3 Utf8 java/lang/Object
        bytes.push(1);
        bytes.extend_from_slice(&16_u16.to_be_bytes());
        bytes.extend_from_slice(b"java/lang/Object");
        // #4 Class #3
        bytes.push(7);
        bytes.extend_from_slice(&3_u16.to_be_bytes());
        bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // public super
        bytes.extend_from_slice(&2_u16.to_be_bytes()); // this
        bytes.extend_from_slice(&4_u16.to_be_bytes()); // super
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // fields
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // methods
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // attributes
        bytes
    }

    fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
        ArtifactSnapshot::open(ArtifactInput::bytes(bytes), &mut budget())
            .expect("the hand-written fixture opens")
    }

    fn tree_scope() -> PhysicalScope {
        PhysicalScope::ArtifactTree {
            root_container: ContainerId("root".into()),
        }
    }

    /// One entry of a walk or of an enumeration, in the terms both are compared in.
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Event {
        container: String,
        ordinal: u64,
        raw_name: Vec<u8>,
        depth: u64,
    }

    /// Every entry of one scope, pulled by the incremental walk.
    fn walked(
        snapshot: &ArtifactSnapshot,
        scope: &PhysicalScope,
    ) -> (Vec<Event>, Vec<ScopeEntryKind>, Vec<String>, CoverageState) {
        let mut cursor = snapshot.entry_cursor(scope).expect("the scope opens");
        let mut budget = budget();
        let mut events = Vec::new();
        let mut kinds = Vec::new();
        loop {
            let next = cursor
                .next_entry(&mut budget)
                .expect("the fixture walk reads cleanly");
            let Some(entry) = next else { break };
            events.push(Event {
                container: entry.entry.id.origin.current_container().0.clone(),
                ordinal: entry.entry.id.ordinal,
                raw_name: entry.entry.id.raw_name.0.clone(),
                depth: entry.depth,
            });
            kinds.push(entry.kind);
        }
        let diagnostics = cursor
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect();
        (events, kinds, diagnostics, cursor.coverage_state())
    }

    /// Every entry of one artifact tree, as the provider's own report enumerates it.
    fn enumerated(snapshot: &ArtifactSnapshot) -> Vec<Event> {
        let report = snapshot
            .enumerate_artifact_tree(&mut budget())
            .expect("the fixture tree enumerates");
        report
            .containers
            .iter()
            .flat_map(|container| {
                container.entries.iter().map(|entry| Event {
                    container: container.origin.current_container().0.clone(),
                    ordinal: entry.id.ordinal,
                    raw_name: entry.id.raw_name.0.clone(),
                    depth: container.depth,
                })
            })
            .collect()
    }

    /// One artifact tree holding a class, a nested container with a class, a deeper container and
    /// two resources, so the walk has to descend twice and skip entries in between.
    fn tree() -> Vec<u8> {
        let deepest = zip(&[(b"p/Deep.class", class_file(), STORED)]);
        let inner = zip(&[
            (b"p/Inner.class", class_file(), DEFLATED),
            (b"lib/deep.jar", deepest, STORED),
            (
                b"META-INF/services/p.Service",
                b"p.Provider\n".to_vec(),
                STORED,
            ),
        ]);
        zip(&[
            (
                b"META-INF/MANIFEST.MF",
                b"Manifest-Version: 1.0\r\n".to_vec(),
                STORED,
            ),
            (b"p/Root.class", class_file(), DEFLATED),
            (b"lib/inner.jar", inner, DEFLATED),
            (b"notes.txt", b"not a class".to_vec(), STORED),
        ])
    }

    #[test]
    fn the_entry_walk_yields_the_enumerations_own_entries_in_physical_order() {
        let snapshot = open(tree());
        let scope = tree_scope();
        let (events, _, diagnostics, state) = walked(&snapshot, &scope);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(
            state,
            CoverageState::CompleteWithinSchema,
            "a walk pulled to its end that read every container it reached is complete"
        );
        assert_eq!(
            events,
            enumerated(&snapshot),
            "the walk and the artifact-tree report must yield the same entries, in the same order, \
             at the same depth"
        );
        assert_eq!(
            events.iter().map(|event| event.ordinal).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 0, 1, 2, 0],
            "containers are walked depth first, container by container, in entry order"
        );
        assert_eq!(
            events.iter().map(|event| event.depth).collect::<Vec<_>>(),
            vec![0, 0, 0, 0, 1, 1, 1, 2]
        );
    }

    #[test]
    fn every_entry_carries_the_walks_own_class_and_resource_classification() {
        let snapshot = open(tree());
        let scope = tree_scope();
        let (events, kinds, _, state) = walked(&snapshot, &scope);
        assert_eq!(state, CoverageState::CompleteWithinSchema);
        // The classification is stated independently of the walk: a case-sensitive `.class` suffix
        // is a class candidate, an archive candidate the walk descends through is a nested
        // container, and everything else is the entry a class-only walk would have filtered out.
        let nested: Vec<&[u8]> = vec![b"lib/inner.jar", b"lib/deep.jar"];
        for (event, kind) in events.iter().zip(&kinds) {
            let expected = if event.raw_name.ends_with(b".class") {
                ScopeEntryKind::ClassCandidate
            } else if nested.contains(&event.raw_name.as_slice()) {
                ScopeEntryKind::NestedContainer
            } else {
                ScopeEntryKind::Other
            };
            assert_eq!(
                *kind,
                expected,
                "{:?} at ordinal {} is classified as {:?}",
                String::from_utf8_lossy(&event.raw_name),
                event.ordinal,
                kind
            );
        }
        // Every nested candidate of the scope was really descended through: the walk yielded the
        // entries of all three containers.
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == ScopeEntryKind::NestedContainer)
                .count(),
            2,
            "both nested archives of the fixture are candidates the walk descends through"
        );
    }

    #[test]
    fn a_whole_snapshot_scope_walks_the_root_container_only() {
        let snapshot = open(tree());
        let scope = PhysicalScope::SnapshotAll;
        let (events, _, diagnostics, state) = walked(&snapshot, &scope);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(state, CoverageState::CompleteWithinSchema);
        let root = snapshot
            .enumerate(&mut budget())
            .expect("the root container enumerates");
        assert_eq!(
            events,
            root.entries
                .iter()
                .map(|entry| Event {
                    container: entry.id.origin.current_container().0.clone(),
                    ordinal: entry.id.ordinal,
                    raw_name: entry.id.raw_name.0.clone(),
                    depth: 0,
                })
                .collect::<Vec<_>>(),
            "the whole-snapshot scope is the root container's own entries, and it descends nowhere"
        );
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn a_subtree_that_is_not_an_archive_is_named_and_the_walk_continues() {
        let broken = zip(&[
            (b"p/Before.class", class_file(), STORED),
            (b"lib/broken.jar", b"this is not a zip".to_vec(), STORED),
            (b"p/After.class", class_file(), STORED),
        ]);
        let snapshot = open(broken);
        let scope = tree_scope();
        let (events, _, diagnostics, state) = walked(&snapshot, &scope);
        assert_eq!(
            diagnostics,
            vec!["zip_open".to_string()],
            "the unreadable subtree is named by the walk that met it"
        );
        assert_eq!(
            state,
            CoverageState::Partial,
            "a subtree left unknown is never a complete denominator"
        );
        assert_eq!(
            events
                .iter()
                .map(|event| event.raw_name.clone())
                .collect::<Vec<_>>(),
            vec![
                b"p/Before.class".to_vec(),
                b"lib/broken.jar".to_vec(),
                b"p/After.class".to_vec()
            ],
            "the entry that holds the unreadable subtree is yielded like every other entry, and the \
             entries after it are still walked"
        );

        let mut cursor = snapshot.entry_cursor(&scope).expect("the scope opens");
        let mut budget = budget();
        while cursor
            .next_entry(&mut budget)
            .expect("the readable entries read cleanly")
            .is_some()
        {}
        assert!(
            cursor.issue().is_some(),
            "a subtree that could not be read bounds the report"
        );
        let diagnostic = cursor
            .diagnostics()
            .first()
            .expect("the unreadable subtree is reported");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, "zip_open");
        let provenance = diagnostic
            .provenance
            .as_ref()
            .expect("the diagnostic names the entry it hangs from");
        match &provenance.location {
            crate::model::Location::Entry { id, .. } => {
                assert_eq!(id.raw_name.0, b"lib/broken.jar");
                assert_eq!(id.ordinal, 1);
            }
            other => panic!("expected the parent entry as the diagnostic origin, got {other:?}"),
        }
        // The enumeration reads the same subtree as unknown: its entries are in neither report.
        assert_eq!(events, enumerated(&snapshot));
    }

    #[test]
    fn a_cancellation_observed_between_two_entries_stops_the_walk_there() {
        let snapshot = open(tree());
        let scope = tree_scope();
        let token = crate::budget::CancellationToken::new();
        let mut budget = Budget::with_cancellation_token(limits(), token.clone());
        let mut cursor = snapshot.entry_cursor(&scope).expect("the scope opens");

        let first = cursor
            .next_entry(&mut budget)
            .expect("the first entry is read before the stop")
            .expect("the scope is not empty");
        assert_eq!(first.entry.id.ordinal, 0);
        token.cancel();
        let stopped = cursor.next_entry(&mut budget);
        assert!(
            matches!(stopped, Err(Error::Cancelled { .. })),
            "the cancellation is observed before the next entry: {stopped:?}"
        );
        assert!(
            cursor
                .next_entry(&mut budget)
                .expect("a stopped walk answers")
                .is_none(),
            "a stopped walk yields nothing further"
        );
        assert_eq!(cursor.coverage_state(), CoverageState::Partial);
        assert_eq!(
            cursor
                .containers()
                .iter()
                .map(|container| (
                    container.origin.current_container().0.clone(),
                    container.walked
                ))
                .collect::<Vec<_>>(),
            vec![("root".to_string(), true)],
            "the walk states the denominator of the one container it opened"
        );
    }

    #[test]
    fn a_budget_stop_keeps_the_containers_already_walked_and_states_the_one_it_did_not_open() {
        let snapshot = open(tree());
        let mut budget = Budget::new(Limits {
            archive_entries: 5,
            ..limits()
        });
        let mut cursor = snapshot
            .entry_cursor(&tree_scope())
            .expect("the scope opens");
        let mut names = Vec::new();
        let stopped = loop {
            match cursor.next_entry(&mut budget) {
                Ok(Some(entry)) => names.push(entry.entry.id.raw_name.0.clone()),
                Ok(None) => panic!("the fixture tree is larger than the budget allows"),
                Err(error) => break error,
            }
        };
        assert!(
            matches!(
                stopped,
                Error::BudgetExceeded {
                    dimension: crate::budget::BudgetDimension::ArchiveEntries,
                    ..
                }
            ),
            "the stop names the dimension it exhausted: {stopped:?}"
        );
        assert_eq!(
            names,
            vec![
                b"META-INF/MANIFEST.MF".to_vec(),
                b"p/Root.class".to_vec(),
                b"lib/inner.jar".to_vec(),
                b"notes.txt".to_vec()
            ],
            "the root container's four entries were yielded before the descent ran out of budget"
        );
        let containers = cursor.containers();
        assert_eq!(containers.len(), 1, "{containers:?}");
        assert_eq!(containers[0].entries, 4);
        assert!(containers[0].walked);
        assert_eq!(cursor.coverage_state(), CoverageState::Partial);
    }

    #[test]
    fn a_walk_that_stopped_before_its_first_entry_states_the_roots_declared_range() {
        let snapshot = open(tree());
        let token = crate::budget::CancellationToken::new();
        token.cancel();
        let mut budget = Budget::with_cancellation_token(limits(), token);
        let mut cursor = snapshot
            .entry_cursor(&tree_scope())
            .expect("the scope opens");
        assert!(matches!(
            cursor.next_entry(&mut budget),
            Err(Error::Cancelled { .. })
        ));
        let containers = cursor.containers();
        assert_eq!(containers.len(), 1, "{containers:?}");
        assert_eq!(
            containers[0].entries, 4,
            "the root directory's own declaration is stated even though nothing was examined"
        );
        assert!(
            !containers[0].walked,
            "no record of it was validated, so no range may claim it as examined"
        );
    }

    #[test]
    fn a_standalone_class_snapshot_has_no_entry_to_walk() {
        let snapshot = open(class_file());
        let error = snapshot
            .entry_cursor(&PhysicalScope::SnapshotAll)
            .err()
            .expect("a standalone CLASS snapshot holds no container");
        match error {
            Error::InvalidInput { code, .. } => assert_eq!(code, "navigation_not_zip"),
            other => panic!("expected the scope refusal, got {other:?}"),
        }
    }

    #[test]
    fn a_duplicate_raw_name_is_one_fact_of_the_walks_own_directory() {
        let duplicated = zip(&[
            (b"p/A.class", class_file(), STORED),
            (b"p/A.class", class_file(), STORED),
        ]);
        let snapshot = open(duplicated);
        let mut cursor = snapshot
            .entry_cursor(&tree_scope())
            .expect("the scope opens");
        let mut budget = budget();
        while cursor
            .next_entry(&mut budget)
            .expect("the fixture walk reads cleanly")
            .is_some()
        {}
        let diagnostics = cursor.diagnostics();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0].code, "duplicate_raw_name");
        assert_eq!(
            diagnostics[0].message,
            "entry 1 repeats raw name first seen at ordinal 0"
        );
    }
}
