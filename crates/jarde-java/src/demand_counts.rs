//! D1's construction-site port: how many **owning evidence records** this layer really built.
//!
//! # Why a port, and why it sits at the construction site
//!
//! The property change `add-demand-driven-core-results` states for D1 is "an unrequested category is
//! not built", and it is a property of a *run* rather than of any result the run publishes: a record
//! that was constructed and dropped before publication leaves no trace in the report. So the count
//! has to be taken where the record is built — inside the rule modules and the report — and not
//! where a report is read back. That is the difference this port exists to make checkable: an
//! implementation that builds the whole table and then hides it from the payload says zero in the
//! status list and says nothing here.
//!
//! # Why this crate's arm is `test` alone
//!
//! The root crate's own port (`src/d0_counts.rs`) is gated on `any(test, feature = "test-support")`,
//! because that crate declares the feature. This one has no such feature in its manifest, and this
//! round adds none: a feature would be a manifest change, and a manifest change made to observe a
//! counter is exactly the kind of coupling the port is meant to avoid. So the counters exist for
//! this crate's own tests (its own unit and module tests, which build real fixtures through the
//! reader's class builder), and every build that ships has no counter at all: the hooks below stay
//! one-line calls into an empty function, nothing is retained, and no counter can be read.
//!
//! The port is bounded by construction whatever a run does: five `u64`s, one per category this layer
//! materializes, and no record of the work they counted. It is not part of [`crate::RecoveryReport`],
//! of any other report, of a stop record or of a fingerprint.

use crate::evidence::RecoveryEvidenceKind;

/// The counter indices, in the order the categories are stated.
///
/// They exist only where the counters do: a hook call site names the *category*, so the same line
/// compiles with and without the port.
#[cfg(test)]
const SOURCE_MAP: usize = 0;
#[cfg(test)]
const REGION_DETAILS: usize = 1;
#[cfg(test)]
const RULE_DETAILS: usize = 2;
#[cfg(test)]
const NAME_DETAILS: usize = 3;

/// How many counters this port keeps.
#[cfg(test)]
const COUNTERS: usize = 4;

// Every counter of the thread reading it, in the order the indices above name them.
#[cfg(test)]
thread_local! {
    static COUNTS: std::cell::Cell<[u64; COUNTERS]> = const { std::cell::Cell::new([0; COUNTERS]) };
}

/// The index of one category's counter, when this layer materializes it at all.
#[cfg(test)]
fn index(kind: RecoveryEvidenceKind) -> Option<usize> {
    match kind {
        RecoveryEvidenceKind::SourceMap => Some(SOURCE_MAP),
        RecoveryEvidenceKind::RegionDetails => Some(REGION_DETAILS),
        RecoveryEvidenceKind::RuleDetails => Some(RULE_DETAILS),
        RecoveryEvidenceKind::NameDetails => Some(NAME_DETAILS),
        RecoveryEvidenceKind::ReadDetails => None,
    }
}

/// One owning record of one category was constructed.
#[cfg(test)]
#[inline]
pub(crate) fn record_built(kind: RecoveryEvidenceKind) {
    let Some(counter) = index(kind) else {
        return;
    };
    COUNTS.with(|counts| {
        let mut read = counts.get();
        read[counter] = read[counter].saturating_add(1);
        counts.set(read);
    });
}

/// The same, in a build without the port: nothing is counted and nothing is kept.
#[cfg(not(test))]
#[inline]
pub(crate) fn record_built(_kind: RecoveryEvidenceKind) {}

/// How many owning records of each category the tests that call this read have built so far.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Built {
    pub(crate) source_map: u64,
    pub(crate) region_details: u64,
    pub(crate) rule_details: u64,
    pub(crate) name_details: u64,
}

#[cfg(test)]
impl Built {
    /// The work counted between `self` and `later`: one field per category, `later` minus `self`.
    pub(crate) fn since(self, later: Built) -> Built {
        Built {
            source_map: later.source_map - self.source_map,
            region_details: later.region_details - self.region_details,
            rule_details: later.rule_details - self.rule_details,
            name_details: later.name_details - self.name_details,
        }
    }

    /// How many records of one category were built.
    pub(crate) fn of(&self, kind: RecoveryEvidenceKind) -> u64 {
        match kind {
            RecoveryEvidenceKind::SourceMap => self.source_map,
            RecoveryEvidenceKind::RegionDetails => self.region_details,
            RecoveryEvidenceKind::RuleDetails => self.rule_details,
            RecoveryEvidenceKind::NameDetails => self.name_details,
            RecoveryEvidenceKind::ReadDetails => 0,
        }
    }
}

/// Every counter, as one reading.
#[cfg(test)]
pub(crate) fn snapshot() -> Built {
    COUNTS.with(|counts| {
        let read = counts.get();
        Built {
            source_map: read[SOURCE_MAP],
            region_details: read[REGION_DETAILS],
            rule_details: read[RULE_DETAILS],
            name_details: read[NAME_DETAILS],
        }
    })
}
