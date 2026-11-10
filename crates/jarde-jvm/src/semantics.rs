//! Resolution/analysis semantics are independent of wire schemas and dialect capability limits.
use serde::{Deserialize, Serialize};

/// Change this revision when the resolution/analysis contract changes. Implementations also
/// carry their actual source identities, including dirty same-version builds.
pub const RESOLUTION_SEMANTICS: u16 = 6;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionSemantics {
    pub contract: u16,
    pub reader: String,
    pub query: String,
    pub jvm: String,
}

impl ResolutionSemantics {
    pub fn current() -> Self {
        Self {
            contract: RESOLUTION_SEMANTICS,
            reader: jarde_reader::IMPLEMENTATION_ID.into(),
            query: jarde_query::IMPLEMENTATION_ID.into(),
            jvm: crate::IMPLEMENTATION_ID.into(),
        }
    }
}
