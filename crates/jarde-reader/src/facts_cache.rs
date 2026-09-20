//! A bounded in-memory store of the facts a request reads and another request may reuse.
//!
//! P5 2.1 measured the direct path and 2.2 decided what a cache would have to bind; this module is
//! that record built, under the boundary the change documents fix: **in memory only** (no file, no
//! format on disk, no migration), **no third-party dependency**, **disabled unless a caller
//! attaches one**, and **no concurrency**: the engine is one sequential scan and nothing here
//! spawns work.
//!
//! It holds two layers, keyed and verified separately because they answer different questions:
//!
//! * the **CP/Header layer** — a class's parsed structure, keyed by the class bytes' content and
//!   the parse policy;
//! * the **container layer** (bound-container-lookup) — a container's **verified facts**: the
//!   immutable backing its entries live in, its complete central directory, and the multi-value
//!   raw-name locator over that directory.
//!
//! Both are immutable once written, both are written only by a read that ran to the end, and both
//! are refused rather than evicted when they do not fit. Nothing here is a session, a global, or a
//! second injection path: a [`FactsCache`] is an explicit handle on [`Budget`], and the engine
//! constructs none.
//!
//! ## What the CP/Header layer holds, and what it deliberately does not
//!
//! The entry is a [`ClassFacts`] (constant pool and declaration structure) or a
//! [`HeaderInspection`] (the same structure plus the version gate). Both are **pure functions of
//! the class bytes and the parse policy**, which is why this is the layer the `facts-cache` spec
//! calls CP/Header and why its key is the one the spec names:
//!
//! | key dimension | carrier |
//! | --- | --- |
//! | class content digest (and length) | the bytes the entry was parsed from, hashed here |
//! | parser / registry version | [`FactsIdentity::registry`], which is [`HIGHEST_REGISTERED_MAJOR`] |
//! | parse policy | [`ParsePolicy`]: the structural facts, or the header under `InspectionMode` |
//! | budget (completeness) | an entry is written **only** by a parse that ran to the end |
//!
//! The dimensions the upper layers need are **not** in this key and must not be: a `ClassFacts` is
//! the same value whatever runtime profile, output level, dependency set or recovery pass asked for
//! it, so binding any of those here would be an invalidation for a change that cannot change the
//! answer. The resolution layer adds symbol/source context, view/domain/platform and the provider
//! snapshot; the IR and source layers add the method content, the analysis/recovery identities, the
//! output level and the naming configuration — those layers own that record (`tests/p5_benchmark.rs`
//! holds the per-layer dimension list, and its IR and recovery entries still have no carrier in the
//! code at all).
//!
//! ## What the container layer holds, and why its key is physical
//!
//! A container product is what the reader proved about one container in one snapshot: its
//! **backing** (the bytes its central directory and entries live in — the snapshot's own bytes for
//! the root container, the verified materialization of a nested entry otherwise), its **complete**
//! central directory, and the **locator** from a raw name to every record that carries it, in
//! central-directory order. It is keyed by the snapshot's content identity, the full
//! [`ContainerOrigin`] and [`CONTAINER_FACTS_SCHEMA`]:
//!
//! * `RuntimeProfile`, loader order and any prefix are **not** dimensions: a container's physical
//!   facts are the same whatever order searched it, and those are applied by the resolver per
//!   request. Two requests that name the same origin share one product.
//! * Only a directory that parsed **completely** is offered. A stopped parse — budget,
//!   cancellation, damage — is refused with the reason, never published as a prefix, so "this
//!   container holds no such name" can never be read out of an incomplete directory.
//! * A retained locator is only ever used against the backing it was built from, and the selected
//!   entry's own local header, CRC and sizes are re-verified on every read, so a hit changes what
//!   the request pays for and never what it proves.
//!
//! ## What a hit is, and what it can never be
//!
//! * **The read still happens.** The CP/Header cache sits *above* the bounded read and *below*
//!   every consumer: the bytes are materialized, charged, CRC-checked and digested exactly as the
//!   direct path does, and only the parse is skipped. Coverage, origins, read evidence and the
//!   entry's identity are therefore produced by the same code on both paths, and a hit cannot claim
//!   a range nobody read. The container layer is the same rule one level down: a hit skips the
//!   directory parse and the parent's re-materialization, and the class read that follows is still
//!   charged and still verified.
//! * **A hit charges no counted dimension it did not perform.** It reads, decodes and derives
//!   nothing beyond the read the request still owes; the dimensions that measure those steps stay
//!   where they are, and the saving is what the comparison reports. A hit still calls
//!   [`Budget::poll`] and still applies the **current** request's structural limits — a container
//!   retained under a wider `NestedDepth` is not served to a request that may not reach it — so a
//!   cancelled, expired or already-stopped request terminates through the cache exactly as it does
//!   without one.
//! * **Nothing negative, partial or refused is ever stored.** A parse that stops (budget,
//!   cancellation, a strict version refusal) writes nothing, so an incomplete answer can never stand
//!   in for a complete one and a negative verdict can never be replayed after the reason for it
//!   changed. The products here are refused-or-absent, never "no such class" and never "no such
//!   name".
//! * **Content is shared, origins are not.** Two entries, two snapshots or two loaders that hold the
//!   same bytes share one entry; what comes back is a value that carries no origin, and the caller
//!   binds the physical origin it read from. A cache cannot merge two origins because it never held
//!   one. A container product is the other way round: it *is* one origin, and a lookup that names
//!   another one cannot reach it.
//!
//! ## Capacity, weight and release
//!
//! Both layers share one budget and two bounds ([`FactsCapacity`]): a number of answers and a
//! retained weight in bytes. The weight is a **proxy for residency, never RSS**: a container
//! contributes its backing bytes, a per-record estimate for its directory and the names its locator
//! holds; a CP/Header entry contributes the length of the class bytes it was parsed from. A backing
//! referenced by one product is counted once; nothing is ever charged twice for the same memory.
//! An insertion that would cross either bound is refused and counted ([`FactsReport::refused_capacity`],
//! [`FactsReport::refused_capacity_bytes`]) — there is no eviction and no replacement policy — and
//! the request that was refused keeps using the facts it already read, so a refusal never makes it
//! read anything again. [`FactsCache::clear`] and dropping every handle release what the store held.
//!
//! ## Versions, damage and the fallback
//!
//! Every entry records the [`FactsIdentity`] it was written under. A lookup that finds an entry
//! written under another **entry format** or another **registry version** discards it, counts the
//! discard and returns "not answered by the cache" — the caller then runs the direct path under the
//! budget it already holds. There is no second attempt, no re-run and no budget reset: the fallback
//! is the same bounded read the cache would have replaced.

use crate::artifact::ContainerFacts;
use crate::budget::Budget;
use crate::classfile::{ClassFacts, HeaderInspection, InspectionMode};
use crate::error::{Error, Result};
use crate::model::{ContainerOrigin, Digest};
use crate::release_registry::HIGHEST_REGISTERED_MAJOR;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// The entry format this build writes.
///
/// An entry carries the format it was written in, and a lookup discards one written in another: a
/// store that outlived a change to what an entry holds is read as "this entry is not answerable"
/// rather than as the answer. The number is this build's declaration, not a compatibility promise.
pub const FACTS_FORMAT: u16 = 1;

/// The container-directory schema this build writes.
///
/// A container product is keyed by it as well as by the physical origin, so a build that changes
/// what a directory holds or how it is verified cannot serve an old directory as the new answer:
/// the schema is part of the key and a changed schema simply misses. Like [`FACTS_FORMAT`] this is
/// this build's declaration, not a compatibility promise.
pub const CONTAINER_FACTS_SCHEMA: u16 = 1;

/// How much one store may retain, in two independent bounds.
///
/// The store refuses an insertion that would cross either bound; nothing here evicts, so an
/// insertion that fits stays until the store is cleared or dropped. Both bounds are needed
/// because they bound different things: the entry limit bounds the *number* of retained answers
/// (each one a map slot, an identity and a payload header) while the retained-byte limit bounds
/// the bytes those answers hold — a nested container's backing, its directory and name table, or
/// a class's CP/Header payload.
///
/// The byte figure is a **residency proxy**, never RSS: it is the sum of the weights this module
/// computes (see [`FactsReport::retained_bytes`]), with no allocator overhead, no fragmentation
/// and no copy an upper layer made of a fact it read here.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FactsCapacity {
    /// Retained answers (CP/Header entries plus container products).
    pub entries: usize,
    /// Retained weight, in bytes, across every entry and container product.
    pub retained_bytes: u64,
}

impl FactsCapacity {
    pub const fn new(entries: usize, retained_bytes: u64) -> Self {
        Self {
            entries,
            retained_bytes,
        }
    }

    /// A capacity that retains nothing. A store with it is a **counter**: every look-up misses,
    /// every insertion is refused, and no request changes what it does, so a measurement can
    /// observe how many directories a path parsed and how many nested containers it
    /// materialized without retaining any of them.
    pub const fn none() -> Self {
        Self::new(0, 0)
    }

    /// A stable one-line description, with no counter in it.
    pub fn describe(&self) -> String {
        format!(
            "{} entries / {} retained bytes",
            self.entries, self.retained_bytes
        )
    }
}

/// The declaration a store's entries are read under: the parser/registry version and the entry
/// format.
///
/// It is a *declaration* — the same shape `ResolutionEnvironment` has — and [`Self::current`] is the
/// value of this build. A caller holding a store that another build filled declares that other
/// identity when it reads it, which is how an incompatible entry is *detected* instead of served.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct FactsIdentity {
    /// The release registry's version the facts were parsed under (the reader's parser, as the
    /// registry states the dialect it validates).
    pub registry: u16,
    /// The entry format the entries were written in.
    pub format: u16,
}

impl FactsIdentity {
    /// The identity of this build's parser.
    pub const fn current() -> Self {
        Self {
            registry: HIGHEST_REGISTERED_MAJOR,
            format: FACTS_FORMAT,
        }
    }

    pub const fn new(registry: u16, format: u16) -> Self {
        Self { registry, format }
    }

    /// A stable one-line description of the declaration, with no counter in it: this is what a
    /// measurement context records as the cache state, so it may not move between two runs of one
    /// configuration.
    pub fn describe(&self) -> String {
        format!("registry {} entry format {}", self.registry, self.format)
    }
}

/// The parse policy an entry was produced under.
///
/// A header inspection is *not* policy-free: `Strict` refuses a release the registry does not
/// validate while `Forensic` reads its structure and reports the dialect instead. The two answers
/// are different products of the same bytes, so the policy is a key dimension and one policy's
/// entry is never served to another — which is what keeps a strict request from being answered
/// with a forensic read that a strict read would have refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum ParsePolicy {
    /// The constant pool and the declaration structure: `class_facts`, which applies no version
    /// gate and refuses no version.
    Structure,
    /// The header inspection under `InspectionMode::Strict`.
    Strict,
    /// The header inspection under `InspectionMode::Forensic`.
    Forensic,
}

impl ParsePolicy {
    pub(crate) const fn of(mode: InspectionMode) -> Self {
        match mode {
            InspectionMode::Strict => Self::Strict,
            InspectionMode::Forensic => Self::Forensic,
        }
    }

    /// Whether a payload of this kind answers this policy.
    const fn holds(self, payload: &FactsPayload) -> bool {
        match (self, payload) {
            (Self::Structure, FactsPayload::Structure(_)) => true,
            (Self::Strict | Self::Forensic, FactsPayload::Header(_)) => true,
            (Self::Structure, FactsPayload::Header(_))
            | (Self::Strict | Self::Forensic, FactsPayload::Structure(_)) => false,
        }
    }
}

/// What one class's bytes are cached under: their content, and the policy that was asked of them.
///
/// The digest is the *content* identity, so two entries of one archive, two snapshots and two
/// loaders that hold the same bytes share one entry. Nothing about where the bytes were read is
/// part of the key, and that is deliberate: an entry describes bytes, and the caller that returns a
/// fact binds the origin it read the bytes from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FactsKey {
    pub(crate) digest: Digest,
    pub(crate) length: u64,
    pub(crate) policy: ParsePolicy,
}

impl FactsKey {
    /// The class bytes' identity, hashed only when a cache is really consulted: the direct path
    /// computes no digest it does not already need, so attaching no cache costs no hashing.
    fn of(bytes: &[u8], policy: ParsePolicy) -> Self {
        Self {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: bytes.len() as u64,
            policy,
        }
    }
}

impl Ord for FactsKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.digest
            .0
            .cmp(&other.digest.0)
            .then_with(|| self.length.cmp(&other.length))
            .then_with(|| self.policy.cmp(&other.policy))
    }
}

impl PartialOrd for FactsKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// What one container's facts are cached under: the snapshot's content identity, the complete
/// physical origin of the container, and the directory/verification schema.
///
/// The origin carries the snapshot identity, and the snapshot is named again here so the key
/// states its two content dimensions — the bytes the container lives in and the chain that
/// reaches it — without a reader having to know that the origin embeds the first one.
///
/// `RuntimeProfile`, loader order and prefix are **not** dimensions: they are applied by the
/// resolver to the facts it gets back and cannot change what a container physically is. Two
/// requests that name the same origin share one directory whatever order they searched in.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ContainerKey {
    pub(crate) snapshot: crate::model::SnapshotId,
    pub(crate) origin: ContainerOrigin,
    pub(crate) schema: u16,
}

/// One retained container product: the verified facts plus the declaration they were read under.
#[derive(Clone, Debug)]
struct ContainerEntry {
    identity: FactsIdentity,
    facts: Arc<ContainerFacts>,
    /// The weight this product contributes to the store's retained bytes. Recomputed at insertion
    /// and stored beside the product so a rewrite can subtract exactly what it added.
    weight: u64,
}

/// One cached answer.
#[derive(Clone, Debug, Eq, PartialEq)]
enum FactsPayload {
    Structure(ClassFacts),
    Header(HeaderInspection),
}

/// One entry: the payload plus the declaration it was written under.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Entry {
    identity: FactsIdentity,
    payload: FactsPayload,
    /// The weight this payload contributes to the store's retained bytes: the length of the class
    /// bytes it was parsed from, stored beside it so a rewrite can subtract exactly what it added.
    weight: u64,
}

/// Why an entry found under a key could not be used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Unusable {
    /// Written in another entry format.
    Format,
    /// Parsed under another registry version.
    Registry,
    /// Holds the other product: the store's invariant, not a policy a caller can reach.
    Product,
}

/// What one store has answered, in one place.
///
/// The CP/Header counters keep their meaning; the container counters are separate because the two
/// products answer different questions and a measurement has to be able to say which one moved.
/// `directory_parses` and `nested_materializations` count **work the store's callers did**, not
/// work the store did: they are how a request reports that it built a directory or expanded a
/// nested container while holding this handle, and a reuse saves them by never being counted.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Counters {
    consultations: u64,
    hits: u64,
    misses: u64,
    stored: u64,
    refused_capacity: u64,
    refused_capacity_bytes: u64,
    discarded_format: u64,
    discarded_registry: u64,
    discarded_product: u64,
    /// Container-products looked up, answered from retention, and not found.
    container_consultations: u64,
    container_hits: u64,
    container_misses: u64,
    /// Container products written by a request that had already read them completely.
    container_stored: u64,
    /// Container directories parsed by a request holding this handle (a hit never parses one).
    directory_parses: u64,
    /// Nested containers materialized by a request holding this handle, and the bytes produced.
    nested_materializations: u64,
    nested_materialized_bytes: u64,
}

/// The entries and their counters, shared by every handle that reads them.
#[derive(Debug)]
struct Shared {
    capacity: FactsCapacity,
    /// CP/Header entries, keyed by class content and parse policy.
    entries: BTreeMap<FactsKey, Entry>,
    /// Container products, keyed by snapshot, physical origin and schema.
    containers: BTreeMap<ContainerKey, ContainerEntry>,
    /// The weight every retained entry and container product contributes, summed.
    retained_bytes: u64,
    counters: Counters,
}

/// The state of one store, as the caller that holds a handle can report it.
///
/// The whole point of reporting it *outside* the result is that the cache is transparent: a
/// fallback, a discard or a capacity refusal changes no evidence, coverage, representation or
/// diagnostic, so it may not appear in the published report as if it had. `performance-gates`'
/// `Cold and warm results` compares those planes for equality, and a cache state tucked into them
/// would make every warm run disagree with its cold one for a reason that is not a result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FactsReport {
    pub identity: FactsIdentity,
    pub capacity: FactsCapacity,
    /// How many CP/Header entries this store holds now.
    pub entries: usize,
    /// How many container products this store holds now.
    pub containers: usize,
    /// The weight of everything the store holds now, in the units [`FactsCapacity`] bounds. A
    /// **residency proxy**, not RSS: see that type's documentation.
    pub retained_bytes: u64,
    /// Lookups asked of the store. `hits + misses`; a discarded entry counts as a miss.
    pub consultations: u64,
    pub hits: u64,
    pub misses: u64,
    /// Entries written by a parse that ran to the end. A parse that stopped writes nothing, which
    /// is why no counter here can say "an incomplete answer was stored".
    pub stored: u64,
    /// Stores refused because the store already held `capacity.entries` answers: the facts stayed
    /// uncached and the request was served by its own read.
    pub refused_capacity: u64,
    /// Stores refused because the answer, or the answer beside what the store already held, would
    /// cross `capacity.retained_bytes`. Separate from [`FactsReport::refused_capacity`] so a
    /// measurement can say which bound the retention hit.
    pub refused_capacity_bytes: u64,
    /// Entries *discarded* because they were written in another entry format.
    pub discarded_format: u64,
    /// Entries discarded because they were parsed under another registry version.
    pub discarded_registry: u64,
    /// Entries discarded because they did not hold the product their policy names.
    pub discarded_product: u64,
    /// Container lookups asked of the store, answered from it, and not found.
    pub container_consultations: u64,
    pub container_hits: u64,
    pub container_misses: u64,
    /// Complete container products this store accepted.
    pub container_stored: u64,
    /// Container directories the requests holding this handle parsed. A hit never parses one, so
    /// this is the count a reuse makes stop moving.
    pub directory_parses: u64,
    /// Nested containers those requests materialized, and the bytes that produced. A hit never
    /// materializes one, for the same reason.
    pub nested_materializations: u64,
    pub nested_materialized_bytes: u64,
}

impl FactsReport {
    /// Whether the store answered nothing it was asked.
    pub fn answered_nothing(&self) -> bool {
        self.hits == 0
    }

    /// The discards, as a one-line reading.
    pub fn discards(&self) -> String {
        format!(
            "format {} registry {} product {}",
            self.discarded_format, self.discarded_registry, self.discarded_product
        )
    }

    /// The reuse evidence, as a one-line reading: how many container lookups were answered from
    /// retention, and how much work the requests did anyway.
    pub fn reuse(&self) -> String {
        format!(
            "container {}/{} hits, directories parsed {}, nested materialized {} ({} bytes)",
            self.container_hits,
            self.container_consultations,
            self.directory_parses,
            self.nested_materializations,
            self.nested_materialized_bytes
        )
    }

    /// The residency figure and the refusals, as a one-line reading.
    pub fn residency(&self) -> String {
        format!(
            "{} entries / {} containers, {} retained bytes of {} allowed; refused {} by entry limit, \
             {} by byte limit",
            self.entries,
            self.containers,
            self.retained_bytes,
            self.capacity.describe(),
            self.refused_capacity,
            self.refused_capacity_bytes
        )
    }
}

/// One store's handle: the entries, plus the declaration this handle reads them under.
///
/// A handle is cheap to clone (the entries are behind one `Arc`) and every clone shares one store,
/// which is the whole point: a cache that could not outlive one request would save nothing. The
/// `Mutex` is not a concurrency feature — nothing in this engine spawns work — it is what keeps
/// [`FactsCache`] and [`Budget`] `Send + Sync` while one store is held by several requests.
#[derive(Clone, Debug)]
pub struct FactsCache {
    shared: Arc<Mutex<Shared>>,
    identity: FactsIdentity,
}

impl FactsCache {
    /// A fresh store, read under `identity`, holding at most `capacity`.
    ///
    /// Both bounds are enforced together: an answer is inserted only when its key is already
    /// present (a rewrite) or when it fits in the entry limit *and* in the retained-byte limit
    /// beside everything the store already holds. Nothing is ever evicted: the store is a
    /// retention decision, not a replacement policy, and a caller that wants room takes it back by
    /// dropping handles or calling [`FactsCache::clear`].
    pub fn new(identity: FactsIdentity, capacity: FactsCapacity) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared {
                capacity,
                entries: BTreeMap::new(),
                containers: BTreeMap::new(),
                retained_bytes: 0,
                counters: Counters::default(),
            })),
            identity,
        }
    }

    /// A fresh store read under this build's identity.
    pub fn current(capacity: FactsCapacity) -> Self {
        Self::new(FactsIdentity::current(), capacity)
    }

    /// A handle over **the same entries**, read under another declaration.
    ///
    /// This is how an incompatible entry is reachable at all: a store's entries were written by one
    /// build and are read by a caller that declares what it expects to find. Nothing here rewrites
    /// an entry, and a mismatch discards rather than converts — see [`Self::unusable`].
    pub fn over(&self, identity: FactsIdentity) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            identity,
        }
    }

    pub fn identity(&self) -> FactsIdentity {
        self.identity
    }

    pub fn capacity(&self) -> FactsCapacity {
        self.shared().capacity
    }

    /// A stable one-line label for this handle's declaration. Part of the measurement context, so it
    /// carries no counter: two runs of one configuration describe their cache the same way.
    pub fn describe(&self) -> String {
        format!(
            "facts cache ({}), capacity {}",
            self.identity.describe(),
            self.capacity().describe()
        )
    }

    /// Releases everything the store holds, leaving its counters and its declaration in place.
    ///
    /// The store's *retention* is the only thing this touches: a request that still holds an `Arc`
    /// to a container product keeps using it, and the next request simply finds nothing retained.
    /// A handle is a view of one store, so clearing any handle clears the store every clone sees.
    pub fn clear(&self) {
        let mut guard = self.shared();
        let shared = &mut *guard;
        shared.entries.clear();
        shared.containers.clear();
        shared.retained_bytes = 0;
    }

    pub fn report(&self) -> FactsReport {
        let guard = self.shared();
        let shared = &*guard;
        let counters = shared.counters;
        FactsReport {
            identity: self.identity,
            capacity: shared.capacity,
            entries: shared.entries.len(),
            containers: shared.containers.len(),
            retained_bytes: shared.retained_bytes,
            consultations: counters.consultations,
            hits: counters.hits,
            misses: counters.misses,
            stored: counters.stored,
            refused_capacity: counters.refused_capacity,
            refused_capacity_bytes: counters.refused_capacity_bytes,
            discarded_format: counters.discarded_format,
            discarded_registry: counters.discarded_registry,
            discarded_product: counters.discarded_product,
            container_consultations: counters.container_consultations,
            container_hits: counters.container_hits,
            container_misses: counters.container_misses,
            container_stored: counters.container_stored,
            directory_parses: counters.directory_parses,
            nested_materializations: counters.nested_materializations,
            nested_materialized_bytes: counters.nested_materialized_bytes,
        }
    }

    /// The structural facts of `bytes`, when this store already holds them under this handle's
    /// declaration.
    pub(crate) fn structure(
        &self,
        bytes: &[u8],
        budget: &mut Budget,
    ) -> Result<Option<ClassFacts>> {
        let taken = self.take(FactsKey::of(bytes, ParsePolicy::Structure), budget)?;
        Ok(match taken {
            // The policy check inside `take` makes this arm unreachable; it returns `None` rather
            // than panicking, because a cache is not a place to abort a request.
            Some(FactsPayload::Header(_)) | None => None,
            Some(FactsPayload::Structure(facts)) => Some(facts),
        })
    }

    /// Writes the structural facts of `bytes`. Only a parse that ran to the end reaches this.
    pub(crate) fn remember_structure(&self, bytes: &[u8], facts: &ClassFacts) {
        self.keep(
            FactsKey::of(bytes, ParsePolicy::Structure),
            FactsPayload::Structure(facts.clone()),
        );
    }

    /// The header inspection of `bytes` under `mode`, when this store holds one.
    pub(crate) fn header(
        &self,
        bytes: &[u8],
        mode: InspectionMode,
        budget: &mut Budget,
    ) -> Result<Option<HeaderInspection>> {
        let taken = self.take(FactsKey::of(bytes, ParsePolicy::of(mode)), budget)?;
        Ok(match taken {
            Some(FactsPayload::Header(inspection)) => Some(inspection),
            Some(FactsPayload::Structure(_)) | None => None,
        })
    }

    /// Writes the header inspection of `bytes` under `mode`. The version gate has already accepted
    /// it, so a refusal is never written.
    pub(crate) fn remember_header(
        &self,
        bytes: &[u8],
        mode: InspectionMode,
        inspection: &HeaderInspection,
    ) {
        self.keep(
            FactsKey::of(bytes, ParsePolicy::of(mode)),
            FactsPayload::Header(inspection.clone()),
        );
    }

    /// The retained facts of one container, when this store holds them under this handle's
    /// declaration.
    ///
    /// The **current request** decides first: the budget is polled, so a cancelled or expired
    /// request is refused before anything is served, and the container's own depth is checked
    /// against this request's `NestedDepth` limit, so facts built under a wider allowance are
    /// never handed to a request that is not allowed to reach that depth. Nothing here resets,
    /// borrows or re-creates budget state: a hit is a read of memory the request may use, not a
    /// new allowance.
    pub(crate) fn container(
        &self,
        origin: &ContainerOrigin,
        budget: &mut Budget,
    ) -> Result<Option<Arc<ContainerFacts>>> {
        budget.poll()?;
        let depth = u64::try_from(origin.steps.len()).map_err(|_| {
            Error::invalid_input(
                "nested_depth_overflow",
                "container origin is too deep to address",
            )
        })?;
        budget.check_nested_depth(depth)?;
        let key = ContainerKey {
            snapshot: origin.snapshot.clone(),
            origin: origin.clone(),
            schema: CONTAINER_FACTS_SCHEMA,
        };
        let mut guard = self.shared();
        let shared = &mut *guard;
        shared.counters.container_consultations += 1;
        let Some(entry) = shared.containers.get(&key) else {
            shared.counters.container_misses += 1;
            return Ok(None);
        };
        if let Some(reason) = self.unusable_container(entry) {
            let weight = entry.weight;
            shared.containers.remove(&key);
            shared.retained_bytes = shared.retained_bytes.saturating_sub(weight);
            shared.counters.container_misses += 1;
            match reason {
                Unusable::Format => shared.counters.discarded_format += 1,
                Unusable::Registry => shared.counters.discarded_registry += 1,
                Unusable::Product => shared.counters.discarded_product += 1,
            }
            return Ok(None);
        }
        let facts = Arc::clone(&entry.facts);
        shared.counters.container_hits += 1;
        Ok(Some(facts))
    }

    /// Offers one **complete** container product for retention.
    ///
    /// The product was read completely by the request that calls this, and the request keeps using
    /// it whether or not this store accepts it: a refusal is a retention decision and never a
    /// reason to read the container again. A product that does not fit in the entry limit or in
    /// the retained-byte limit is refused and counted; nothing is evicted to make room.
    pub(crate) fn remember_container(&self, facts: &Arc<ContainerFacts>) {
        let origin = facts.origin();
        let key = ContainerKey {
            snapshot: origin.snapshot.clone(),
            origin: origin.clone(),
            schema: CONTAINER_FACTS_SCHEMA,
        };
        let weight = facts.weight();
        let entry = ContainerEntry {
            identity: self.identity,
            facts: Arc::clone(facts),
            weight,
        };
        let mut guard = self.shared();
        let shared = &mut *guard;
        let occupied_weight = shared.containers.get(&key).map(|existing| existing.weight);
        match occupied_weight {
            Some(previous) => {
                // A key the store already holds may always be rewritten — the product that
                // reached here was read completely — so only a *new* key can be refused.
                shared.retained_bytes = shared.retained_bytes.saturating_sub(previous);
                shared.retained_bytes = shared.retained_bytes.saturating_add(weight);
                shared.containers.insert(key, entry);
                shared.counters.container_stored += 1;
            }
            None => {
                let items = shared.entries.len() + shared.containers.len();
                if items >= shared.capacity.entries {
                    shared.counters.refused_capacity += 1;
                    return;
                }
                if weight > shared.capacity.retained_bytes
                    || shared
                        .retained_bytes
                        .saturating_add(weight)
                        .gt(&shared.capacity.retained_bytes)
                {
                    shared.counters.refused_capacity_bytes += 1;
                    return;
                }
                shared.retained_bytes += weight;
                shared.containers.insert(key, entry);
                shared.counters.container_stored += 1;
            }
        }
    }

    /// Records that the request holding this handle parsed one container's directory.
    ///
    /// Called by the reader while it builds a container's facts, so the count includes a build
    /// whose product the store then refused. A hit never calls it, which is exactly the evidence a
    /// reuse is judged by.
    pub(crate) fn note_directory_parse(&self) {
        self.shared().counters.directory_parses += 1;
    }

    /// Records that the request holding this handle materialized one nested container, of
    /// `bytes` produced.
    pub(crate) fn note_nested_materialization(&self, bytes: u64) {
        let mut guard = self.shared();
        let counters = &mut guard.counters;
        counters.nested_materializations += 1;
        counters.nested_materialized_bytes =
            counters.nested_materialized_bytes.saturating_add(bytes);
    }

    /// A lock this cache never leaves poisoned behind it.
    ///
    /// `std::sync::Mutex` poisons on a panic in a critical section, and the sections here are a map
    /// lookup, a map write and counter increments — no user code, no allocation that can fail
    /// without aborting. If one were poisoned anyway, a cache is not a reason to fail every later
    /// request: the store is read as it stands, which is at worst a miss.
    fn shared(&self) -> MutexGuard<'_, Shared> {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The entry under `key`, when it is answerable under this handle's declaration.
    ///
    /// A hit polls the budget first: serving a fact from memory is not a way past a cancellation or
    /// an expired clock, and the request that asked for it still has to terminate.
    fn take(&self, key: FactsKey, budget: &mut Budget) -> Result<Option<FactsPayload>> {
        budget.poll()?;
        let mut guard = self.shared();
        let shared = &mut *guard;
        shared.counters.consultations += 1;
        let Some(entry) = shared.entries.get(&key) else {
            shared.counters.misses += 1;
            return Ok(None);
        };
        if let Some(reason) = self.unusable(&key, entry) {
            // The entry is *discarded*, not left for the next lookup: a version it was written
            // under is not a state a later request could find it answerable in.
            shared.entries.remove(&key);
            shared.counters.misses += 1;
            match reason {
                Unusable::Format => shared.counters.discarded_format += 1,
                Unusable::Registry => shared.counters.discarded_registry += 1,
                Unusable::Product => shared.counters.discarded_product += 1,
            }
            return Ok(None);
        }
        let payload = entry.payload.clone();
        shared.counters.hits += 1;
        Ok(Some(payload))
    }

    /// Why an entry found under `key` is not answerable under this handle's declaration.
    ///
    /// The content identity is the key, so there is no second copy of it to compare: a lookup for
    /// other content cannot find this entry at all. What is checked here is the entry's
    /// **declaration** (the format and registry version it was written under) and its **product**
    /// (the payload kind its policy says it holds).
    fn unusable(&self, key: &FactsKey, entry: &Entry) -> Option<Unusable> {
        if entry.identity.format != self.identity.format {
            return Some(Unusable::Format);
        }
        if entry.identity.registry != self.identity.registry {
            return Some(Unusable::Registry);
        }
        if !key.policy.holds(&entry.payload) {
            return Some(Unusable::Product);
        }
        None
    }

    /// Why a container product is not answerable under this handle's declaration.
    ///
    /// The schema is part of the key, so it cannot disagree; what can is the entry format and the
    /// registry version the directory was validated under.
    fn unusable_container(&self, entry: &ContainerEntry) -> Option<Unusable> {
        if entry.identity.format != self.identity.format {
            return Some(Unusable::Format);
        }
        if entry.identity.registry != self.identity.registry {
            return Some(Unusable::Registry);
        }
        None
    }

    /// Whether one more answer of `weight` fits in the store, counting `items` answers already
    /// retained. The two bounds are independent: an answer has to fit in both.
    fn fits(shared: &Shared, weight: u64) -> bool {
        let items = shared.entries.len() + shared.containers.len();
        items < shared.capacity.entries
            && weight <= shared.capacity.retained_bytes
            && shared.retained_bytes.saturating_add(weight) <= shared.capacity.retained_bytes
    }

    /// Writes one complete CP/Header answer, or refuses because the store is full.
    ///
    /// The weight of a CP/Header payload is the length of the class bytes it was parsed from: the
    /// payload is a function of those bytes and is not held anywhere else, so that length is the
    /// honest proxy for the residency it adds. It is not an allocation measurement.
    fn keep(&self, key: FactsKey, payload: FactsPayload) {
        let weight = key.length;
        let entry = Entry {
            identity: self.identity,
            payload,
            weight,
        };
        let mut guard = self.shared();
        let shared = &mut *guard;
        match shared.entries.get(&key).map(|existing| existing.weight) {
            Some(previous) => {
                // A key the store already holds may always be rewritten — the parse that reached
                // here ran to the end — and only a *new* key can be refused.
                shared.retained_bytes = shared.retained_bytes.saturating_sub(previous);
                shared.retained_bytes = shared.retained_bytes.saturating_add(weight);
                shared.entries.insert(key, entry);
                shared.counters.stored += 1;
            }
            None => {
                if !Self::fits(shared, weight) {
                    if shared.entries.len() + shared.containers.len() >= shared.capacity.entries {
                        shared.counters.refused_capacity += 1;
                    } else {
                        shared.counters.refused_capacity_bytes += 1;
                    }
                    return;
                }
                shared.retained_bytes += weight;
                shared.entries.insert(key, entry);
                shared.counters.stored += 1;
            }
        }
    }
}
