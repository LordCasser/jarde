//! Request identities for physical and runtime artifact views.

use crate::model::{ContainerId, ContainerOrigin, SnapshotId};
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

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LoadRoot {
    Snapshot { snapshot: SnapshotId },
    ArtifactTree { root: ContainerOrigin },
    External { id: String },
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
