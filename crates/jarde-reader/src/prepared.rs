//! One trusted class, read once, for every method a class task asks for.
//!
//! A class is the scheduling unit of a bulk recovery operation and the method is its delivery unit
//! (change `add-parallel-bulk-recovery`, decision 2). That makes one class's *preparation* a shared
//! product: its declaration, its constant pool, its member table and the raw-name locator over that
//! table are read once and every method of that class is decoded against them. This module is that
//! lifecycle.
//!
//! # What a prepared class guarantees
//!
//! * **One preparation.** [`PreparedClass::prepare`] reads the class once — its declaration, its
//!   constant pool, its member table and the parser that read them — and holds the method records it
//!   read. No body read rebuilds a parser, re-scans the member table or re-reads the container
//!   directory: the number of body decodes equals the number of methods actually asked for, and a
//!   class with `N` methods costs one class read however many of them are decoded.
//! * **One decoder.** [`PreparedClass::method_code`] decodes through `classfile::decode_method_code`,
//!   the same function [`classfile::method_code_facts`] delegates to, so a body decoded here and the
//!   same body decoded by the single-method entry point cannot drift in facts, in stop position or
//!   in what they charge (`AttributeBytes` for the `Code` shell, `CodeBytes` per instruction width,
//!   no `ClassBytes`, no `ResultItems`).
//! * **No upgraded conclusion.** An incomplete member table is never turned into a per-method body
//!   verdict: [`MethodCodeAttribute::Unreadable`] carries the [`MemberTableStop`] that ended the
//!   walk, and [`PreparedClass::member_table_stop`] publishes it on the class — never `Absent`.
//! * **No collapsed candidates.** [`PreparedClass::locate_method`] returns *every* ordinal whose raw
//!   name and descriptor are the requested ones, in declaration order. Two declarations of one
//!   signature stay two declarations.
//! * **Trusted bytes.** A [`PreparedClassRead`] comes from an [`ArtifactSnapshot`] method that
//!   verifies the entry exactly as the materializing read does (central-directory record cross-check
//!   on raw name, offset, CRC and sizes, then the entry's own verified read) and hands out a strong
//!   reference to the container backing. Loader, profile and version gates are deliberately **not**
//!   applied here: those belong to `jarde-jvm` and the facade, which keep applying them to the same
//!   bytes.
//!
//! # What it deliberately does not do
//!
//! A prepared class is not a cache and not a second reader. It does not consult the facts cache (one
//! class is prepared once per class task and its facts are shared by reference, so a lookup keyed by
//! a re-hash of the same bytes would buy nothing here), it does not decode a body nobody asked for,
//! and it does not turn its facts into an analysis conclusion: dialect validation, verification and
//! source recovery remain separate facts of the layers above.
//!
//! [`ArtifactSnapshot`]: crate::artifact::ArtifactSnapshot

use crate::budget::Budget;
use crate::classfile::{
    self, AttributeShell, ClassFacts, MemberHeader, MemberTableStop, MethodCodeFacts,
    decode_method_code,
};
use crate::error::{Error, Result};
use crate::model::{ByteSpan, ClassBytesId, ContainerId, Digest, JvmString, PhysicalClassLocation};
use noak::reader::{Attribute, Class, Method};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Where one class's bytes live inside a container that is held as shared backing.
///
/// The value is produced by [`crate::artifact::ArtifactSnapshot::prepared_class`] (or by its
/// standalone-root equivalent, [`crate::artifact::ArtifactSnapshot::prepared_root_class`]) and never
/// assembled by a caller: its fields are the evidence that read established, and its backing is the
/// strong reference that keeps the container's bytes alive for as long as the class task runs — even
/// if the facts cache is cleared or refuses admission in the meantime.
///
/// `span` indexes [`PreparedClassRead::backing`] and yields the class bytes. That holds for both
/// shapes a class entry has: an entry stored without compression lives inside the container backing
/// the read already holds, and an entry that has to be decompressed is handed back as the backing of
/// its own (the bytes the verified read produced). [`PreparedClassRead::backing_digest`] is the
/// trusted content identity of whichever backing that is — the container's own digest when nothing
/// was copied, the class's content digest when the read produced the bytes.
#[derive(Clone, Debug)]
pub struct PreparedClassRead {
    /// The physical position of the class: an archive entry, or the root of a standalone snapshot.
    pub location: PhysicalClassLocation,
    /// Trusted digest and length of the class entry's own bytes.
    pub class_bytes: ClassBytesId,
    /// The container the entry lives in.
    pub container: ContainerId,
    /// How deep that container is: 0 for a snapshot's root, 1 for its direct child, and so on.
    pub depth: u64,
    /// The backing [`PreparedClassRead::span`] indexes.
    pub backing: Arc<[u8]>,
    /// The class bytes' range inside [`PreparedClassRead::backing`].
    pub span: ByteSpan,
    /// Trusted content identity of `backing`, established by the read that materialized it.
    backing_digest: Digest,
}

impl PreparedClassRead {
    /// The class bytes.
    ///
    /// For a read this crate produced the span is always inside the backing, so this is the bytes
    /// the entry holds. A value this crate did not produce is refused by [`PreparedClass::prepare`]
    /// rather than silently read as an empty class.
    pub fn bytes(&self) -> &[u8] {
        let start = usize::try_from(self.span.start).unwrap_or(usize::MAX);
        let end = usize::try_from(self.span.length)
            .ok()
            .and_then(|length| start.checked_add(length))
            .unwrap_or(usize::MAX);
        self.backing.get(start..end).unwrap_or(&[])
    }

    /// Trusted content identity of the backing these bytes live in.
    ///
    /// For a class entry read out of the container's own backing this is the container's digest, so
    /// two classes of one container share it and neither costs a second hash; for a class entry
    /// whose bytes had to be produced (a deflated entry) it is the digest of those bytes.
    pub fn backing_digest(&self) -> &Digest {
        &self.backing_digest
    }

    /// Builds one read out of the parts a snapshot path verified.
    pub(crate) fn new(
        location: PhysicalClassLocation,
        class_bytes: ClassBytesId,
        container: ContainerId,
        depth: u64,
        backing: Arc<[u8]>,
        span: ByteSpan,
        backing_digest: Digest,
    ) -> Self {
        Self {
            location,
            class_bytes,
            container,
            depth,
            backing,
            span,
            backing_digest,
        }
    }
}

/// The parse policy a prepared class's structural facts were read under.
///
/// One variant, because a prepared class is always the *structural* read: the declaration, the
/// constant pool and the member table, with no version gate and no inspection mode applied. Those
/// belong to `classfile::inspect_header` and to the layers above this crate, and claiming one here
/// would be exactly the conclusion this read does not establish.
///
/// The value exists so a caller can bind the facts it holds to the parse policy they came from — the
/// input a facts-cache key made of a trusted identity needs ([`PreparedClass::class_bytes`],
/// [`PreparedClassRead::backing_digest`] and this). It is a type of its own rather than the facts
/// cache's key policy, which is crate-private and must stay optional: a prepared read is usable
/// without any cache existing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParsePolicy {
    /// The structural read: declaration, constant pool and member records, no version gate.
    Structure,
}

/// The ordinal of one method record inside its class, counted in declaration order from 0.
///
/// An ordinal is a position inside one class's own member table and nothing else. It is not a name,
/// it is not stable across two physical definitions of the same class, and it is never an identity
/// a caller may carry from one class to another: the physical identity of a method stays its owner's
/// [`PhysicalClassLocation`] plus its raw name and descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct MethodOrdinal(pub u32);

/// One method record of a prepared class and the state of its `Code` entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MethodSlot {
    /// Where this record sits in its class's declaration order.
    pub ordinal: MethodOrdinal,
    /// The raw name the record declares.
    pub name: JvmString,
    /// The raw descriptor the record declares.
    pub descriptor: JvmString,
    /// The record's access flags, exactly as the class file spells them.
    pub access_flags: u16,
    /// The whole record as the preparation walk read it — attribute **shells** only, never a body.
    pub header: MemberHeader,
    /// Which `Code` entry the record declares, or why that cannot be stated.
    pub code: MethodCodeAttribute,
}

/// What one method record states about its `Code` entry.
///
/// The four states are four different facts and no two of them may be collapsed into one:
///
/// * [`MethodCodeAttribute::Absent`] — the record was read to its declared end and declares no
///   `Code` entry. That is the normal shape of an abstract or native member, not a failure.
/// * [`MethodCodeAttribute::Single`] — exactly one `Code` entry, whose shell is carried here. A body
///   read resolves that entry and decodes it.
/// * [`MethodCodeAttribute::Duplicate`] — the record declares more than one `Code` entry, so no body
///   is defined for it; a body read answers `classfile_duplicate_code_attribute`.
/// * [`MethodCodeAttribute::Unreadable`] — the class's member table did not read to its declared
///   end, so this record's `Code` state is **not established**. The stop is carried here because an
///   incomplete read must not be reported as "no body": every slot of such a class states this
///   variant, and a body read refuses with the stop's own code instead of guessing.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MethodCodeAttribute {
    Absent,
    Single(AttributeShell),
    Duplicate,
    Unreadable(MemberTableStop),
}

/// One trusted, once-read class: declaration, constant pool, member table and a multi-valued method
/// locator, borrowing the read's backing.
///
/// The value borrows [`PreparedClassRead`] and never stores a self-reference inside an `Arc`: the
/// parser view noak hands out borrows the class bytes, so it stays a borrow of the read that is alive
/// for the whole class task, and only independently ownable payloads (the facts, the slots, the
/// locator) could ever cross an `Arc`.
///
/// A prepared class answers everything a method consumer of one class asks:
/// [`PreparedClass::class_facts`] for the shared structural payload,
/// [`PreparedClass::locate_method`] for the declaration-to-ordinal mapping, and
/// [`PreparedClass::method_code`] for a body.
pub struct PreparedClass<'a> {
    read: &'a PreparedClassRead,
    facts: ClassFacts,
    /// The parser view of this class, held when its structure decoded as a whole. `None` exactly
    /// when the member table stopped, in which case no slot states a decodable `Code` entry.
    parser: Option<PreparedParser<'a>>,
    slots: Vec<MethodSlot>,
    /// Raw name + descriptor -> every ordinal declaring it, sorted by that key. A lookup needs no
    /// allocation, and two equal declarations are two entries, never one.
    located: Vec<LocatedMethod>,
    /// How many field records the class file declares.
    field_count: u64,
    /// How many method records the class file declares.
    method_count: u64,
    /// Where the member-table read stopped, when it did not reach the declared end.
    stop: Option<MemberTableStop>,
}

/// The parser view one prepared class holds: the class's pool and its method records.
struct PreparedParser<'a> {
    class: Class<'a>,
    /// The method records, parallel to `PreparedClass::slots`.
    methods: Vec<Method<'a>>,
}

/// One locator entry: a raw name and descriptor, and every ordinal declaring them.
#[derive(Clone, Debug)]
struct LocatedMethod {
    name: Vec<u8>,
    descriptor: Vec<u8>,
    ordinals: Vec<MethodOrdinal>,
}

impl<'a> PreparedClass<'a> {
    /// Reads one trusted class once: its declaration, constant pool, member table and locator.
    ///
    /// The walk happens exactly once, and it is the walk the class's own body decodes then use. A
    /// class whose **declaration** does not decode — no class-file magic, an unmeasurable constant
    /// pool, a truncated fixed part, trailing bytes — is an `Err`, exactly as in the whole-structure
    /// reads. A class whose **member table** stops is `Ok`: the records the walk read become slots,
    /// the stop is published on the class and by every slot, and a body read refuses rather than
    /// guessing. A refusal of the request itself (an exceeded budget, a cancellation) is always an
    /// `Err` and never a stop, because that is the request ending rather than the class ending.
    pub fn prepare(read: &'a PreparedClassRead, budget: &mut Budget) -> Result<Self> {
        budget.poll()?;
        let bytes = read.bytes();
        if read.class_bytes.length != read.span.length {
            return Err(Error::invalid_input(
                "prepared_read_mismatch",
                "the trusted length of the class bytes does not match the span the read handed out",
            ));
        }
        match classfile::read_class_structure(bytes, budget) {
            Ok(structure) => Self::from_structure(read, structure),
            // A request-ending failure — budget, cancellation, I/O — ends this read too: it is not
            // damage in the class, and a second walk must not be started after a stop.
            Err(error) if !classfile::is_class_structure_damage(&error) => Err(error),
            Err(structure_error) => {
                // Damage: either the class's own declaration does not decode (the tolerant walk
                // fails too, and that failure is the answer) or a member record does, in which case
                // the walk publishes the reliable prefix and the stop.
                match classfile::read_class_member_prefix(bytes, budget)? {
                    Some(prefix) => Self::from_member_prefix(read, prefix),
                    None => Err(structure_error),
                }
            }
        }
    }

    /// Builds the prepared class of a structure that decoded as a whole.
    fn from_structure(
        read: &'a PreparedClassRead,
        structure: classfile::ClassStructure<'a>,
    ) -> Result<Self> {
        let classfile::ClassStructure {
            facts,
            class,
            methods,
        } = structure;
        let slots = Self::slots(&facts.methods, None)?;
        let located = locate_slots(&slots);
        let field_count = count(facts.fields.len())?;
        let method_count = count(facts.methods.len())?;
        Ok(Self {
            read,
            facts,
            parser: Some(PreparedParser { class, methods }),
            slots,
            located,
            field_count,
            method_count,
            stop: None,
        })
    }

    /// Builds the prepared class of a member table that stopped.
    ///
    /// Every slot states [`MethodCodeAttribute::Unreadable`]: the table did not read to its declared
    /// end, and a class whose structure did not decode as a whole has no parser view, so **no**
    /// method of it can be decoded. Reporting the readable prefix as `Absent` or `Single` would turn
    /// an incomplete read into a per-method conclusion, which is the one thing this state exists to
    /// prevent.
    fn from_member_prefix(
        read: &'a PreparedClassRead,
        prefix: classfile::ClassMemberPrefix,
    ) -> Result<Self> {
        let classfile::ClassMemberPrefix {
            facts,
            field_count,
            method_count,
            stop,
        } = prefix;
        let slots = Self::slots(&facts.methods, Some(&stop))?;
        let located = locate_slots(&slots);
        Ok(Self {
            read,
            facts,
            parser: None,
            slots,
            located,
            field_count,
            method_count,
            stop: Some(stop),
        })
    }

    /// One slot per method record, in declaration order.
    fn slots(headers: &[MemberHeader], stop: Option<&MemberTableStop>) -> Result<Vec<MethodSlot>> {
        let mut slots = Vec::with_capacity(headers.len());
        for (index, header) in headers.iter().enumerate() {
            let ordinal = MethodOrdinal(u32::try_from(index).map_err(|_| {
                Error::invalid_input("classfile_result_overflow", "method ordinal exceeds u32")
            })?);
            let code = match stop {
                Some(stop) => MethodCodeAttribute::Unreadable(stop.clone()),
                None => classify_code(header),
            };
            slots.push(MethodSlot {
                ordinal,
                name: header.name.clone(),
                descriptor: header.descriptor.clone(),
                access_flags: header.access_flags,
                header: header.clone(),
                code,
            });
        }
        Ok(slots)
    }

    /// The physical position of this class.
    pub fn location(&self) -> &PhysicalClassLocation {
        &self.read.location
    }

    /// The trusted identity of this class's bytes, as the read established them.
    pub fn class_bytes(&self) -> &ClassBytesId {
        &self.read.class_bytes
    }

    /// The class bytes themselves — exactly the span the read that prepared this class published.
    ///
    /// This is [`PreparedClassRead::bytes`] of the read this class borrows: the range `span` names
    /// inside the backing the read holds, verified against the entry's central-directory record by
    /// the read that produced it. It is handed out for the one question a prepared class cannot
    /// answer from its facts alone — the **content** of an attribute the class declares, such as the
    /// `BootstrapMethods` entry whose handle and argument indexes
    /// [`PreparedClass::class_facts`] carries — and it is the same bytes every body decode of this
    /// class is charged against, never a copy of them. Which definition these bytes *are* is the
    /// consumer's own check ([`PreparedClass::class_bytes`] plus
    /// [`PreparedClassRead::backing_digest`]), not a property this accessor asserts.
    pub fn bytes(&self) -> &[u8] {
        self.read.bytes()
    }

    /// The shared structural payload: declaration, constant pool and member records.
    ///
    /// This is the value every method consumer of one class shares — it is read once here and handed
    /// out by reference. When the member table stopped, these are the facts of the prefix the walk
    /// read ([`PreparedClass::member_table_stop`] says so, and
    /// [`PreparedClass::field_count`]/[`PreparedClass::method_count`] hold the counts the class file
    /// declares). Because the class attribute table follows the member tables, a stopped walk never
    /// reached it, so [`ClassFacts::attributes`] is empty and states nothing about that region.
    pub fn class_facts(&self) -> &ClassFacts {
        &self.facts
    }

    /// The parse policy these facts were read under.
    pub fn parse_policy(&self) -> ParsePolicy {
        ParsePolicy::Structure
    }

    /// The trusted content identity of the backing this class's bytes live in.
    pub fn backing_digest(&self) -> &Digest {
        self.read.backing_digest()
    }

    /// How many field records the class file declares.
    ///
    /// With no stop this equals `class_facts().fields.len()`; with a stop the class declares this
    /// many and the facts hold the prefix before it.
    pub fn field_count(&self) -> u64 {
        self.field_count
    }

    /// How many method records the class file declares; see [`PreparedClass::field_count`].
    pub fn method_count(&self) -> u64 {
        self.method_count
    }

    /// Where the member-table read stopped, or `None` when it read to the declared end.
    ///
    /// A caller that treats a locator answer as complete — "this class declares no such method" —
    /// has to consult this first: with a stop, the records after it were never read, so the remainder
    /// of the table is **unknown** rather than empty.
    pub fn member_table_stop(&self) -> Option<&MemberTableStop> {
        self.stop.as_ref()
    }

    /// Every method record this class read, in declaration order.
    pub fn method_slots(&self) -> &[MethodSlot] {
        &self.slots
    }

    /// The slot at one ordinal, or `None` when this class declares no such record.
    pub fn slot(&self, ordinal: MethodOrdinal) -> Option<&MethodSlot> {
        self.slots.get(usize::try_from(ordinal.0).ok()?)
    }

    /// Every method ordinal declaring this raw name and descriptor, in declaration order.
    ///
    /// The answer is the whole set: two declarations of one name and descriptor are two ordinals, and
    /// the locator never elects a first match. An empty answer means "no record *this read read*
    /// declares it" — [`PreparedClass::member_table_stop`] decides whether that is a complete claim
    /// about the class.
    pub fn locate_method(&self, name: &[u8], descriptor: &[u8]) -> &[MethodOrdinal] {
        match self.located.binary_search_by(|entry| {
            (entry.name.as_slice(), entry.descriptor.as_slice()).cmp(&(name, descriptor))
        }) {
            Ok(position) => &self.located[position].ordinals,
            Err(_) => &[],
        }
    }

    /// Decodes one located method body with the same implementation the single-method path uses.
    ///
    /// The charges are exactly `classfile::method_code_facts`'s for the same body: one
    /// `AttributeBytes` charge for the `Code` entry's shell, one `CodeBytes` charge per instruction
    /// width, no `ClassBytes` and no `ResultItems`. The errors are that entry point's too: a record
    /// whose raw name and descriptor are declared **more than once** is `classfile_method_ambiguous`
    /// — the same code the single-method entry point answers for that class, because the class does
    /// not distinguish two such declarations and electing one of them is exactly what the
    /// multi-valued locator exists to prevent — a record with no `Code` entry is
    /// `classfile_method_has_no_code`, and one with more than one is
    /// `classfile_duplicate_code_attribute`. Two states an ordinal can have and a raw name and
    /// descriptor cannot are the class's own: an ordinal this class does not declare is
    /// `classfile_method_not_found`, and a class whose member table stopped refuses with the stop's
    /// own code instead of decoding a body it cannot establish.
    pub fn method_code(
        &self,
        ordinal: MethodOrdinal,
        budget: &mut Budget,
    ) -> Result<MethodCodeFacts> {
        budget.poll()?;
        let index = usize::try_from(ordinal.0).map_err(|_| {
            Error::invalid_input(
                "classfile_result_overflow",
                "method ordinal does not fit usize",
            )
        })?;
        let slot = self.slot(ordinal).ok_or_else(|| {
            Error::invalid_input(
                "classfile_method_not_found",
                format!(
                    "this class declares no method record at ordinal {}",
                    ordinal.0
                ),
            )
        })?;
        // The ambiguity is decided from the locator the preparation built, not by decoding a first
        // match: two records under one signature stay two, and neither of them gets a body.
        if self
            .locate_method(
                slot.name.raw().0.as_slice(),
                slot.descriptor.raw().0.as_slice(),
            )
            .len()
            > 1
        {
            return Err(Error::invalid_input(
                "classfile_method_ambiguous",
                "multiple methods exactly matched the raw name and descriptor",
            ));
        }
        let shell = match &slot.code {
            MethodCodeAttribute::Absent => {
                return Err(Error::unsupported(
                    "classfile_method_has_no_code",
                    "selected method has no Code attribute",
                ));
            }
            MethodCodeAttribute::Duplicate => {
                return Err(Error::invalid_input(
                    "classfile_duplicate_code_attribute",
                    "selected method has multiple Code attributes",
                ));
            }
            MethodCodeAttribute::Unreadable(stop) => {
                return Err(stop_refusal(ordinal, stop));
            }
            MethodCodeAttribute::Single(shell) => shell,
        };
        let parser = match self.parser.as_ref() {
            Some(parser) => parser,
            // A slot states a decodable entry only when the class's structure decoded as a whole, so
            // this is a state `prepare` does not build; it is refused, never guessed at.
            None => {
                return Err(match self.stop.as_ref() {
                    Some(stop) => stop_refusal(ordinal, stop),
                    None => Error::invalid_input(
                        "classfile_method_header_mismatch",
                        "this class's structure did not decode as a whole, so no body can be read",
                    ),
                });
            }
        };
        let record = parser.methods.get(index).ok_or_else(|| {
            Error::invalid_input(
                "classfile_method_header_mismatch",
                format!(
                    "the parser holds no method record at ordinal {} that this class located",
                    ordinal.0
                ),
            )
        })?;
        let attribute =
            located_code_attribute(self.read.bytes(), parser.class.pool(), record, shell)?;
        decode_method_code(
            self.read.bytes(),
            parser.class.pool(),
            &slot.header,
            &attribute,
            budget,
        )
    }
}

/// How many records one table holds, as the count a report states.
fn count(records: usize) -> Result<u64> {
    u64::try_from(records)
        .map_err(|_| Error::invalid_input("classfile_result_overflow", "member count exceeds u64"))
}

/// Which `Code` entry one member record declares, read from the shells alone.
///
/// The shells are the whole evidence this needs: a `Code` entry costs its declaration and never its
/// body, so classifying a record reads no attribute content.
fn classify_code(header: &MemberHeader) -> MethodCodeAttribute {
    let mut code = header
        .attributes
        .iter()
        .filter(|shell| shell.name.raw().0.as_slice() == b"Code");
    match (code.next(), code.next()) {
        (None, _) => MethodCodeAttribute::Absent,
        (Some(shell), None) => MethodCodeAttribute::Single(shell.clone()),
        (Some(_), Some(_)) => MethodCodeAttribute::Duplicate,
    }
}

/// The locator built from one class's slots: one entry per declared raw name+descriptor, holding
/// every ordinal that declares it, in declaration order.
fn locate_slots(slots: &[MethodSlot]) -> Vec<LocatedMethod> {
    let mut collected: Vec<(&[u8], &[u8], MethodOrdinal)> = slots
        .iter()
        .map(|slot| {
            (
                slot.name.raw().0.as_slice(),
                slot.descriptor.raw().0.as_slice(),
                slot.ordinal,
            )
        })
        .collect();
    // Sorting by the key and then by the ordinal puts equal declarations next to each other and
    // keeps their ordinals ascending, so grouping in one pass preserves declaration order without
    // merging anything.
    collected.sort_by(|left, right| (left.0, left.1, left.2).cmp(&(right.0, right.1, right.2)));
    let mut located: Vec<LocatedMethod> = Vec::new();
    for (name, descriptor, ordinal) in collected {
        match located.last_mut() {
            Some(entry) if entry.name == name && entry.descriptor == descriptor => {
                entry.ordinals.push(ordinal);
            }
            _ => located.push(LocatedMethod {
                name: name.to_vec(),
                descriptor: descriptor.to_vec(),
                ordinals: vec![ordinal],
            }),
        }
    }
    located
}

/// The `Code` entry of one method record, as the parser decoded it, bound to the shell the
/// preparation walk read.
///
/// The shell and the parser's own record are two views of the same attribute list, and both are
/// needed: the shell names the entry this decode belongs to, and the parser's record is what the
/// decode reads. They are reconciled here instead of being trusted: the entry selected has to be the
/// one whose content **is** the shell's content and whose own name resolves to the shell's raw name,
/// and anything else — an entry that does not decode, an index that does not resolve, a record whose
/// attribute list disagrees with the header — is the existing `classfile_method_header_mismatch`
/// error rather than a silently wrong body.
fn located_code_attribute<'a>(
    bytes: &[u8],
    pool: &noak::reader::cpool::ConstantPool<'a>,
    record: &Method<'a>,
    shell: &AttributeShell,
) -> Result<Attribute<'a>> {
    let expected = classfile::attribute_slice(bytes, &shell.span, &shell.content_span)?;
    for attribute in record.attributes() {
        let Ok(attribute) = attribute else {
            break;
        };
        if attribute.content() != expected {
            continue;
        }
        let name_matches = pool
            .get(attribute.name())
            .is_ok_and(|name| name.content.as_bytes() == shell.name.raw().0.as_slice());
        if name_matches {
            return Ok(attribute);
        }
    }
    Err(Error::invalid_input(
        "classfile_method_header_mismatch",
        "the Code attribute of this method does not match the member header's shells",
    ))
}

/// The refusal a body read returns for a method of a class whose member table did not read
/// completely.
///
/// The stop's own code is reused — it is the failure that really happened — and the message names the
/// position it happened at, so a caller can point at the record rather than at a whole class.
fn stop_refusal(ordinal: MethodOrdinal, stop: &MemberTableStop) -> Error {
    Error::invalid_input(
        stop.code.clone(),
        format!(
            "method ordinal {} cannot be decoded: the member table stopped in the {:?} table at \
             record {} (class offset {}): {}",
            ordinal.0, stop.phase, stop.index, stop.class_offset, stop.message
        ),
    )
}
