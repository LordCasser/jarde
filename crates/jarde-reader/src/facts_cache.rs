//! The **CP/Header layer** of P5's `facts-cache` (task 2.3): an in-memory store of the class-file
//! facts a request parses, keyed by the semantic inputs of that parse.
//!
//! P5 2.1 measured the direct path and 2.2 decided what a cache would have to bind; this module is
//! the first layer of that record built, under the boundary the change documents fix: **in memory
//! only** (no file, no format on disk, no migration), **no third-party dependency**, **disabled
//! unless a caller attaches one**, and **no concurrency**: the engine is one sequential scan and
//! nothing here spawns work.
//!
//! ## What this layer holds, and what it deliberately does not
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
//! ## What a hit is, and what it can never be
//!
//! * **The read still happens.** The cache sits *above* the bounded read and *below* every consumer:
//!   the bytes are materialized, charged, CRC-checked and digested exactly as the direct path does,
//!   and only the parse is skipped. Coverage, origins, read evidence and the entry's identity are
//!   therefore produced by the same code on both paths, and a hit cannot claim a range nobody read.
//! * **A hit charges no counted dimension.** It reads, decodes and derives nothing; the dimensions
//!   that measure those steps stay where they are, and the saving is what the comparison reports. A
//!   hit still calls [`Budget::poll`], so a cancelled or expired request terminates through the
//!   cache exactly as it does without one.
//! * **Nothing negative, partial or refused is ever stored.** A parse that stops (budget,
//!   cancellation, a strict version refusal) writes nothing, so an incomplete answer can never stand
//!   in for a complete one and a negative verdict can never be replayed after the reason for it
//!   changed. The two products here are refused-or-absent, never "no such class".
//! * **Content is shared, origins are not.** Two entries, two snapshots or two loaders that hold the
//!   same bytes share one entry; what comes back is a value that carries no origin, and the caller
//!   binds the physical origin it read from. A cache cannot merge two origins because it never held
//!   one.
//!
//! ## Versions, damage and the fallback
//!
//! Every entry records the [`FactsIdentity`] it was written under. A lookup that finds an entry
//! written under another **entry format** or another **registry version** discards it, counts the
//! discard and returns "not answered by the cache" — the caller then runs the direct path under the
//! budget it already holds. There is no second attempt, no re-run and no budget reset: the fallback
//! is the same bounded parse the cache would have replaced.

use crate::budget::Budget;
use crate::classfile::{ClassFacts, HeaderInspection, InspectionMode};
use crate::error::Result;
use crate::model::Digest;
use crate::release_registry::HIGHEST_REGISTERED_MAJOR;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry as MapEntry;
use std::sync::{Arc, Mutex, MutexGuard};

/// The entry format this build writes.
///
/// An entry carries the format it was written in, and a lookup discards one written in another: a
/// store that outlived a change to what an entry holds is read as "this entry is not answerable"
/// rather than as the answer. The number is this build's declaration, not a compatibility promise.
pub const FACTS_FORMAT: u16 = 1;

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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Counters {
    consultations: u64,
    hits: u64,
    misses: u64,
    stored: u64,
    refused_capacity: u64,
    discarded_format: u64,
    discarded_registry: u64,
    discarded_product: u64,
}

/// The entries and their counters, shared by every handle that reads them.
#[derive(Debug)]
struct Shared {
    capacity: usize,
    entries: BTreeMap<FactsKey, Entry>,
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
    pub capacity: usize,
    pub entries: usize,
    /// Lookups asked of the store. `hits + misses`; a discarded entry counts as a miss.
    pub consultations: u64,
    pub hits: u64,
    pub misses: u64,
    /// Entries written by a parse that ran to the end. A parse that stopped writes nothing, which
    /// is why no counter here can say "an incomplete answer was stored".
    pub stored: u64,
    /// Stores refused because the store already held `capacity` entries: the facts stayed uncached
    /// and the request was served by its own parse.
    pub refused_capacity: u64,
    /// Entries *discarded* because they were written in another entry format.
    pub discarded_format: u64,
    /// Entries discarded because they were parsed under another registry version.
    pub discarded_registry: u64,
    /// Entries discarded because they did not hold the product their policy names.
    pub discarded_product: u64,
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
    /// A fresh store, read under `identity`, holding at most `capacity` entries.
    ///
    /// The bound is on **entries**, not on bytes: the payload carries no size accounting, and a byte
    /// bound would need one the reader does not keep. What bounds a byte worth of entries today is
    /// the request that filled them — a parse only happens under a budget that admitted the class —
    /// so the ceiling is `capacity × the largest class a request was allowed to read`.
    pub fn new(identity: FactsIdentity, capacity: usize) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared {
                capacity,
                entries: BTreeMap::new(),
                counters: Counters::default(),
            })),
            identity,
        }
    }

    /// A fresh store read under this build's identity.
    pub fn current(capacity: usize) -> Self {
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

    pub fn capacity(&self) -> usize {
        self.shared().capacity
    }

    /// A stable one-line label for this handle's declaration. Part of the measurement context, so it
    /// carries no counter: two runs of one configuration describe their cache the same way.
    pub fn describe(&self) -> String {
        format!(
            "facts cache ({}), capacity {} entries",
            self.identity.describe(),
            self.capacity()
        )
    }

    pub fn report(&self) -> FactsReport {
        let guard = self.shared();
        let shared = &*guard;
        let counters = shared.counters;
        FactsReport {
            identity: self.identity,
            capacity: shared.capacity,
            entries: shared.entries.len(),
            consultations: counters.consultations,
            hits: counters.hits,
            misses: counters.misses,
            stored: counters.stored,
            refused_capacity: counters.refused_capacity,
            discarded_format: counters.discarded_format,
            discarded_registry: counters.discarded_registry,
            discarded_product: counters.discarded_product,
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

    /// Writes one complete answer, or refuses because the store is full.
    fn keep(&self, key: FactsKey, payload: FactsPayload) {
        let entry = Entry {
            identity: self.identity,
            payload,
        };
        let mut guard = self.shared();
        let shared = &mut *guard;
        // Read the bound before the entry borrow: a key the store already holds may always be
        // rewritten — the parse that reached here ran to the end — and only a *new* key can be
        // refused.
        let full = shared.entries.len() >= shared.capacity;
        match shared.entries.entry(key) {
            MapEntry::Occupied(mut occupied) => {
                occupied.insert(entry);
                shared.counters.stored += 1;
            }
            MapEntry::Vacant(vacant) => {
                if full {
                    shared.counters.refused_capacity += 1;
                    return;
                }
                vacant.insert(entry);
                shared.counters.stored += 1;
            }
        }
    }
}
