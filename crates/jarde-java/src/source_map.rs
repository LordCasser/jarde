//! The presentation-side origin of one node and the segment table the emitter produces beside the
//! text (P3 1.2 decision 2: the carrier 3.2 grows on).
//!
//! # Why this is not the reader's `OriginSet`
//!
//! The reader's `OriginSet` is a *fact*: a set of class-file ranges and method points that says
//! where something was read from, with no notion of which member is the node's own and which one
//! the node merely reproduces. A Java presentation needs that notion — `other.count` is the field
//! access at BCI 30 *and* the accessor BCI 55 it was derived from — and it needs it per node,
//! because the segment table answers "which text came from where", not "which bytes were read for
//! this method". So the recovery layer states its own pair: [`Origin`] (one anchor, with
//! provenance) and [`OriginSet`] (the node's own anchor plus the ones it presents).
//!
//! # What a segment is keyed by
//!
//! **Generated text bytes**, not line/column: this emitter never reflows, so a byte range is the
//! position of the node in the produced artifact and needs no second coordinate system that could
//! disagree with the first. `start..end` is a half-open range into the emitted text, and
//! [`Segment::text`] slices it back out.
//!
//! # Provenance
//!
//! [`Provenance::Direct`] — the text this node carries is produced by the bytecode the origin
//! names: an instruction's own BCI anchors the statement it became.
//! [`Provenance::Derived`] — the text is a presentation of evidence anchored somewhere else: the
//! `if` statement is anchored at its branch BCI while the condition expression it prints is
//! derived from the same BCI (the branch tests the value it read), and a declaration derived from
//! the first store that fills the slot carries both anchors. `derived` is a list and not a field
//! because 3.2 lands the accessor/lambda second origin exactly there.
//!
//! # The CP field
//!
//! [`Origin::cp`] is `None` for an anchor whose producing rule had no constant-pool index to
//! record, and that is a statement about the evidence rather than a placeholder somebody forgot to
//! fill: a `ldc`-less instruction names no pool entry, and the 1.1 IR handoff publishes none at
//! all, so a run that reads only the payload has none to record. Where a rule does hold one — the
//! `invokedynamic` site of a lambda (`lambda@1`) — the anchor carries it.
//!
//! # The member an anchor belongs to (P3 3.2)
//!
//! A bytecode index is not a place by itself: BCI 3 of the presented body and BCI 3 of a callee are
//! different instructions, and a CP index means nothing without the pool that holds it. So every
//! anchor states the **member** it belongs to beside its index — the reader's own
//! [`PhysicalMethodId`], which names the member *and* the class-file definition (its bytes' digest
//! and length) those bytes are. That pair is exactly the reader's
//! [`OriginMember::MethodPoint`] coordinate, carried here without the enum's class-file forms
//! because the class-file coordinate this table records is the CP index above.
//!
//! [`Origin::method`] is `None` for exactly one kind of anchor: one of the body this run presents
//! whose own member declaration the run did not read (`raw_facts` located no member header), so the
//! run has no identity to state and invents none. An anchor a rule presented from *another*
//! member's body always states one — that member's body was read, so its identity is in hand, and
//! that is what tells "the call site at BCI 3 here" from "the field access at BCI 1 inside
//! `access$100`".

use std::collections::BTreeSet;

use jarde_reader::model::{OriginMember, PhysicalMethodId};

/// Where one anchor of a node's text comes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize)]
pub enum Provenance {
    /// The node's own bytecode produced this text.
    Direct,
    /// The text presents evidence anchored here; the node's own anchor is another one.
    Derived,
}

/// One anchor: a bytecode index inside one member body, the constant-pool entry naming it when the
/// run that produced the node had one, and which member body that is.
///
/// The member is boxed for the same reason the identities of the reader's own coordinate types are
/// carried behind a pointer wherever one is held per node: a [`PhysicalMethodId`] states a whole
/// class-file definition (its location, the digest and length of its bytes, its variant) beside the
/// member's two names, and inlining that in every anchor would make every statement and expression
/// of the tree carry it. The anchor's own reads are unaffected — [`Origin::method`] answers with the
/// member itself.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize)]
pub struct Origin {
    bci: u32,
    method: Option<Box<PhysicalMethodId>>,
    cp: Option<u16>,
    provenance: Provenance,
}

impl Origin {
    /// The direct anchor of a node's own instruction, in the member body the run stated for the
    /// presented member ([`Origin::in_method`]; `None` when it stated none).
    pub fn direct(bci: u32) -> Self {
        Self {
            bci,
            method: None,
            cp: None,
            provenance: Provenance::Direct,
        }
    }

    /// An anchor the node presents but does not own; see [`Provenance::Derived`].
    pub fn derived(bci: u32) -> Self {
        Self {
            bci,
            method: None,
            cp: None,
            provenance: Provenance::Derived,
        }
    }

    /// The same anchor, stating the member body its bytecode index is in.
    ///
    /// A rule that presents another member's body — the accessor whose field access is inside the
    /// callee — states that member here, and the run's own declaration states the presented one for
    /// every anchor that belongs to it.
    pub fn in_method(mut self, method: &PhysicalMethodId) -> Self {
        self.method = Some(Box::new(method.clone()));
        self
    }

    /// The same anchor, stating `method` when it states no member at all.
    ///
    /// This is how the body of one run is stated once for all of its own anchors: the anchors a rule
    /// built name no member (they cannot know the run's declaration), and the one member the whole
    /// artifact is of is the request's own.
    pub(crate) fn in_body(self, method: Option<&PhysicalMethodId>) -> Self {
        match (&self.method, method) {
            (None, Some(method)) => self.in_method(method),
            _ => self,
        }
    }

    /// The same anchor, naming the constant-pool entry behind it.
    pub fn with_cp(self, cp: u16) -> Self {
        Self {
            cp: Some(cp),
            ..self
        }
    }

    /// The bytecode index this anchor names.
    pub fn bci(&self) -> u32 {
        self.bci
    }

    /// The member body this anchor's bytecode index is in, when the run stated one: the member's own
    /// identity, and with it the class-file definition whose bytes (and whose constant pool) the
    /// index and [`Origin::cp`] are coordinates in.
    pub fn method(&self) -> Option<&PhysicalMethodId> {
        self.method.as_deref()
    }

    /// The same anchor as the physical coordinate the reader states: a method point, or `None` when
    /// the run stated no member for this anchor.
    pub fn member(&self) -> Option<OriginMember> {
        self.method().map(|method| OriginMember::MethodPoint {
            method: method.clone(),
            bci: self.bci,
        })
    }

    /// The constant-pool index behind this anchor, when the producing run had one.
    pub fn cp(&self) -> Option<u16> {
        self.cp
    }

    /// Whether this anchor is the node's own or one of the ones it presents.
    pub fn provenance(&self) -> Provenance {
        self.provenance
    }
}

/// One AST node's anchors: the instruction the node is, plus the evidence its text presents.
///
/// The two are kept apart because they answer different questions — "where does this statement
/// come from" is the primary, "what else does this text reproduce" is the derived list — and
/// because the derived list is where 3.2's double origin (the accessor call site *and* the field
/// BCI) belongs. A node with one anchor is the ordinary case.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct OriginSet {
    primary: Origin,
    derived: Vec<Origin>,
}

impl OriginSet {
    /// A node anchored at one direct origin.
    pub fn new(primary: Origin) -> Self {
        Self {
            primary,
            derived: Vec::new(),
        }
    }

    /// A node anchored at one direct origin, presenting a further anchor.
    pub fn derived_from(primary: Origin, derived: Origin) -> Self {
        Self {
            primary,
            derived: vec![derived],
        }
    }

    /// The anchor the node itself is.
    pub fn primary(&self) -> &Origin {
        &self.primary
    }

    /// The anchors the node's text presents without owning, in first-appearance order.
    pub fn derived(&self) -> &[Origin] {
        &self.derived
    }

    /// The same set with one more presented anchor; exact repeats — and the primary itself — are
    /// dropped, because a node cannot present its own anchor as evidence for itself.
    pub fn plus_derived(mut self, origin: Origin) -> Self {
        if origin != self.primary && !self.derived.contains(&origin) {
            self.derived.push(origin);
        }
        self
    }

    /// The same anchors, each stating `method` when it states no member of its own.
    ///
    /// The one writer that knows what the presented body *is* — the request's own declaration —
    /// states it here for every anchor that belongs to that body, and leaves the anchors a rule
    /// presented from another member's body exactly as the rule read them (P3 3.2).
    pub(crate) fn in_body(&self, method: Option<&PhysicalMethodId>) -> Self {
        Self {
            primary: self.primary.clone().in_body(method),
            derived: self
                .derived
                .iter()
                .map(|origin| origin.clone().in_body(method))
                .collect(),
        }
    }

    /// Every bytecode index this node mentions, whichever provenance carries it.
    pub fn bcis(&self) -> BTreeSet<u32> {
        let mut bcis = BTreeSet::new();
        bcis.insert(self.primary.bci);
        bcis.extend(self.derived.iter().map(|origin| origin.bci));
        bcis
    }

    /// How this set mentions one bytecode index, primary beating derived.
    pub fn mentions(&self, bci: u32) -> Option<Provenance> {
        if self.primary.bci == bci {
            return Some(Provenance::Direct);
        }
        self.derived
            .iter()
            .find(|origin| origin.bci == bci)
            .map(|_| Provenance::Derived)
    }
}

/// One node's span in the produced text, with the anchors that produced it.
///
/// `start..end` is a half-open byte range into the emitted text: `end` is one past the last byte
/// the node wrote, so segments of adjacent nodes abut without overlapping.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Segment {
    start: usize,
    end: usize,
    origin: OriginSet,
}

impl Segment {
    /// A segment over one half-open byte range of the produced text.
    pub fn new(start: usize, end: usize, origin: OriginSet) -> Self {
        Self { start, end, origin }
    }

    /// The first byte of the node's text.
    pub fn start(&self) -> usize {
        self.start
    }

    /// One past the last byte of the node's text.
    pub fn end(&self) -> usize {
        self.end
    }

    /// The number of bytes the node wrote.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Whether the node wrote nothing; such a node is never in the table.
    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }

    /// The anchors behind this node's text.
    pub fn origin(&self) -> &OriginSet {
        &self.origin
    }

    /// The node's own text, sliced out of the artifact the emitter produced.
    ///
    /// Panics when `text` is not the text this segment was recorded over — the one way a segment
    /// table can be wrong without being detected, since the table states byte ranges and a table
    /// recorded against other text would otherwise slice silently.
    pub fn text<'t>(&self, text: &'t str) -> &'t str {
        &text[self.start..self.end]
    }

    /// How this node's anchors mention one bytecode index.
    pub fn mentions(&self, bci: u32) -> Option<Provenance> {
        self.origin.mentions(bci)
    }
}

/// The segment table of one produced artifact: every node's byte range, in completion order.
///
/// A node's span is recorded when the node's own writes finish, so a node that contains others — a
/// statement holding its expression — appears *after* the nodes it contains, and a consumer reading
/// the table finds the most specific node first. That order is also the order a consumer scans it in
/// when it asks "which node covered this byte". The table *is* the source map (P3 decision 3):
/// nothing here is recovered from the text afterwards, because the ranges are recorded by the same
/// writes that produce the text.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize)]
pub struct SourceMap {
    segments: Vec<Segment>,
}

impl SourceMap {
    /// The whole table, in completion order: a node's own span follows the spans it contains.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Whether the artifact holds no mapped node at all.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// The number of nodes the table maps.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// The node whose text covers one byte of the artifact.
    pub fn covering(&self, byte: usize) -> Option<&Segment> {
        self.segments
            .iter()
            .find(|segment| segment.start <= byte && byte < segment.end)
    }

    /// Every node that mentions one bytecode index, in writing order — the derived ones included,
    /// which is how "the same BCI reached two nodes" is answered.
    pub fn of_bci(&self, bci: u32) -> Vec<&Segment> {
        self.segments
            .iter()
            .filter(|segment| segment.mentions(bci).is_some())
            .collect()
    }

    /// Every node this bytecode index anchors directly.
    pub fn direct_of_bci(&self, bci: u32) -> Vec<&Segment> {
        self.segments
            .iter()
            .filter(|segment| segment.mentions(bci) == Some(Provenance::Direct))
            .collect()
    }

    /// Every node that presents this bytecode index without owning it.
    pub fn derived_of_bci(&self, bci: u32) -> Vec<&Segment> {
        self.segments
            .iter()
            .filter(|segment| segment.mentions(bci) == Some(Provenance::Derived))
            .collect()
    }

    /// The text one node covers, sliced out of the artifact.
    pub fn segment_text<'t>(&self, text: &'t str, segment: &Segment) -> &'t str {
        segment.text(text)
    }

    /// The text of every node that mentions this bytecode index, in writing order.
    pub fn text_of_bci<'t>(&self, text: &'t str, bci: u32) -> Vec<&'t str> {
        self.of_bci(bci)
            .into_iter()
            .map(|segment| segment.text(text))
            .collect()
    }

    /// Records one node's span. Crate-private: the only writer is the emitter, in the same
    /// mechanism that wrote the bytes.
    pub(crate) fn record(&mut self, segment: Segment) {
        debug_assert!(
            !segment.is_empty(),
            "an empty node wrote no text and has no span to map"
        );
        self.segments.push(segment);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_node_keeps_its_own_anchor_apart_from_the_one_it_presents() {
        let set = OriginSet::derived_from(Origin::direct(30), Origin::derived(55));
        assert_eq!(
            set.mentions(30),
            Some(Provenance::Direct),
            "the field access"
        );
        assert_eq!(set.mentions(55), Some(Provenance::Derived), "its accessor");
        assert_eq!(set.mentions(7), None, "and nothing else");
        assert_eq!(set.bcis(), BTreeSet::from([30, 55]));

        let set = set
            .plus_derived(Origin::derived(55))
            .plus_derived(Origin::direct(30));
        assert_eq!(set.derived().len(), 1, "one presented anchor, kept once");
    }

    #[test]
    fn the_table_answers_by_bci_and_by_byte() {
        let text = "other.count + 1;";
        let mut map = SourceMap::default();
        map.record(Segment::new(
            0,
            text.len(),
            OriginSet::derived_from(Origin::direct(30), Origin::derived(55)),
        ));
        assert_eq!(map.covering(3).map(Segment::len), Some(text.len()));
        assert_eq!(map.covering(text.len()), None, "the table is half-open");
        assert_eq!(
            map.text_of_bci(text, 55),
            vec![text],
            "the derived anchor reaches the same text"
        );
        assert!(map.direct_of_bci(55).is_empty());
        assert_eq!(map.derived_of_bci(30).len(), 0, "30 is the node's own");
    }

    #[test]
    fn cp_is_optional_and_survives_the_set() {
        let anchor = Origin::direct(12).with_cp(9);
        assert_eq!(anchor.cp(), Some(9));
        assert_eq!(Origin::derived(13).cp(), None);
    }

    /// One member's identity, as the run that read it states it: the class-file definition and the
    /// member's own name and descriptor (both spelled raw here, as the class file does).
    fn method(definition: &str, name: &str, descriptor: &str) -> PhysicalMethodId {
        use jarde_reader::model::{
            ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
            PhysicalVariant, SnapshotId,
        };
        PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: SnapshotId(definition.to_string()),
                },
                class_bytes: ClassBytesId {
                    digest: Digest(format!("{definition}-digest")),
                    length: 7,
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        }
    }

    #[test]
    fn an_anchor_says_which_member_body_its_bytecode_index_is_in() {
        // The two shapes A12 cannot tell apart from the BCI alone: the call site in the presented
        // body and the field access at the **same** BCI inside the callee's body, and two callees
        // whose field access sits at one BCI each.
        let caller = method("snapshot", "method", "()I");
        let accessor = method("snapshot", "access$100", "(LTest;)I");
        let other = method("snapshot", "access$200", "(LTest;)I");

        let call_site = Origin::direct(3).in_method(&caller);
        let field_in_accessor = Origin::derived(3).in_method(&accessor);
        let field_in_other = Origin::derived(3).in_method(&other);
        assert_eq!(call_site.bci(), field_in_accessor.bci(), "the same BCI");
        assert_ne!(call_site, field_in_accessor, "in two different members");
        assert_ne!(
            field_in_accessor, field_in_other,
            "and two callees at one BCI are two anchors"
        );
        assert_eq!(
            field_in_accessor.method().map(|method| &method.name.0),
            Some(&b"access$100".to_vec()),
            "the anchor's own method answers which member it is"
        );

        // The reader's own coordinate for the same place: a method point, with the definition the
        // member — and therefore its constant pool — belongs to.
        assert_eq!(
            field_in_accessor.member(),
            Some(OriginMember::MethodPoint {
                method: accessor.clone(),
                bci: 3,
            })
        );
        assert_eq!(
            field_in_accessor.method().map(|method| &method.owner),
            field_in_other.method().map(|method| &method.owner),
            "both callees are declared in one definition, which is what binds their CP indices"
        );

        // An anchor of the body the run presents carries that body's identity once the run states
        // it — and stays unstated when the run read no declaration for it.
        let set = OriginSet::derived_from(Origin::direct(3), Origin::derived(3));
        assert_eq!(set.primary().method(), None, "no member was stated");
        // The set one accessor node carries: its own anchor in the presented body, and the callee's
        // field access as the member it really is in. Stating the presented body leaves the callee's
        // anchor alone, because an anchor that already names a member is never restated.
        let node = OriginSet::new(Origin::direct(3))
            .plus_derived(Origin::derived(1).in_method(&accessor))
            .in_body(Some(&caller));
        assert_eq!(node.primary().method(), Some(&caller));
        assert_eq!(node.derived()[0].method(), Some(&accessor));
        assert_eq!(node.derived()[0].bci(), 1);
    }
}
