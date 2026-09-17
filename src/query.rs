//! Stable query relation and consumer-schema value types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryRelation {
    ConstantPoolContains,
    MentionsSymbol,
    LiteralValue,
    ReferencesDefinition,
    MayDispatchTo,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsumerKind {
    Invocation,
    Field,
    Type,
    Constant,
    Exception,
    Signature,
    Annotation,
    InnerNest,
    Module,
    Bootstrap,
    Resource,
    Verification,
    Debug,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumerSchema {
    pub version: u16,
    pub kinds: BTreeSet<ConsumerKind>,
}

impl ConsumerSchema {
    pub fn new(version: u16, kinds: impl IntoIterator<Item = ConsumerKind>) -> Self {
        Self {
            version,
            kinds: kinds.into_iter().collect(),
        }
    }
}
