//! Request identities for physical and runtime artifact views.

use crate::model::{ArchiveNameBytes, ContainerId, ContainerOrigin, SnapshotId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PhysicalScope {
    SnapshotAll,
    ArtifactTree { root_container: ContainerId },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalView {
    pub snapshot: SnapshotId,
    pub scope: PhysicalScope,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MultiReleasePolicy {
    Disabled,
    Enabled,
    Custom { id: String },
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LayoutMode {
    Generic,
    War,
    SpringBoot,
    Custom { id: String },
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfile {
    pub java_release: u16,
    pub multi_release: MultiReleasePolicy,
    pub layout: LayoutMode,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoaderId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DelegationPolicy {
    ParentFirst,
    ChildFirst,
    Custom { id: String },
    Unknown,
}

/// One declared load position: the physical content a loader searches and where inside it a
/// class name is looked up.
///
/// The three variants are the three physical shapes a declaration can name, and nothing above
/// them infers which one a caller meant:
///
/// * [`LoadRoot::StandaloneClass`] is one whole CLASS file: the snapshot *is* the definition,
///   and its own `this_class` is the only name it can provide. It is never inferred from a file
///   name.
/// * [`LoadRoot::Container`] is one container of a ZIP snapshot — the snapshot's root container,
///   or one reached along the origin chain of nested entries — together with the raw byte prefix
///   a name is looked up under. The prefix is an archive-internal byte prefix, not a host path: it
///   is empty (the container's own root) or ends with `/`, and a non-empty prefix that does not
///   end with that boundary is an invalid declaration (the runtime environment validator reports
///   it as `invalid_root_prefix`). Lookup composes `prefix + internal name + ".class"` byte for
///   byte — no trimming, no URL decoding, no case folding and no `.`/`..`/backslash folding — and
///   never requires a directory entry to exist for the prefix, because a ZIP's directory entries
///   are not what makes an entry reachable.
/// * [`LoadRoot::External`] names content the entry has not provided: it is a declaration, not a
///   readable position, and it resolves nothing.
///
/// The prefix is part of the runtime environment's identity: two roots that differ only in
/// prefix are two declarations, and a lookup that changed its environment may not reuse the
/// verdict of the old one.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LoadRoot {
    StandaloneClass {
        snapshot: SnapshotId,
    },
    Container {
        origin: ContainerOrigin,
        prefix: ArchiveNameBytes,
    },
    External {
        id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModuleMode {
    ClassPath,
    ModulePath,
    Hybrid,
    Custom { id: String },
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeUncertainty {
    None,
    Possible,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoadDomain {
    pub loader: LoaderId,
    pub parent_loader: Option<LoaderId>,
    pub delegation: DelegationPolicy,
    pub roots: Vec<LoadRoot>,
    pub module_mode: ModuleMode,
    pub external_override: RuntimeUncertainty,
    pub runtime_transformation: RuntimeUncertainty,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeView {
    pub physical: PhysicalView,
    pub profile: RuntimeProfile,
    pub load_domain: LoadDomain,
}
