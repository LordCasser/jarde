//! Semantic handoff identity. Optional evidence and candidate retention are not inputs here.
use std::io;

use jarde_reader::{
    budget::{Budget, CountedBudgetDimension},
    model::JvmBytes,
};
use serde::{Deserialize, Serialize, ser::SerializeSeq};

use crate::{
    facts::ClassMembers,
    report::*,
    stop::{self, StopReason},
};

/// Lexical/projection settings and the digest of this run's semantic handoff.
/// The digest is BLAKE3 over the fixed ordered JSON encoding below, without an intermediate
/// document. Changing that encoding changes the artifact binding schema.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactConfiguration {
    pub pool_spelled_members: bool,
    pub projection_types: Vec<JvmBytes>,
    pub allow_array_constructor_method_references: bool,
    pub inputs_digest: String,
}

#[derive(Serialize)]
struct Inputs<'a> {
    facts: &'a crate::facts::RecoveryFacts,
    members: Option<Members<'a>>,
    member_inner_targets: &'a [MemberInnerTargetInput],
    static_member_target: Option<&'a StaticMemberTargetInput>,
    typed_functional_target: Option<FunctionalTargetInput>,
    interface_super_calls: &'a [InterfaceSuperCallInput],
    reference_overload_calls: &'a [ReferenceOverloadCallInput],
    reference_relations: &'a [ReferenceRelationInput],
    declared_types: Option<&'a DeclaredGenericTypes>,
    superclass_field_writes: &'a [SuperclassFieldWriteInput],
    captured_outer_reads: &'a [CapturedOuterReadInput],
    outer_super_calls: &'a [OuterSuperCallInput],
}

/// Member bodies are identified by the physical read, not serialized copies of the decode.
/// The public low-level handoff's consistency with that identity is the separate A1.6 gate.
struct Members<'a>(&'a ClassMembers);
impl Serialize for Members<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.members().len() + 1))?;
        sequence.serialize_element(self.0.raw_owner())?;
        for member in self.0.members() {
            sequence.serialize_element(&(
                member.identity(),
                member.access_flags(),
                member.code().is_some(),
            ))?;
        }
        sequence.end()
    }
}

struct DigestWriter<'a> {
    hash: blake3::Hasher,
    budget: &'a mut Budget,
    stopped: Option<StopReason>,
}
impl io::Write for DigestWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Err(stop) = stop::charge(
            self.budget,
            CountedBudgetDimension::AnalysisSteps,
            (bytes.len() as u64).div_ceil(64),
            None,
        ) {
            self.stopped = Some(stop);
            return Err(io::Error::other("semantic input digest stopped"));
        }
        self.hash.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ArtifactConfiguration {
    pub(crate) fn of(
        request: &RecoveryRequest<'_>,
        allow_array_constructor_method_references: bool,
        budget: &mut Budget,
    ) -> Result<Self, StopReason> {
        if let Some(subject) = &request.subject {
            stop::charge(
                budget,
                CountedBudgetDimension::IrItems,
                subject.dependencies().len() as u64,
                None,
            )?;
        }
        let inputs = Inputs {
            facts: request.facts,
            members: request.members.map(Members),
            member_inner_targets: request.member_inner_targets,
            static_member_target: request.static_member_target,
            typed_functional_target: request.typed_functional_target,
            interface_super_calls: request.interface_super_calls,
            reference_overload_calls: request.reference_overload_calls,
            reference_relations: request.reference_relations,
            declared_types: request.declared_types,
            superclass_field_writes: request.superclass_field_writes,
            captured_outer_reads: request.captured_outer_reads,
            outer_super_calls: request.outer_super_calls,
        };
        let mut writer = DigestWriter {
            hash: blake3::Hasher::new(),
            budget,
            stopped: None,
        };
        writer.hash.update(b"jarde-recovery-inputs-v11\0");
        if serde_json::to_writer(&mut writer, &inputs).is_err() {
            // Only fixed structs, raw byte arrays, strings and integer fields are serialized. The only fallible
            // operation is our budgeted writer; no JSON value can fail independently.
            return Err(writer.stopped.expect("the semantic digest writer stopped"));
        }
        let inputs_digest = format!("blake3:{}", writer.hash.finalize().to_hex());
        let mut projection_types = Vec::new();
        for name in request.projection_types {
            stop::charge(budget, CountedBudgetDimension::IrItems, 1, None)?;
            stop::charge(
                budget,
                CountedBudgetDimension::AnalysisSteps,
                (name.len() as u64).div_ceil(64),
                None,
            )?;
            projection_types.push(JvmBytes(name.to_vec()));
        }
        Ok(Self {
            pool_spelled_members: request.pool_spelled_members,
            projection_types,
            allow_array_constructor_method_references,
            inputs_digest,
        })
    }
}
