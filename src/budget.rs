use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CountedBudgetDimension {
    InputBytes,
    /// Central-directory records processed by this request, including locator scans.
    ArchiveEntries,
    EntryBytes,
    ReadBytes,
    ClassBytes,
    AttributeBytes,
    CodeBytes,
    ResultItems,
    OutputBytes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetDimension {
    InputBytes,
    ArchiveEntries,
    EntryBytes,
    ReadBytes,
    ClassBytes,
    AttributeBytes,
    CodeBytes,
    ResultItems,
    OutputBytes,
    NestedDepth,
    ElapsedMillis,
}

impl From<CountedBudgetDimension> for BudgetDimension {
    fn from(value: CountedBudgetDimension) -> Self {
        match value {
            CountedBudgetDimension::InputBytes => Self::InputBytes,
            CountedBudgetDimension::ArchiveEntries => Self::ArchiveEntries,
            CountedBudgetDimension::EntryBytes => Self::EntryBytes,
            CountedBudgetDimension::ReadBytes => Self::ReadBytes,
            CountedBudgetDimension::ClassBytes => Self::ClassBytes,
            CountedBudgetDimension::AttributeBytes => Self::AttributeBytes,
            CountedBudgetDimension::CodeBytes => Self::CodeBytes,
            CountedBudgetDimension::ResultItems => Self::ResultItems,
            CountedBudgetDimension::OutputBytes => Self::OutputBytes,
        }
    }
}

impl TryFrom<BudgetDimension> for CountedBudgetDimension {
    type Error = ();

    fn try_from(value: BudgetDimension) -> std::result::Result<Self, Self::Error> {
        match value {
            BudgetDimension::InputBytes => Ok(Self::InputBytes),
            BudgetDimension::ArchiveEntries => Ok(Self::ArchiveEntries),
            BudgetDimension::EntryBytes => Ok(Self::EntryBytes),
            BudgetDimension::ReadBytes => Ok(Self::ReadBytes),
            BudgetDimension::ClassBytes => Ok(Self::ClassBytes),
            BudgetDimension::AttributeBytes => Ok(Self::AttributeBytes),
            BudgetDimension::CodeBytes => Ok(Self::CodeBytes),
            BudgetDimension::ResultItems => Ok(Self::ResultItems),
            BudgetDimension::OutputBytes => Ok(Self::OutputBytes),
            BudgetDimension::NestedDepth | BudgetDimension::ElapsedMillis => Err(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Limits {
    pub input_bytes: u64,
    pub archive_entries: u64,
    pub entry_bytes: u64,
    pub read_bytes: u64,
    pub class_bytes: u64,
    pub attribute_bytes: u64,
    pub code_bytes: u64,
    pub result_items: u64,
    pub output_bytes: u64,
    pub nested_depth: u64,
    pub elapsed_millis: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub input_bytes: u64,
    pub archive_entries: u64,
    pub entry_bytes: u64,
    pub read_bytes: u64,
    pub class_bytes: u64,
    pub attribute_bytes: u64,
    pub code_bytes: u64,
    pub result_items: u64,
    pub output_bytes: u64,
    pub nested_depth: u64,
    pub elapsed_millis: u64,
}

impl Limits {
    fn get(&self, dimension: CountedBudgetDimension) -> u64 {
        match dimension {
            CountedBudgetDimension::InputBytes => self.input_bytes,
            CountedBudgetDimension::ArchiveEntries => self.archive_entries,
            CountedBudgetDimension::EntryBytes => self.entry_bytes,
            CountedBudgetDimension::ReadBytes => self.read_bytes,
            CountedBudgetDimension::ClassBytes => self.class_bytes,
            CountedBudgetDimension::AttributeBytes => self.attribute_bytes,
            CountedBudgetDimension::CodeBytes => self.code_bytes,
            CountedBudgetDimension::ResultItems => self.result_items,
            CountedBudgetDimension::OutputBytes => self.output_bytes,
        }
    }
}

impl UsageSnapshot {
    fn get(&self, dimension: CountedBudgetDimension) -> u64 {
        match dimension {
            CountedBudgetDimension::InputBytes => self.input_bytes,
            CountedBudgetDimension::ArchiveEntries => self.archive_entries,
            CountedBudgetDimension::EntryBytes => self.entry_bytes,
            CountedBudgetDimension::ReadBytes => self.read_bytes,
            CountedBudgetDimension::ClassBytes => self.class_bytes,
            CountedBudgetDimension::AttributeBytes => self.attribute_bytes,
            CountedBudgetDimension::CodeBytes => self.code_bytes,
            CountedBudgetDimension::ResultItems => self.result_items,
            CountedBudgetDimension::OutputBytes => self.output_bytes,
        }
    }

    fn add(&mut self, dimension: CountedBudgetDimension, amount: u64) -> Option<()> {
        let slot = match dimension {
            CountedBudgetDimension::InputBytes => &mut self.input_bytes,
            CountedBudgetDimension::ArchiveEntries => &mut self.archive_entries,
            CountedBudgetDimension::EntryBytes => &mut self.entry_bytes,
            CountedBudgetDimension::ReadBytes => &mut self.read_bytes,
            CountedBudgetDimension::ClassBytes => &mut self.class_bytes,
            CountedBudgetDimension::AttributeBytes => &mut self.attribute_bytes,
            CountedBudgetDimension::CodeBytes => &mut self.code_bytes,
            CountedBudgetDimension::ResultItems => &mut self.result_items,
            CountedBudgetDimension::OutputBytes => &mut self.output_bytes,
        };
        *slot = slot.checked_add(amount)?;
        Some(())
    }
}

#[derive(Clone, Debug)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Budget {
    limits: Limits,
    usage: UsageSnapshot,
    cancellation: CancellationToken,
    started_at: Instant,
}

impl Budget {
    pub fn new(limits: Limits) -> Self {
        Self::with_cancellation_token(limits, CancellationToken::new())
    }

    pub fn with_cancellation_token(limits: Limits, cancellation: CancellationToken) -> Self {
        Self {
            limits,
            usage: UsageSnapshot::default(),
            cancellation,
            started_at: Instant::now(),
        }
    }

    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    pub fn usage(&self) -> UsageSnapshot {
        let mut usage = self.usage.clone();
        usage.elapsed_millis = self.elapsed_millis();
        usage
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    pub fn poll(&self) -> Result<()> {
        self.check_cancelled()?;
        self.check_elapsed()
    }

    pub fn check(&self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        self.poll()?;
        self.ensure_within(dimension, requested)
    }

    pub fn charge(&mut self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        self.poll()?;
        self.ensure_within(dimension, requested)?;
        if self.usage.add(dimension, requested).is_none() {
            return Err(self.exceeded(dimension, requested));
        }
        self.usage.elapsed_millis = self.elapsed_millis();
        Ok(())
    }

    pub fn check_nested_depth(&mut self, depth: u64) -> Result<()> {
        self.poll()?;
        if depth > self.limits.nested_depth {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth,
                limit: self.limits.nested_depth,
                consumed: depth.saturating_sub(1),
                requested: 1,
            });
        }
        self.usage.nested_depth = self.usage.nested_depth.max(depth);
        self.usage.elapsed_millis = self.elapsed_millis();
        Ok(())
    }

    fn elapsed_millis(&self) -> u64 {
        self.started_at.elapsed().as_millis().min(u64::MAX as u128) as u64
    }

    fn check_cancelled(&self) -> Result<()> {
        if self.cancellation.is_cancelled() {
            Err(Error::Cancelled {
                reason: "cooperative cancellation requested".into(),
            })
        } else {
            Ok(())
        }
    }

    fn check_elapsed(&self) -> Result<()> {
        let elapsed = self.elapsed_millis();
        if elapsed >= self.limits.elapsed_millis {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                limit: self.limits.elapsed_millis,
                consumed: elapsed,
                requested: 0,
            });
        }
        Ok(())
    }

    fn ensure_within(&self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        let consumed = self.usage.get(dimension);
        let limit = self.limits.get(dimension);
        match consumed.checked_add(requested) {
            Some(total) if total <= limit => Ok(()),
            _ => Err(self.exceeded(dimension, requested)),
        }
    }

    fn exceeded(&self, dimension: CountedBudgetDimension, requested: u64) -> Error {
        Error::BudgetExceeded {
            dimension: dimension.into(),
            limit: self.limits.get(dimension),
            consumed: self.usage.get(dimension),
            requested,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(value: u64) -> Limits {
        Limits {
            input_bytes: value,
            archive_entries: value,
            entry_bytes: value,
            read_bytes: value,
            class_bytes: value,
            attribute_bytes: value,
            code_bytes: value,
            result_items: value,
            output_bytes: value,
            nested_depth: value,
            elapsed_millis: u64::MAX,
        }
    }

    #[test]
    fn exact_limit_succeeds_and_next_charge_reports_counts() {
        let mut budget = Budget::new(limits(3));
        budget.charge(CountedBudgetDimension::CodeBytes, 3).unwrap();
        let error = budget
            .charge(CountedBudgetDimension::CodeBytes, 1)
            .unwrap_err();
        assert_eq!(
            error,
            Error::BudgetExceeded {
                dimension: BudgetDimension::CodeBytes,
                limit: 3,
                consumed: 3,
                requested: 1
            }
        );
        assert_eq!(budget.usage().code_bytes, 3);
    }

    #[test]
    fn overflow_is_rejected_without_mutating_usage() {
        let mut budget = Budget::new(limits(u64::MAX));
        budget
            .charge(CountedBudgetDimension::InputBytes, u64::MAX - 1)
            .unwrap();
        let before = budget.usage().input_bytes;
        assert!(
            budget
                .charge(CountedBudgetDimension::InputBytes, 2)
                .is_err()
        );
        assert_eq!(budget.usage().input_bytes, before);
    }

    #[test]
    fn cancellation_applies_before_and_after_a_charge() {
        let mut budget = Budget::new(limits(4));
        let token = budget.cancellation_token();
        token.cancel();
        assert!(matches!(
            budget.check(CountedBudgetDimension::CodeBytes, 1),
            Err(Error::Cancelled { .. })
        ));
        assert!(matches!(
            budget.charge(CountedBudgetDimension::CodeBytes, 1),
            Err(Error::Cancelled { .. })
        ));
        assert_eq!(budget.usage().code_bytes, 0);
    }

    #[test]
    fn usage_snapshot_is_json_serializable() {
        let mut budget = Budget::new(limits(5));
        budget
            .charge(CountedBudgetDimension::ResultItems, 2)
            .unwrap();
        let json = serde_json::to_string(&budget.usage()).unwrap();
        let round_trip: UsageSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, budget.usage());
    }

    #[test]
    fn successful_charge_is_retained_when_cancelled_afterward() {
        let mut budget = Budget::new(limits(5));
        let token = budget.cancellation_token();
        budget.charge(CountedBudgetDimension::ReadBytes, 2).unwrap();
        token.cancel();
        assert!(matches!(budget.poll(), Err(Error::Cancelled { .. })));
        assert_eq!(budget.usage().read_bytes, 2);
    }

    #[test]
    fn elapsed_limit_is_cooperative_and_reported_in_usage() {
        let mut configured = limits(10);
        configured.elapsed_millis = 1;
        let budget = Budget::new(configured);
        std::thread::sleep(std::time::Duration::from_millis(3));
        assert!(matches!(
            budget.poll(),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                ..
            })
        ));
        assert!(budget.usage().elapsed_millis >= 1);
    }

    #[test]
    fn nested_depth_is_a_non_counted_high_water_mark() {
        let mut configured = limits(10);
        configured.nested_depth = 1;
        let mut budget = Budget::new(configured);
        budget.check_nested_depth(0).unwrap();
        budget.check_nested_depth(1).unwrap();
        assert_eq!(budget.usage().nested_depth, 1);
        assert_eq!(
            budget.check_nested_depth(2).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth,
                limit: 1,
                consumed: 1,
                requested: 1,
            }
        );
        assert_eq!(budget.usage().nested_depth, 1);
        assert!(CountedBudgetDimension::try_from(BudgetDimension::NestedDepth).is_err());
    }

    #[test]
    fn elapsed_is_not_a_chargeable_dimension() {
        assert!(CountedBudgetDimension::try_from(BudgetDimension::ElapsedMillis).is_err());
        assert_eq!(
            BudgetDimension::from(CountedBudgetDimension::ReadBytes),
            BudgetDimension::ReadBytes
        );
    }
}
