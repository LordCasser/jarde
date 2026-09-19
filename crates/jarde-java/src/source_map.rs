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
//! [`Origin::cp`] is `None` throughout this slice, and that is a statement about the evidence
//! rather than a placeholder somebody forgot to fill: the 1.1 IR handoff publishes no
//! constant-pool index, so a run that reads only the payload has none to record. The field exists
//! so that 3.2's CP/attribute mapping can fill it without reshaping the table — adding an optional
//! CP later would change the type of every node the table holds.

use std::collections::BTreeSet;

/// Where one anchor of a node's text comes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Provenance {
    /// The node's own bytecode produced this text.
    Direct,
    /// The text presents evidence anchored here; the node's own anchor is another one.
    Derived,
}

/// One anchor: a bytecode index inside the method body, and the constant-pool entry naming it when
/// the run that produced the node had one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Origin {
    bci: u32,
    cp: Option<u16>,
    provenance: Provenance,
}

impl Origin {
    /// The direct anchor of a node's own instruction.
    pub fn direct(bci: u32) -> Self {
        Self {
            bci,
            cp: None,
            provenance: Provenance::Direct,
        }
    }

    /// An anchor the node presents but does not own; see [`Provenance::Derived`].
    pub fn derived(bci: u32) -> Self {
        Self {
            bci,
            cp: None,
            provenance: Provenance::Derived,
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
}
