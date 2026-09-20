use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SnapshotId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContainerId(pub String);

/// Raw bytes whose JSON representation is an array of octets, never lossy text.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArchiveNameBytes(pub Vec<u8>);

/// JVM Modified UTF-8 bytes, retained as supplied by a classfile.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JvmBytes(pub Vec<u8>);

/// An owned, lossless JVM Modified UTF-8 value.
///
/// Equality and hashing include the original bytes and decoded UTF-16 code units;
/// the escaped display is always derived and cannot be supplied independently.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct JvmString {
    raw: JvmBytes,
    utf16: Vec<u16>,
}

impl JvmString {
    pub(crate) fn from_parts(raw: Vec<u8>, utf16: Vec<u16>) -> Self {
        Self {
            raw: JvmBytes(raw),
            utf16,
        }
    }

    pub fn raw(&self) -> &JvmBytes {
        &self.raw
    }

    pub fn utf16(&self) -> &[u16] {
        &self.utf16
    }

    pub fn escaped(&self) -> String {
        let mut output = String::new();
        for &unit in &self.utf16 {
            match unit {
                0x5c => output.push_str("\\\\"),
                0x22 => output.push_str("\\\""),
                0x20..=0x21 | 0x23..=0x5b | 0x5d..=0x7e => {
                    output.push(char::from_u32(u32::from(unit)).expect("printable ASCII is valid"));
                }
                _ => {
                    use std::fmt::Write as _;
                    write!(output, "\\u{unit:04X}").expect("writing to String cannot fail");
                }
            }
        }
        output
    }
}

impl Serialize for JvmString {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct View<'a> {
            raw: &'a JvmBytes,
            utf16: &'a [u16],
            escaped: String,
        }
        View {
            raw: &self.raw,
            utf16: &self.utf16,
            escaped: self.escaped(),
        }
        .serialize(serializer)
    }
}

fn decode_valid_mutf8(raw: &[u8]) -> Option<Vec<u16>> {
    let mut units = Vec::new();
    let mut offset = 0usize;
    while offset < raw.len() {
        let first = *raw.get(offset)?;
        let (unit, width) = if (1..=0x7f).contains(&first) {
            (u16::from(first), 1)
        } else if first & 0xe0 == 0xc0 {
            let second = *raw.get(offset.checked_add(1)?)?;
            if second & 0xc0 != 0x80 {
                return None;
            }
            let unit = (u16::from(first & 0x1f) << 6) | u16::from(second & 0x3f);
            if unit != 0 && unit < 0x80 {
                return None;
            }
            (unit, 2)
        } else if first & 0xf0 == 0xe0 {
            let second = *raw.get(offset.checked_add(1)?)?;
            let third = *raw.get(offset.checked_add(2)?)?;
            if second & 0xc0 != 0x80 || third & 0xc0 != 0x80 {
                return None;
            }
            let unit = (u16::from(first & 0x0f) << 12)
                | (u16::from(second & 0x3f) << 6)
                | u16::from(third & 0x3f);
            if unit < 0x800 {
                return None;
            }
            (unit, 3)
        } else {
            return None;
        };
        units.push(unit);
        offset = offset.checked_add(width)?;
    }
    Some(units)
}

impl<'de> Deserialize<'de> for JvmString {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct View {
            raw: JvmBytes,
            utf16: Vec<u16>,
            escaped: String,
        }
        let view = View::deserialize(deserializer)?;
        let decoded = decode_valid_mutf8(&view.raw.0)
            .ok_or_else(|| D::Error::custom("raw JVM string is not valid Modified UTF-8"))?;
        if decoded != view.utf16 {
            return Err(D::Error::custom(
                "UTF-16 JVM string value is inconsistent with raw bytes",
            ));
        }
        let value = Self::from_parts(view.raw.0, decoded);
        if view.escaped != value.escaped() {
            return Err(D::Error::custom(
                "escaped JVM string display is inconsistent",
            ));
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Digest(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ByteSpan {
    pub start: u64,
    pub length: u64,
}

impl ByteSpan {
    pub const fn new(start: u64, length: u64) -> Self {
        Self { start, length }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerOriginStep {
    pub via_ordinal: u64,
    pub via_raw_name: ArchiveNameBytes,
    pub child_container: ContainerId,
}

/// One container's physical identity: the immutable snapshot plus the chain of nested entries
/// that reaches it.
///
/// The order is the derivation order — snapshot, root container, then each step — so a
/// `BTreeMap` over origins is a stable, total order that no hash seed can move. The container
/// facts cache keys on this type and nothing about the request (`RuntimeProfile`, loader order,
/// prefix) is in it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerOrigin {
    pub snapshot: SnapshotId,
    pub root_container: ContainerId,
    pub steps: Vec<ContainerOriginStep>,
}

impl ContainerOrigin {
    pub fn current_container(&self) -> &ContainerId {
        self.steps
            .last()
            .map_or(&self.root_container, |step| &step.child_container)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalEntryId {
    pub origin: ContainerOrigin,
    pub ordinal: u64,
    pub raw_name: ArchiveNameBytes,
}

impl PhysicalEntryId {
    pub fn snapshot(&self) -> &SnapshotId {
        &self.origin.snapshot
    }

    pub fn container(&self) -> &ContainerId {
        self.origin.current_container()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ClassBytesId {
    pub digest: Digest,
    pub length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PhysicalVariant {
    Base,
    MultiRelease { version: u16 },
    Other { label: String },
}

/// Syntactic physical-variant label of a container-relative raw path.
///
/// This is the one derivation of [`PhysicalVariant`] from an entry name, shared by the
/// physical scan and the P2 header lookup so one entry has one identity in both reports.
/// The label is syntactic: it is not the multi-release selection contract and claims
/// nothing about activation or validity.
pub fn physical_variant_for_path(raw_name: &[u8]) -> PhysicalVariant {
    const PREFIX: &[u8] = b"META-INF/versions/";
    let Some(rest) = raw_name.strip_prefix(PREFIX) else {
        return PhysicalVariant::Base;
    };
    let Some(slash) = rest.iter().position(|byte| *byte == b'/') else {
        return PhysicalVariant::Base;
    };
    let (release, logical) = rest.split_at(slash);
    let release_unlabelled = PhysicalVariant::Other {
        label: "multi_release_version_unlabelled".into(),
    };
    if release.is_empty() || logical.len() <= 1 || (release.len() > 1 && release[0] == b'0') {
        return release_unlabelled;
    }
    if !release.iter().all(u8::is_ascii_digit) {
        return release_unlabelled;
    }
    let mut version = 0_u64;
    for digit in release {
        version = match version
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(digit - b'0')))
        {
            Some(value) => value,
            None => return release_unlabelled,
        };
    }
    match u16::try_from(version) {
        Ok(version) => PhysicalVariant::MultiRelease { version },
        Err(_) => release_unlabelled,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PhysicalClassLocation {
    StandaloneRoot { snapshot: SnapshotId },
    ArchiveEntry { entry: PhysicalEntryId },
}

impl PhysicalClassLocation {
    pub fn snapshot(&self) -> &SnapshotId {
        match self {
            Self::StandaloneRoot { snapshot } => snapshot,
            Self::ArchiveEntry { entry } => entry.snapshot(),
        }
    }

    pub fn entry(&self) -> Option<&PhysicalEntryId> {
        match self {
            Self::StandaloneRoot { .. } => None,
            Self::ArchiveEntry { entry } => Some(entry),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalDefinitionId {
    pub location: PhysicalClassLocation,
    pub class_bytes: ClassBytesId,
    pub variant: PhysicalVariant,
}

impl PhysicalDefinitionId {
    pub fn snapshot(&self) -> &SnapshotId {
        self.location.snapshot()
    }

    pub fn entry(&self) -> Option<&PhysicalEntryId> {
        self.location.entry()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemberKey {
    Field {
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    Method {
        name: JvmBytes,
        descriptor: JvmBytes,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PhysicalMemberId {
    pub owner: PhysicalDefinitionId,
    pub member: MemberKey,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalMethodId {
    pub owner: PhysicalDefinitionId,
    pub name: JvmBytes,
    pub descriptor: JvmBytes,
}

/// Physical anchors of one generated IR artifact, in generation order.
///
/// Members are kept in the order the artifacts were produced and are deduplicated by
/// equality. Every member is a physical coordinate: a class file identity, a range in
/// class-file bytes, or a method point. A generated node that comes from several
/// original BCIs keeps one member per original location instead of collapsing them.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OriginSet {
    pub members: Vec<OriginMember>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OriginMember {
    /// A whole class file, without a narrower range.
    ClassFile { definition: PhysicalDefinitionId },
    /// A byte range in a class file. `span` is always a class-file coordinate, never a
    /// container- or entry-relative one.
    ClassRange {
        definition: PhysicalDefinitionId,
        span: ByteSpan,
    },
    /// A bytecode index inside a method body.
    ///
    /// Synonymous with [`Location::Code`] but a separate type, so no IR field can pass an
    /// entry-relative [`Location::Entry`] span where a code coordinate is required.
    MethodPoint { method: PhysicalMethodId, bci: u32 },
}

impl OriginSet {
    /// Records one member, keeping first-appearance order and dropping exact repeats.
    pub fn insert(&mut self, member: OriginMember) {
        if !self.members.contains(&member) {
            self.members.push(member);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SymbolRef {
    Class {
        owner: JvmBytes,
    },
    Field {
        owner: JvmBytes,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    Method {
        owner: JvmBytes,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Location {
    Container {
        snapshot: SnapshotId,
        id: ContainerId,
        span: ByteSpan,
    },
    Entry {
        id: PhysicalEntryId,
        span: ByteSpan,
    },
    ClassOffset {
        definition: PhysicalDefinitionId,
        offset: u64,
    },
    Code {
        method: PhysicalMethodId,
        bci: u32,
    },
    Attribute {
        owner: PhysicalDefinitionId,
        path: String,
        span: ByteSpan,
    },
    Resource {
        entry: PhysicalEntryId,
        span: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Provenance {
    pub location: Location,
}

impl Location {
    pub fn snapshot(&self) -> &SnapshotId {
        match self {
            Self::Container { snapshot, .. } => snapshot,
            Self::Entry { id, .. } | Self::Resource { entry: id, .. } => id.snapshot(),
            Self::ClassOffset { definition, .. }
            | Self::Attribute {
                owner: definition, ..
            } => definition.snapshot(),
            Self::Code { method, .. } => method.owner.snapshot(),
        }
    }
}

impl Provenance {
    pub fn snapshot(&self) -> &SnapshotId {
        self.location.snapshot()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoverageRange {
    pub label: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageState {
    NotRequested,
    CompleteWithinSchema,
    Partial,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoverageDimension {
    pub state: CoverageState,
    pub scanned: Vec<CoverageRange>,
    pub skipped: Vec<CoverageRange>,
    pub uninterpreted_extensions: Vec<String>,
}

impl CoverageDimension {
    pub fn not_requested() -> Self {
        Self {
            state: CoverageState::NotRequested,
            scanned: Vec::new(),
            skipped: Vec::new(),
            uninterpreted_extensions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Coverage {
    pub artifact_structural: CoverageDimension,
    pub runtime_resolution: CoverageDimension,
    pub dynamic_analysis: CoverageDimension,
}

impl Coverage {
    /// Coverage of a request that performed no pass at all: every dimension is
    /// `NotRequested`, so nothing can be read as a completed or partial range.
    pub fn not_requested() -> Self {
        Self {
            artifact_structural: CoverageDimension::not_requested(),
            runtime_resolution: CoverageDimension::not_requested(),
            dynamic_analysis: CoverageDimension::not_requested(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TerminationReason {
    BudgetExceeded {
        dimension: crate::budget::BudgetDimension,
    },
    Error {
        code: String,
    },
    Unsupported {
        code: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ExecutionReport {
    Complete {
        usage: crate::budget::UsageSnapshot,
    },
    Partial {
        reason: TerminationReason,
        usage: crate::budget::UsageSnapshot,
    },
    Cancelled {
        usage: crate::budget::UsageSnapshot,
    },
    Failed {
        reason: TerminationReason,
        usage: crate::budget::UsageSnapshot,
    },
}

/// Restates one execution report under the usage of the request it ends.
///
/// A stop is decided at one point of a request and published at its end, so the counts a
/// caller reads are the ones the whole request consumed. Every layer that merges its own stop
/// into an inner report maps it through this one function: the artifact, multi-release, query
/// and JVM paths of a request must not disagree about what a usage figure means.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub provenance: Option<Provenance>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(ordinal: u64, name: &str) -> PhysicalEntryId {
        PhysicalEntryId {
            origin: ContainerOrigin {
                snapshot: SnapshotId("snap-1".into()),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            ordinal,
            raw_name: ArchiveNameBytes(name.as_bytes().to_vec()),
        }
    }

    #[test]
    fn jvm_string_deserialization_enforces_modified_utf8_nul_and_overlong_rules() {
        let encoded_nul = serde_json::json!({
            "raw": [0xc0, 0x80],
            "utf16": [0],
            "escaped": "\\u0000"
        });
        let value: JvmString = serde_json::from_value(encoded_nul).unwrap();
        assert_eq!(value.raw().0, [0xc0, 0x80]);
        assert_eq!(value.utf16(), [0]);

        let literal_nul = serde_json::json!({
            "raw": [0],
            "utf16": [0],
            "escaped": "\\u0000"
        });
        assert!(serde_json::from_value::<JvmString>(literal_nul).is_err());

        let non_nul_overlong = serde_json::json!({
            "raw": [0xc1, 0x81],
            "utf16": [65],
            "escaped": "A"
        });
        assert!(serde_json::from_value::<JvmString>(non_nul_overlong).is_err());
    }

    #[test]
    fn identical_bytes_keep_distinct_physical_origins() {
        let bytes = ClassBytesId {
            digest: Digest("same".into()),
            length: 4,
        };
        let left = PhysicalDefinitionId {
            location: PhysicalClassLocation::ArchiveEntry {
                entry: entry(1, "a/A.class"),
            },
            class_bytes: bytes.clone(),
            variant: PhysicalVariant::Base,
        };
        let right = PhysicalDefinitionId {
            location: PhysicalClassLocation::ArchiveEntry {
                entry: entry(2, "b/A.class"),
            },
            class_bytes: bytes,
            variant: PhysicalVariant::Base,
        };
        assert_ne!(left, right);
        assert_eq!(left.class_bytes, right.class_bytes);
    }

    #[test]
    fn representative_contract_round_trips_as_readable_json() {
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::ArchiveEntry {
                entry: entry(7, "A.class"),
            },
            class_bytes: ClassBytesId {
                digest: Digest("abc".into()),
                length: 12,
            },
            variant: PhysicalVariant::Base,
        };
        let report = (
            Provenance {
                location: Location::Code {
                    method: PhysicalMethodId {
                        owner: definition,
                        name: JvmBytes(b"run".to_vec()),
                        descriptor: JvmBytes(b"(I)Ljava/lang/String;".to_vec()),
                    },
                    bci: 4,
                },
            },
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: crate::budget::BudgetDimension::CodeBytes,
                },
                usage: crate::budget::UsageSnapshot::default(),
            },
        );
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"kind\":\"code\""));
        assert!(json.contains("\"status\":\"partial\""));
        assert_eq!(
            serde_json::from_str::<(Provenance, ExecutionReport)>(&json).unwrap(),
            report
        );
    }

    #[test]
    fn raw_archive_and_jvm_bytes_round_trip_without_text_loss() {
        let entry = PhysicalEntryId {
            origin: ContainerOrigin {
                snapshot: SnapshotId("snap-raw".into()),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            ordinal: 1,
            raw_name: ArchiveNameBytes(vec![0xff, 0x00, 0x80]),
        };
        let symbol = SymbolRef::Method {
            owner: JvmBytes(vec![b'A', 0, 0xff]),
            name: JvmBytes(vec![0xed, 0xa0, 0x80]),
            descriptor: JvmBytes(vec![b'(', 0xff, b')', b'V']),
        };
        let json = serde_json::to_string(&(entry.clone(), symbol.clone())).unwrap();
        let round_trip: (PhysicalEntryId, SymbolRef) = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, (entry, symbol));
    }

    #[test]
    fn provenance_derives_one_snapshot_from_location() {
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::ArchiveEntry {
                entry: entry(2, "A.class"),
            },
            class_bytes: ClassBytesId {
                digest: Digest("d".into()),
                length: 1,
            },
            variant: PhysicalVariant::Base,
        };
        let provenance = Provenance {
            location: Location::ClassOffset {
                definition,
                offset: 0,
            },
        };
        assert_eq!(provenance.snapshot(), &SnapshotId("snap-1".into()));
        assert_eq!(provenance.location.snapshot(), provenance.snapshot());
    }

    #[test]
    fn physical_code_location_uses_method_identity_only() {
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::ArchiveEntry {
                entry: entry(3, "A.class"),
            },
            class_bytes: ClassBytesId {
                digest: Digest("d".into()),
                length: 1,
            },
            variant: PhysicalVariant::Base,
        };
        let location = Location::Code {
            method: PhysicalMethodId {
                owner: definition.clone(),
                name: JvmBytes(b"run".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            },
            bci: 0,
        };
        assert_eq!(location.snapshot(), definition.snapshot());
        let json = serde_json::to_string(&location).unwrap();
        assert!(json.contains("\"kind\":\"code\""));
        assert!(json.contains("\"descriptor\":[40,41,86]"));
    }

    #[test]
    fn field_identity_cannot_deserialize_as_code_method() {
        let json = serde_json::json!({
            "kind": "code",
            "method": {
                "owner": {
                    "location": {
                        "kind": "archive_entry",
                        "entry": {
                            "origin": {
                                "snapshot": "snap-1",
                                "root_container": "root",
                                "steps": []
                            },
                            "ordinal": 3,
                            "raw_name": [65, 46, 99, 108, 97, 115, 115]
                        }
                    },
                    "class_bytes": {"digest": "d", "length": 1},
                    "variant": {"kind": "base"}
                },
                "kind": "field",
                "name": [118, 97, 108, 117, 101],
                "descriptor": [73]
            },
            "bci": 0
        });
        assert!(serde_json::from_value::<Location>(json).is_err());
    }

    #[test]
    fn execution_status_tags_have_only_valid_variant_fields() {
        let complete = ExecutionReport::Complete {
            usage: crate::budget::UsageSnapshot::default(),
        };
        let json = serde_json::to_string(&complete).unwrap();
        assert!(json.contains("\"status\":\"complete\""));
        assert!(!json.contains("reason"));
        assert_eq!(
            serde_json::from_str::<ExecutionReport>(&json).unwrap(),
            complete
        );
    }
}
