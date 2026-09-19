//! Owned, bounded header inspection backed exclusively by noak 0.7.0.

use crate::budget::{Budget, CountedBudgetDimension};
use crate::error::{Error, Result};
use crate::model::{
    ByteSpan, Coverage, CoverageDimension, CoverageRange, CoverageState, Diagnostic,
    DiagnosticSeverity, ExecutionReport, JvmBytes, JvmString, TerminationReason,
};
use crate::release_registry::{
    HIGHEST_REGISTERED_MAJOR, MINIMUM_MAJOR, PREVIEW_MARKER, ReleaseLookup, ReleaseRegistration,
    feature_registry,
};
use noak::reader::attributes::{ArrayType, Code, RawInstruction};
use noak::reader::{Attribute, Class};
use serde::{Deserialize, Serialize};

const ATTRIBUTE_HEADER_LENGTH: usize = 6;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributeShell {
    pub name: JvmString,
    pub span: ByteSpan,
    pub content_span: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemberHeader {
    pub name: JvmString,
    pub descriptor: JvmString,
    pub access_flags: u16,
    pub attributes: Vec<AttributeShell>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct MinimalHeaderFacts {
    pub major_version: u16,
    pub minor_version: u16,
    pub access_flags: u16,
    pub this_class: JvmString,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassHeader {
    pub major_version: u16,
    pub minor_version: u16,
    pub access_flags: u16,
    pub this_class: JvmString,
    pub super_class: Option<JvmString>,
    pub interfaces: Vec<JvmString>,
    pub fields: Vec<MemberHeader>,
    pub methods: Vec<MemberHeader>,
    pub attributes: Vec<AttributeShell>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionMode {
    Strict,
    Forensic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassfileVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionRuleStatus {
    Valid,
    InvalidMajor,
    InvalidModernMinor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionDialectSupport {
    Supported,
    StructuralProbeOnly,
    UnsupportedPreview,
    FutureRelease,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DialectValidationScope {
    VersionOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Java8RuntimeCompatibility {
    Accepted,
    Rejected,
}

/// Whether a declared verification of this artifact really ran, and what it concluded.
///
/// `NotPerformed` is the only variant P0, P1 and P2 produce, and it stays the honest state of
/// every header, bytecode and multi-release report they assemble: a structural read, a dialect
/// validation or an IR phase is not a verification, so the plane MUST NOT be read as "the
/// artifact is legal" or "the input can be linked or run".
///
/// `Performed` and `Failed` are the vocabulary later phases state when they really run one: the
/// recovery layer's controlled check (P3 3.3's recompile and behavior comparison) verifies the
/// artifact it produced in a declared environment, and either that check accepted it or it did
/// not. Both variants are evidence about the sample, compiler and environment they ran under,
/// never a general claim over other inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    NotPerformed,
    Performed,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeaderStructuralRead {
    Complete,
}

/// Whether the version's `minor_version` carries the preview marker (JVMS 4.1).
///
/// The marker is a fact about the *format*: from major 56 on, minor 65535 marks a preview class
/// file. It says nothing about whether this build validates a preview dialect — that is
/// [`VersionCapability::version_dialect_support`], and the two stay separate statements so that a
/// marker this build does not support can never read as support.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewMarker {
    Absent,
    Present,
}

/// The output level an artifact's modern facts are asked to be consumed at.
///
/// A level is a *request*, not a property of the artifact: a class compiled at major 61 is asked to
/// be consumed by a Java 8 toolchain, and the answer is stated over the facts that class really
/// carries — never over the compiler that produced it and never over its own `major_version`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputLevel {
    /// Java 8: the level the recovery layer this reader feeds targets.
    Java8,
}

/// One modern construct a target output level either represents or cannot.
///
/// The three are exactly the constructs P4 design decision 5 names for the Java 8 question: record
/// components, sealed permitted subclasses and a `StringConcatFactory` `invokedynamic` site. Module,
/// nestmate and constant-dynamic facts are Java 9+ structures too, and this vocabulary deliberately
/// makes no claim about them: a feature is listed here only because the pass that reports it decides
/// the output-level answer for it, and inventing a conflict for a construct no pass evaluates would
/// report a verdict nobody produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModernFeature {
    /// The `Record` attribute and its components (JVMS 4.7.30).
    RecordComponents,
    /// The `PermittedSubclasses` attribute of a sealed class (JVMS 4.7.31).
    PermittedSubclasses,
    /// A `java.lang.invoke.StringConcatFactory` `invokedynamic` site.
    StringConcat,
}

/// Where one modern fact was read from, kept so the fact's **modern origin** survives an
/// output-level conflict (P4 design decision 5): the fallback answer names the bytes the fact came
/// from, and those bytes are never replaced by an equivalent-looking Java 8 construct.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModernOrigin {
    /// A class-level `attribute_info`: the raw name, the `attribute_name_index` that spells it, the
    /// class-file range of the whole entry and the range of its content.
    ClassAttribute {
        name: JvmBytes,
        name_index: u16,
        span: ByteSpan,
        content_span: ByteSpan,
    },
    /// The constant-pool entry the construct lives in, by 1-based index, with its own span.
    ConstantPoolEntry { index: u16, span: ByteSpan },
}

/// One modern fact a target output level cannot represent.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputLevelConflict {
    pub feature: ModernFeature,
    pub level: OutputLevel,
    /// The lowest `major_version` whose release defines the construct, taken from the release-bound
    /// registry rule the fact was read under.
    pub since: u16,
    /// The JVMS section that rule was read from.
    pub source: String,
    /// The bytes the fact came from. The fact itself is still published by the pass that read it:
    /// the conflict is reported, not resolved by rewriting the construct.
    pub origin: ModernOrigin,
}

/// Whether an output-level evaluation ran over this artifact, and what it concluded.
///
/// `NotEvaluated` is the only variant the **header** plane produces: classifying a version and
/// reading a structure is not a statement about what an output level can represent. `Representable`
/// and `Conflict` are produced by the pass that really evaluates the questions over the modern
/// facts ([`crate::modern::modern_facts`], P4 1.2), and `Conflict` is the shape design decision 5
/// asks for: the Java 8 output level returns the conflict **and** keeps each conflicting fact as a
/// fallback that still names its modern origin, instead of hiding a syntax transformation behind an
/// equivalent-looking downgrade.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputLevelStatus {
    NotEvaluated,
    /// The evaluation ran over the artifact's modern facts and every one of them has an equivalent
    /// at this level.
    Representable {
        level: OutputLevel,
    },
    /// The evaluation ran and at least one modern fact has no equivalent at this level. Each
    /// conflict names the fact's origin, the release that requires the construct and the JVMS
    /// section that release rule was read from.
    Conflict {
        level: OutputLevel,
        conflicts: Vec<OutputLevelConflict>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VersionCapability {
    pub version: ClassfileVersion,
    pub version_rule: VersionRuleStatus,
    /// The dialect support this build declares, read from the release-bound registry: `Supported`
    /// only where the registry validates the dialect, `StructuralProbeOnly` where the structure is
    /// read without dialect validation, and the preview/future states where the registry holds no
    /// validating record. The registry's tag, attribute, flag and opcode constraints are stated by
    /// [`crate::release_registry`] and are not applied here yet, which is what
    /// `dialect_validation_scope` records.
    pub version_dialect_support: VersionDialectSupport,
    pub dialect_validation_scope: DialectValidationScope,
    pub java8_runtime: Java8RuntimeCompatibility,
    /// Whether the input carries the release's preview marker.
    pub preview_marker: PreviewMarker,
    /// Whether the registry holds a record for the input's major.
    ///
    /// `UnregisteredFutureRelease` and `UnregisteredBelowMinimum` are the conservative path: the
    /// registry claims nothing about such a version's constraints, so no capability of a registered
    /// release may be read into it.
    pub release_registration: ReleaseRegistration,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HeaderInspection {
    pub header: ClassHeader,
    /// Parse plane: how much of the structure this read established. `Complete` says the header was
    /// read to the end of its schema — nothing about the dialect, and nothing about verification.
    pub structural_read: HeaderStructuralRead,
    /// Dialect-validation plane: the version rule, and the dialect support the registry declares for
    /// the release.
    pub version_capability: VersionCapability,
    /// Verification plane: whether a verifier really ran, and what it concluded. This layer never
    /// runs one, so it states `NotPerformed` whatever the dialect plane says.
    pub verification: VerificationStatus,
    /// Output-level plane: whether an output-level evaluation ran. This layer never runs one.
    pub output_level: OutputLevelStatus,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MethodSelector {
    pub name: JvmBytes,
    pub descriptor: JvmBytes,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstructionFact {
    pub bci: u32,
    pub opcode: u8,
    pub width: u32,
    pub span: ByteSpan,
    pub operands_span: ByteSpan,
    pub constant_pool_index: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExceptionHandlerFact {
    pub ordinal: u32,
    pub start_bci: u32,
    pub end_bci: u32,
    pub handler_bci: u32,
    pub catch_type_index: Option<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BytecodeStopPhase {
    ExceptionHandlers,
    Instructions,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum BytecodeStop {
    ExceptionHandlers {
        ordinal: u32,
        class_offset: u64,
        code: String,
    },
    Instructions {
        bci: u32,
        class_offset: u64,
        code: String,
    },
}

impl BytecodeStop {
    pub const fn phase(&self) -> BytecodeStopPhase {
        match self {
            Self::ExceptionHandlers { .. } => BytecodeStopPhase::ExceptionHandlers,
            Self::Instructions { .. } => BytecodeStopPhase::Instructions,
        }
    }
}

/// Coverage of one body decode, in the reader's own BCI and handler-ordinal coordinates.
///
/// One body has one coverage plane, whichever reader path produced it: the public
/// [`BytecodeInspection`] and the crate-private [`MethodCodeFacts`] call this same mapping, so
/// an analysis caller and a bytecode caller cannot publish two different ranges for the same
/// `Code` attribute. The scanned ranges are the decoded instruction prefix
/// (`method_code_bci`) and the handler records that were read
/// (`exception_handler_ordinal`), the skipped ranges are the rest of each, and the state is
/// `CompleteWithinSchema` exactly when the decode ran to the end of the body.
///
/// Handlers are decoded before instructions, so a handler-phase stop leaves
/// `handlers_returned < handlers_total` while the instruction prefix is empty, and an
/// instruction-phase stop leaves the handler list complete.
pub fn method_code_coverage(
    code_length: u64,
    instructions: &[InstructionFact],
    handlers_returned: usize,
    handlers_total: u32,
    execution: &ExecutionReport,
    stopped_at: Option<&BytecodeStop>,
) -> Result<Coverage> {
    let prefix_end = instructions.last().map_or(Ok(0), |fact| {
        u64::from(fact.bci)
            .checked_add(u64::from(fact.width))
            .ok_or_else(|| {
                Error::invalid_input("classfile_coverage_overflow", "instruction prefix overflow")
            })
    })?;
    if prefix_end > code_length {
        return Err(Error::invalid_input(
            "classfile_coverage_out_of_bounds",
            "instruction prefix exceeds code length",
        ));
    }
    let handlers_returned = u64::try_from(handlers_returned).map_err(|_| {
        Error::invalid_input(
            "classfile_coverage_overflow",
            "handler count does not fit u64",
        )
    })?;
    let handlers_total = u64::from(handlers_total);
    if handlers_returned > handlers_total {
        return Err(Error::invalid_input(
            "classfile_coverage_out_of_bounds",
            "returned handler count exceeds declared count",
        ));
    }
    let complete = matches!(execution, ExecutionReport::Complete { .. });
    let handlers_complete = complete
        || !matches!(
            stopped_at.map(BytecodeStop::phase),
            Some(BytecodeStopPhase::ExceptionHandlers)
        );
    let mut scanned = vec![CoverageRange {
        label: "method_code_bci".into(),
        start: 0,
        end: prefix_end,
    }];
    scanned.push(CoverageRange {
        label: "exception_handler_ordinal".into(),
        start: 0,
        end: handlers_returned,
    });
    let mut skipped = Vec::new();
    if prefix_end < code_length {
        skipped.push(CoverageRange {
            label: "method_code_bci".into(),
            start: prefix_end,
            end: code_length,
        });
    }
    if !handlers_complete && handlers_returned < handlers_total {
        skipped.push(CoverageRange {
            label: "exception_handler_ordinal".into(),
            start: handlers_returned,
            end: handlers_total,
        });
    }
    Ok(Coverage {
        artifact_structural: CoverageDimension {
            state: if complete {
                CoverageState::CompleteWithinSchema
            } else {
                CoverageState::Partial
            },
            scanned,
            skipped,
            uninterpreted_extensions: Vec::new(),
        },
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BytecodeInspection {
    pub selector: MethodSelector,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code_span: ByteSpan,
    pub instructions: Vec<InstructionFact>,
    pub exception_handlers: Vec<ExceptionHandlerFact>,
    pub exception_handler_count: u32,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
    pub verification: VerificationStatus,
    pub stopped_at: Option<BytecodeStop>,
}

pub(crate) fn probe_minimal_header(
    bytes: &[u8],
    budget: &mut Budget,
) -> Result<MinimalHeaderFacts> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::ClassBytes, to_u64(bytes.len())?)?;
    validate_constant_pool_slots(bytes, budget)?;
    let class = Class::new(bytes).map_err(map_decode_error)?;
    if class.buffer_size() != bytes.len() {
        return Err(Error::invalid_input(
            "classfile_trailing_bytes",
            "class structure has trailing bytes",
        ));
    }
    let pool = class.pool();
    let this_class = jvm_string_mstr(
        pool.retrieve(class.this_class())
            .map_err(map_decode_error)?
            .name,
    )?;
    Ok(MinimalHeaderFacts {
        major_version: class.version().major,
        minor_version: class.version().minor,
        access_flags: class.access_flags().bits(),
        this_class,
    })
}

/// Reads declaration-level structure and applies the version rule of the release-bound registry.
/// Attribute content, attribute and flag legality, instructions, and JVM verification are never
/// performed here: the registry states those constraints, and the passes that read the facts apply
/// them.
pub fn inspect_header(
    bytes: &[u8],
    budget: &mut Budget,
    mode: InspectionMode,
) -> Result<HeaderInspection> {
    let header = inspect_header_structure(bytes, budget)?;
    let version_capability = classify_version(header.major_version, header.minor_version);
    if mode == InspectionMode::Strict {
        enforce_strict_version_gate(&version_capability)?;
    }
    let diagnostics = version_diagnostics(&version_capability);
    budget.charge(
        CountedBudgetDimension::ResultItems,
        to_u64(diagnostics.len())?,
    )?;
    Ok(HeaderInspection {
        header,
        structural_read: HeaderStructuralRead::Complete,
        version_capability,
        verification: VerificationStatus::NotPerformed,
        output_level: OutputLevelStatus::NotEvaluated,
        diagnostics,
    })
}

/// Inspects one exact method body using noak's Java 8 instruction cursor.
pub fn inspect_method_bytecode(
    bytes: &[u8],
    selector: MethodSelector,
    budget: &mut Budget,
) -> Result<BytecodeInspection> {
    inspect_method_bytecode_with(bytes, selector, budget, |_| {}, |_| {})
}

/// Every instruction opcode of every method body in one class, one entry per method in declaration
/// order, decoded by the same cursor the bytecode read path uses.
///
/// The fact pass needs the *opcodes* a release gates (`invokedynamic`, `jsr`, `ret` and the reserved
/// opcodes) without publishing a second body report: an instruction fact, its span and the BCI a
/// stop happened at belong to [`inspect_method_bytecode`], which charges the body it reads and says
/// where it stopped. This probe states opcodes and nothing else, so no instruction fact is published
/// twice.
///
/// # Failure and charging
///
/// A body whose structure does not decode states **no** opcode instead of failing the probe — the
/// convention `local_debug_table` already follows — because a body this probe cannot read is no
/// evidence that the release's opcode rules hold in it, and it must not turn a class whose
/// declaration structure read completely into an error. The probe charges `CodeBytes` for every code
/// array it reads, the dimension the read path charges for the same bytes; the class structure and
/// its constant pool are the structure the caller already holds and are charged no second time.
pub(crate) fn method_body_opcodes(bytes: &[u8], budget: &mut Budget) -> Result<Vec<Vec<u8>>> {
    budget.poll()?;
    let Ok(class) = Class::new(bytes) else {
        return Ok(Vec::new());
    };
    let pool = class.pool();
    let mut bodies = Vec::new();
    for method in class.methods() {
        budget.poll()?;
        let Ok(method) = method else { break };
        let mut opcodes = Vec::new();
        'attributes: for attribute in method.attributes() {
            budget.poll()?;
            let Ok(attribute) = attribute else { break };
            let Ok(name) = pool.get(attribute.name()) else {
                break;
            };
            if name.content.as_bytes() != b"Code" {
                continue;
            }
            // The code array is the `Code` content's own slice (JVMS 4.7.3), bounded exactly as the
            // read path bounds it before it decodes the same bytes.
            let content = attribute.content();
            let Ok(code_length) = read_u32(content, 4) else {
                break;
            };
            let Ok(code_length) = usize::try_from(code_length) else {
                break;
            };
            let Some(code) = content.get(8..).and_then(|rest| rest.get(..code_length)) else {
                break;
            };
            let Ok(noak::reader::AttributeContent::Code(decoded)) = attribute.read_content(pool)
            else {
                break;
            };
            budget.charge(CountedBudgetDimension::CodeBytes, to_u64(code.len())?)?;
            for item in decoded.raw_instructions() {
                budget.poll()?;
                let Ok((index, _instruction)) = item else {
                    break 'attributes;
                };
                let Ok(at) = usize::try_from(index.as_u32()) else {
                    break 'attributes;
                };
                let Some(opcode) = code.get(at) else {
                    break 'attributes;
                };
                opcodes.push(*opcode);
            }
        }
        bodies.push(opcodes);
    }
    Ok(bodies)
}

fn inspect_method_bytecode_with(
    bytes: &[u8],
    selector: MethodSelector,
    budget: &mut Budget,
    before_handler: impl FnMut(u32),
    before_instruction: impl FnMut(u32),
) -> Result<BytecodeInspection> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::ClassBytes, to_u64(bytes.len())?)?;
    validate_constant_pool_slots(bytes, budget)?;
    let class = Class::new(bytes).map_err(map_decode_error)?;
    if class.buffer_size() != bytes.len() {
        return Err(Error::invalid_input(
            "classfile_trailing_bytes",
            "class structure has trailing bytes",
        ));
    }
    let capability = classify_version(class.version().major, class.version().minor);
    enforce_strict_version_gate(&capability)?;
    let pool = class.pool();
    let mut matches = Vec::new();
    for method in class.methods() {
        budget.poll()?;
        let method = method.map_err(map_decode_error)?;
        let name = pool
            .get(method.name())
            .map_err(map_decode_error)?
            .content
            .as_bytes();
        let descriptor = pool
            .get(method.descriptor())
            .map_err(map_decode_error)?
            .content
            .as_bytes();
        if name == selector.name.0 && descriptor == selector.descriptor.0 {
            matches.push(method);
        }
    }
    let method = match matches.len() {
        0 => {
            return Err(Error::invalid_input(
                "classfile_method_not_found",
                "no method exactly matched the raw name and descriptor",
            ));
        }
        1 => matches.pop().expect("length checked"),
        _ => {
            return Err(Error::invalid_input(
                "classfile_method_ambiguous",
                "multiple methods exactly matched the raw name and descriptor",
            ));
        }
    };

    let mut code_attributes = Vec::new();
    for attribute in method.attributes() {
        budget.poll()?;
        let attribute = attribute.map_err(map_decode_error)?;
        let name = pool
            .get(attribute.name())
            .map_err(map_decode_error)?
            .content
            .as_bytes();
        if name == b"Code" {
            code_attributes.push(attribute);
        }
    }
    let attribute = match code_attributes.len() {
        0 => {
            return Err(Error::unsupported(
                "classfile_method_has_no_code",
                "selected method has no Code attribute",
            ));
        }
        1 => code_attributes.pop().expect("length checked"),
        _ => {
            return Err(Error::invalid_input(
                "classfile_duplicate_code_attribute",
                "selected method has multiple Code attributes",
            ));
        }
    };
    let content = attribute.content();
    let content_span = span_for_slice(bytes, content)?;
    let shell_length = content_span
        .length
        .checked_add(ATTRIBUTE_HEADER_LENGTH as u64)
        .ok_or_else(|| {
            Error::invalid_input("classfile_span_overflow", "Code shell length overflow")
        })?;
    let shell_start = content_span
        .start
        .checked_sub(ATTRIBUTE_HEADER_LENGTH as u64)
        .ok_or_else(|| {
            Error::invalid_input(
                "classfile_invalid_attribute_span",
                "Code content begins before its shell",
            )
        })?;
    checked_span(bytes, shell_start, shell_length)?;
    budget.charge(CountedBudgetDimension::AttributeBytes, shell_length)?;
    let code_length = read_u32(content, 4)?;
    let code_length_usize = usize::try_from(code_length).map_err(|_| {
        Error::invalid_input(
            "classfile_code_span_overflow",
            "code length does not fit usize",
        )
    })?;
    let code_start_in_content = 8usize;
    let code_end = code_start_in_content
        .checked_add(code_length_usize)
        .ok_or_else(|| {
            Error::invalid_input("classfile_code_span_overflow", "code span overflow")
        })?;
    let code_bytes = content
        .get(code_start_in_content..code_end)
        .ok_or_else(|| {
            Error::invalid_input(
                "classfile_code_out_of_bounds",
                "code array exceeds Code attribute content",
            )
        })?;
    let code_start = content_span
        .start
        .checked_add(code_start_in_content as u64)
        .ok_or_else(|| {
            Error::invalid_input("classfile_code_span_overflow", "code start overflow")
        })?;
    let code_span = checked_span(bytes, code_start, u64::from(code_length))?;
    let decoded = attribute.read_content(pool).map_err(map_decode_error)?;
    let code = match decoded {
        noak::reader::AttributeContent::Code(code) => code,
        _ => unreachable!("raw Code name selected"),
    };
    budget.charge(CountedBudgetDimension::ResultItems, 1)?;
    decode_code(
        selector,
        code,
        code_bytes,
        code_span,
        budget,
        before_handler,
        before_instruction,
    )
}

fn decode_code(
    selector: MethodSelector,
    code: Code<'_>,
    code_bytes: &[u8],
    code_span: ByteSpan,
    budget: &mut Budget,
    mut before_handler: impl FnMut(u32),
    mut before_instruction: impl FnMut(u32),
) -> Result<BytecodeInspection> {
    let exception_handler_count =
        u32::try_from(code.exception_handlers().count()).map_err(|_| {
            Error::invalid_input(
                "classfile_result_overflow",
                "handler count does not fit u32",
            )
        })?;
    let mut handlers = Vec::new();
    for (ordinal, handler) in code.exception_handlers().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| {
            Error::invalid_input("classfile_result_overflow", "handler ordinal overflow")
        })?;
        before_handler(ordinal);
        if let Err(error) = budget.poll() {
            return terminated_bytecode(
                selector,
                &code,
                code_span,
                Vec::new(),
                handlers,
                BytecodeStopPosition::ExceptionHandler(ordinal),
                error,
                budget,
            );
        }
        if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 1) {
            return terminated_bytecode(
                selector,
                &code,
                code_span,
                Vec::new(),
                handlers,
                BytecodeStopPosition::ExceptionHandler(ordinal),
                error,
                budget,
            );
        }
        handlers.push(ExceptionHandlerFact {
            ordinal,
            start_bci: handler.start().as_u32(),
            end_bci: handler.end().as_u32(),
            handler_bci: handler.handler().as_u32(),
            catch_type_index: handler.catch_type().map(|index| index.as_u16()),
        });
    }

    let mut instructions = Vec::new();
    let mut cursor_bci = 0u32;
    let raw = code.raw_instructions();
    for item in raw {
        before_instruction(cursor_bci);
        if let Err(error) = budget.poll() {
            return terminated_bytecode(
                selector,
                &code,
                code_span,
                instructions,
                handlers,
                BytecodeStopPosition::Instruction(cursor_bci),
                error,
                budget,
            );
        }
        let (index, instruction) = match item {
            Ok(value) => value,
            Err(error) => {
                return decode_failed_bytecode(
                    selector,
                    &code,
                    code_span,
                    instructions,
                    handlers,
                    cursor_bci,
                    error,
                    budget,
                );
            }
        };
        if index.as_u32() != cursor_bci {
            return Err(Error::invalid_input(
                "classfile_instruction_cursor_mismatch",
                format!(
                    "noak returned BCI {} but expected {cursor_bci}",
                    index.as_u32()
                ),
            ));
        }
        let start = usize::try_from(cursor_bci).map_err(|_| {
            Error::invalid_input("classfile_code_span_overflow", "BCI does not fit usize")
        })?;
        let opcode = *code_bytes.get(start).ok_or_else(|| {
            Error::invalid_input(
                "classfile_instruction_out_of_bounds",
                "instruction opcode is outside code array",
            )
        })?;
        let width = match instruction_width(opcode, cursor_bci, &instruction) {
            Ok(width) => width,
            Err(error @ Error::InvalidInput { .. }) => {
                return adapter_failed_bytecode(
                    selector,
                    &code,
                    code_span,
                    instructions,
                    handlers,
                    cursor_bci,
                    error,
                    budget,
                );
            }
            Err(error) => return Err(error),
        };
        let end = match cursor_bci.checked_add(width) {
            Some(end) => end,
            None => {
                return adapter_failed_bytecode(
                    selector,
                    &code,
                    code_span,
                    instructions,
                    handlers,
                    cursor_bci,
                    Error::invalid_input(
                        "classfile_instruction_width_overflow",
                        "instruction end overflow",
                    ),
                    budget,
                );
            }
        };
        if end
            > u32::try_from(code_bytes.len()).map_err(|_| {
                Error::invalid_input(
                    "classfile_code_span_overflow",
                    "code length does not fit u32",
                )
            })?
        {
            return Err(Error::invalid_input(
                "classfile_instruction_out_of_bounds",
                "instruction exceeds code array",
            ));
        }
        if let Err(error) = budget.check(CountedBudgetDimension::CodeBytes, u64::from(width)) {
            return terminated_bytecode(
                selector,
                &code,
                code_span,
                instructions,
                handlers,
                BytecodeStopPosition::Instruction(cursor_bci),
                error,
                budget,
            );
        }
        if let Err(error) = budget.check(CountedBudgetDimension::ResultItems, 1) {
            return terminated_bytecode(
                selector,
                &code,
                code_span,
                instructions,
                handlers,
                BytecodeStopPosition::Instruction(cursor_bci),
                error,
                budget,
            );
        }
        if let Err(error) = budget.charge(CountedBudgetDimension::CodeBytes, u64::from(width)) {
            return terminated_bytecode(
                selector,
                &code,
                code_span,
                instructions,
                handlers,
                BytecodeStopPosition::Instruction(cursor_bci),
                error,
                budget,
            );
        }
        if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 1) {
            return terminated_bytecode(
                selector,
                &code,
                code_span,
                instructions,
                handlers,
                BytecodeStopPosition::Instruction(cursor_bci),
                error,
                budget,
            );
        }
        let span_start = code_span
            .start
            .checked_add(u64::from(cursor_bci))
            .ok_or_else(|| {
                Error::invalid_input("classfile_code_span_overflow", "instruction span overflow")
            })?;
        let end_usize = usize::try_from(end).map_err(|_| {
            Error::invalid_input(
                "classfile_code_span_overflow",
                "instruction end does not fit usize",
            )
        })?;
        let instruction_bytes = code_bytes.get(start..end_usize).ok_or_else(|| {
            Error::invalid_input(
                "classfile_instruction_out_of_bounds",
                "instruction slice exceeds code array",
            )
        })?;
        instructions.push(InstructionFact {
            bci: cursor_bci,
            opcode,
            width,
            span: ByteSpan::new(span_start, u64::from(width)),
            operands_span: ByteSpan::new(span_start + 1, u64::from(width - 1)),
            constant_pool_index: constant_pool_index(opcode, instruction_bytes),
        });
        cursor_bci = end;
    }
    if cursor_bci != code_bytes.len() as u32 {
        return Err(Error::invalid_input(
            "classfile_instruction_cursor_mismatch",
            "instruction cursor did not consume the code array",
        ));
    }

    Ok(BytecodeInspection {
        selector,
        max_stack: code.max_stack(),
        max_locals: code.max_locals(),
        code_span,
        instructions,
        exception_handlers: handlers,
        exception_handler_count,
        execution: ExecutionReport::Complete {
            usage: budget.usage(),
        },
        diagnostics: Vec::new(),
        verification: VerificationStatus::NotPerformed,
        stopped_at: None,
    })
}

fn inspect_header_structure(bytes: &[u8], budget: &mut Budget) -> Result<ClassHeader> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::ClassBytes, to_u64(bytes.len())?)?;
    validate_constant_pool_slots(bytes, budget)?;

    let class = Class::new(bytes).map_err(map_decode_error)?;
    if class.buffer_size() != bytes.len() {
        return Err(Error::invalid_input(
            "classfile_trailing_bytes",
            format!(
                "class structure ended at position {}, but input length is {}",
                class.buffer_size(),
                bytes.len()
            ),
        ));
    }

    let pool = class.pool();
    let this_class = jvm_string_mstr(
        pool.retrieve(class.this_class())
            .map_err(map_decode_error)?
            .name,
    )?;
    let super_class = class
        .super_class()
        .map(|index| {
            let name = pool.retrieve(index).map_err(map_decode_error)?.name;
            jvm_string_mstr(name)
        })
        .transpose()?;

    let mut interfaces = Vec::new();
    for interface in class.interfaces() {
        budget.poll()?;
        let name = pool
            .retrieve(interface.map_err(map_decode_error)?)
            .map_err(map_decode_error)?
            .name;
        charge_item(budget)?;
        interfaces.push(jvm_string_mstr(name)?);
    }

    let mut fields = Vec::new();
    for field in class.fields() {
        budget.poll()?;
        let field = field.map_err(map_decode_error)?;
        let attributes = collect_attributes(bytes, pool, field.attributes(), budget)?;
        charge_item(budget)?;
        fields.push(MemberHeader {
            name: jvm_string(field.name(), pool)?,
            descriptor: jvm_string(field.descriptor(), pool)?,
            access_flags: field.access_flags().bits(),
            attributes,
        });
    }

    let mut methods = Vec::new();
    for method in class.methods() {
        budget.poll()?;
        let method = method.map_err(map_decode_error)?;
        let attributes = collect_attributes(bytes, pool, method.attributes(), budget)?;
        charge_item(budget)?;
        methods.push(MemberHeader {
            name: jvm_string(method.name(), pool)?,
            descriptor: jvm_string(method.descriptor(), pool)?,
            access_flags: method.access_flags().bits(),
            attributes,
        });
    }

    let attributes = collect_attributes(bytes, pool, class.attributes(), budget)?;
    charge_item(budget)?;
    Ok(ClassHeader {
        major_version: class.version().major,
        minor_version: class.version().minor,
        access_flags: class.access_flags().bits(),
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    })
}

/// Classifies one class-file version from the release-bound registry.
///
/// P0 compared `major` and `minor` against literal ranges here. The rules are the same rules, but
/// they are now read from the registry record of the release the version names, so the version rule,
/// the dialect band, the preview marker and the Java 8 profile all follow the release table instead
/// of a second copy of it:
///
/// - the major floor, the minor rule of modern releases and the preview marker are the registry's
///   own constants and rules (JVMS 4.1), and the minor rule deliberately reaches *above* the
///   registry ceiling — it is a rule of the format, not a capability of a recorded release;
/// - the dialect support and the Java 8 profile verdict are the record's, and a major the registry
///   does not hold makes no claim: a later release states [`VersionDialectSupport::FutureRelease`],
///   a major below the format's minimum keeps the structural-probe band it always had;
/// - preview support is decided by the record's preview rule, never by the marker alone, so a
///   preview class file this build does not validate reports `UnsupportedPreview` while its
///   `preview_marker` stays `Present`.
fn classify_version(major: u16, minor: u16) -> VersionCapability {
    let registry = feature_registry();
    let release = registry.release(major);
    let version_rule = if major < MINIMUM_MAJOR {
        VersionRuleStatus::InvalidMajor
    } else if major >= registry.modern_minor_since() && minor != 0 && minor != PREVIEW_MARKER {
        VersionRuleStatus::InvalidModernMinor
    } else {
        VersionRuleStatus::Valid
    };
    let preview_marker = if major >= registry.modern_minor_since() && minor == PREVIEW_MARKER {
        PreviewMarker::Present
    } else {
        PreviewMarker::Absent
    };
    let version_dialect_support = match release {
        ReleaseLookup::Registered(record) => match (preview_marker, record.preview()) {
            (PreviewMarker::Present, Some(preview)) => preview.dialect_support,
            _ => record.dialect_support(),
        },
        ReleaseLookup::UnregisteredFutureRelease => VersionDialectSupport::FutureRelease,
        ReleaseLookup::UnregisteredBelowMinimum => VersionDialectSupport::StructuralProbeOnly,
    };
    let java8_runtime = match release {
        ReleaseLookup::Registered(record) => record.java8_runtime(minor),
        ReleaseLookup::UnregisteredFutureRelease | ReleaseLookup::UnregisteredBelowMinimum => {
            Java8RuntimeCompatibility::Rejected
        }
    };
    VersionCapability {
        version: ClassfileVersion { major, minor },
        version_rule,
        version_dialect_support,
        dialect_validation_scope: DialectValidationScope::VersionOnly,
        java8_runtime,
        preview_marker,
        release_registration: release.registration(),
    }
}

/// The version rule of one class-file version, as this reader classifies it, from the two fields
/// alone and without reading the bytes they came from.
///
/// [`inspect_header`] applies the rule to the header it read; a caller that already holds the
/// version fields — a method-analysis request reads the header by identity instead of through
/// 1.2's inspection — asks this function instead of restating the rule, so the two plans cannot
/// drift apart. `major` below 45 is below the minimum the format defines, and a `minor` that is
/// neither `0` nor `65535` contradicts a major of 56 or above (JVMS 4.1). Both numbers and the
/// dialect bands they select are read from [`crate::release_registry`]; the constraints that table
/// holds for tags, attributes, flags and opcodes are queried there rather than restated here.
pub fn version_capability(major: u16, minor: u16) -> VersionCapability {
    classify_version(major, minor)
}

/// The refusal the format's own version rule states for one version, if the version violates it.
///
/// The diagnostic is the very one [`inspect_header`] publishes for the same version — the reader's
/// own code, message and `Error` severity — so a caller that refuses an illegal version states the
/// fact the header plan states instead of inventing a second wording. `None` for a version the rule
/// accepts, whatever dialect support this build has for it: what a version *is* and what this build
/// supports of it are two different statements, and only the first one is a refusal.
pub fn version_rule_diagnostic(capability: &VersionCapability) -> Option<Diagnostic> {
    match capability.version_rule {
        VersionRuleStatus::Valid => None,
        VersionRuleStatus::InvalidMajor => Some(version_diagnostic(
            "classfile_invalid_major_version",
            DiagnosticSeverity::Error,
            format!(
                "classfile major version {} is below the minimum valid major 45",
                capability.version.major
            ),
        )),
        VersionRuleStatus::InvalidModernMinor => Some(version_diagnostic(
            "classfile_invalid_modern_minor_version",
            DiagnosticSeverity::Error,
            format!(
                "classfile version {}.{} requires minor 0 or 65535 for major >= 56",
                capability.version.major, capability.version.minor
            ),
        )),
    }
}

fn version_diagnostics(capability: &VersionCapability) -> Vec<Diagnostic> {
    let mut diagnostics: Vec<Diagnostic> =
        version_rule_diagnostic(capability).into_iter().collect();
    match capability.version_dialect_support {
        VersionDialectSupport::Supported => {}
        VersionDialectSupport::StructuralProbeOnly => diagnostics.push(version_diagnostic(
            "classfile_version_structural_probe_only",
            DiagnosticSeverity::Warning,
            format!(
                "classfile version {}.{} is only structurally inspected; the release registry validates the dialect up to major {}",
                capability.version.major,
                capability.version.minor,
                feature_registry().dialect_validated_ceiling()
            ),
        )),
        VersionDialectSupport::UnsupportedPreview => diagnostics.push(version_diagnostic(
            "classfile_preview_unsupported",
            DiagnosticSeverity::Warning,
            format!(
                "preview classfile version {}.{} is not supported: the registry records the preview marker and this build validates no preview dialect",
                capability.version.major, capability.version.minor
            ),
        )),
        VersionDialectSupport::FutureRelease => diagnostics.push(version_diagnostic(
            "classfile_future_release",
            DiagnosticSeverity::Warning,
            format!(
                "classfile version {}.{} is above the highest release the registry holds (major {})",
                capability.version.major, capability.version.minor, HIGHEST_REGISTERED_MAJOR
            ),
        )),
    }
    if capability.java8_runtime == Java8RuntimeCompatibility::Rejected {
        diagnostics.push(version_diagnostic(
            "classfile_java8_runtime_rejected",
            DiagnosticSeverity::Warning,
            format!(
                "classfile version {}.{} is not accepted by the Java 8 runtime profile",
                capability.version.major, capability.version.minor
            ),
        ));
    }
    diagnostics
}

fn version_diagnostic(code: &str, severity: DiagnosticSeverity, message: String) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity,
        message,
        provenance: None,
    }
}

fn enforce_strict_version_gate(capability: &VersionCapability) -> Result<()> {
    match capability.version_rule {
        VersionRuleStatus::InvalidMajor => {
            return Err(Error::invalid_input(
                "classfile_invalid_major_version",
                format!(
                    "classfile major version {} is below the minimum valid major 45",
                    capability.version.major
                ),
            ));
        }
        VersionRuleStatus::InvalidModernMinor => {
            return Err(Error::invalid_input(
                "classfile_invalid_modern_minor_version",
                format!(
                    "classfile version {}.{} requires minor 0 or 65535 for major >= 56",
                    capability.version.major, capability.version.minor
                ),
            ));
        }
        VersionRuleStatus::Valid => {}
    }
    match capability.version_dialect_support {
        VersionDialectSupport::Supported => {}
        VersionDialectSupport::StructuralProbeOnly => {
            return Err(Error::unsupported(
                "classfile_version_structural_probe_only",
                format!(
                    "strict header inspection requires a dialect-validated release; the registry validates the dialect up to major {}",
                    feature_registry().dialect_validated_ceiling()
                ),
            ));
        }
        VersionDialectSupport::UnsupportedPreview => {
            return Err(Error::unsupported(
                "classfile_preview_unsupported",
                "strict header inspection validates no preview dialect",
            ));
        }
        VersionDialectSupport::FutureRelease => {
            return Err(Error::unsupported(
                "classfile_future_release",
                format!(
                    "strict header inspection does not support releases above the registry's highest registered release (major {})",
                    HIGHEST_REGISTERED_MAJOR
                ),
            ));
        }
    }
    if capability.java8_runtime == Java8RuntimeCompatibility::Rejected {
        return Err(Error::unsupported(
            "classfile_java8_runtime_rejected",
            "strict header inspection requires Java 8 runtime profile acceptance",
        ));
    }
    Ok(())
}

fn instruction_width(opcode: u8, bci: u32, instruction: &RawInstruction<'_>) -> Result<u32> {
    let width = match opcode {
        0xaa => {
            let RawInstruction::TableSwitch(table) = instruction else {
                return Err(Error::invalid_input(
                    "classfile_instruction_shape_mismatch",
                    "tableswitch shape mismatch",
                ));
            };
            let padding = switch_padding(bci);
            let count = i64::from(table.high())
                .checked_sub(i64::from(table.low()))
                .and_then(|value| value.checked_add(1))
                .filter(|count| *count > 0)
                .ok_or_else(|| {
                    Error::invalid_input(
                        "classfile_instruction_invalid_range",
                        "tableswitch high is less than low",
                    )
                })?;
            let entries = u32::try_from(count).map_err(|_| {
                Error::invalid_input(
                    "classfile_instruction_width_overflow",
                    "tableswitch entry count overflow",
                )
            })?;
            1u32.checked_add(padding)
                .and_then(|v| v.checked_add(12))
                .and_then(|v| entries.checked_mul(4).and_then(|n| v.checked_add(n)))
                .ok_or_else(|| {
                    Error::invalid_input(
                        "classfile_instruction_width_overflow",
                        "tableswitch width overflow",
                    )
                })?
        }
        0xab => {
            let RawInstruction::LookupSwitch(lookup) = instruction else {
                return Err(Error::invalid_input(
                    "classfile_instruction_shape_mismatch",
                    "lookupswitch shape mismatch",
                ));
            };
            let padding = switch_padding(bci);
            let pairs = u32::try_from(lookup.pairs().count()).map_err(|_| {
                Error::invalid_input(
                    "classfile_instruction_width_overflow",
                    "lookupswitch pair count overflow",
                )
            })?;
            1u32.checked_add(padding)
                .and_then(|v| v.checked_add(8))
                .and_then(|v| pairs.checked_mul(8).and_then(|n| v.checked_add(n)))
                .ok_or_else(|| {
                    Error::invalid_input(
                        "classfile_instruction_width_overflow",
                        "lookupswitch width overflow",
                    )
                })?
        }
        0xc4 => match instruction {
            RawInstruction::IIncW { .. } => 6,
            _ => 4,
        },
        0xb9 | 0xba => 5,
        0xc5 => 4,
        0xc8 | 0xc9 => 5,
        0x11
        | 0x13
        | 0x14
        | 0x84
        | 0x99..=0xa8
        | 0xb2..=0xb8
        | 0xbb
        | 0xbd
        | 0xc0
        | 0xc1
        | 0xc6
        | 0xc7 => 3,
        0x10 | 0x12 | 0x15..=0x19 | 0x36..=0x3a | 0xa9 | 0xbc => 2,
        _ => 1,
    };
    Ok(width)
}

fn constant_pool_index(opcode: u8, instruction: &[u8]) -> Option<u16> {
    match opcode {
        0x12 => instruction.get(1).copied().map(u16::from),
        0x13 | 0x14 | 0xb2..=0xb9 | 0xba | 0xbb | 0xbd | 0xc0 | 0xc1 | 0xc5 => {
            Some(u16::from_be_bytes([
                *instruction.get(1)?,
                *instruction.get(2)?,
            ]))
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BytecodeStopPosition {
    ExceptionHandler(u32),
    Instruction(u32),
}

fn bytecode_stop(
    code_span: &ByteSpan,
    position: BytecodeStopPosition,
    code: &str,
) -> Result<BytecodeStop> {
    match position {
        BytecodeStopPosition::ExceptionHandler(ordinal) => {
            let table_count_offset =
                code_span
                    .start
                    .checked_add(code_span.length)
                    .ok_or_else(|| {
                        Error::invalid_input(
                            "classfile_code_span_overflow",
                            "exception table count offset overflow",
                        )
                    })?;
            let table_start = table_count_offset.checked_add(2).ok_or_else(|| {
                Error::invalid_input(
                    "classfile_code_span_overflow",
                    "exception table start overflow",
                )
            })?;
            let record_offset = u64::from(ordinal).checked_mul(8).ok_or_else(|| {
                Error::invalid_input(
                    "classfile_code_span_overflow",
                    "exception table record offset overflow",
                )
            })?;
            let class_offset = table_start.checked_add(record_offset).ok_or_else(|| {
                Error::invalid_input(
                    "classfile_code_span_overflow",
                    "exception table record start overflow",
                )
            })?;
            Ok(BytecodeStop::ExceptionHandlers {
                ordinal,
                class_offset,
                code: code.to_owned(),
            })
        }
        BytecodeStopPosition::Instruction(bci) => {
            let class_offset = code_span.start.checked_add(u64::from(bci)).ok_or_else(|| {
                Error::invalid_input(
                    "classfile_code_span_overflow",
                    "instruction stop offset overflow",
                )
            })?;
            Ok(BytecodeStop::Instructions {
                bci,
                class_offset,
                code: code.to_owned(),
            })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn decode_failed_bytecode(
    selector: MethodSelector,
    code: &Code<'_>,
    code_span: ByteSpan,
    instructions: Vec<InstructionFact>,
    handlers: Vec<ExceptionHandlerFact>,
    bci: u32,
    error: noak::error::DecodeError,
    budget: &Budget,
) -> Result<BytecodeInspection> {
    let stable_code = "classfile_instruction_decode";
    let stopped_at = bytecode_stop(
        &code_span,
        BytecodeStopPosition::Instruction(bci),
        stable_code,
    )?;
    Ok(BytecodeInspection {
        selector,
        max_stack: code.max_stack(),
        max_locals: code.max_locals(),
        code_span,
        instructions,
        exception_handlers: handlers,
        exception_handler_count: u32::try_from(code.exception_handlers().count())
            .expect("Code exception count is bounded by u16"),
        execution: ExecutionReport::Partial {
            reason: TerminationReason::Error {
                code: stable_code.to_owned(),
            },
            usage: budget.usage(),
        },
        diagnostics: vec![version_diagnostic(
            stable_code,
            DiagnosticSeverity::Error,
            format!(
                "noak instruction decode failed: kind={}, context={}, position={}",
                error.kind(),
                error.context(),
                error
                    .position()
                    .map_or_else(|| "unknown".to_owned(), |v| v.to_string())
            ),
        )],
        verification: VerificationStatus::NotPerformed,
        stopped_at: Some(stopped_at),
    })
}

#[allow(clippy::too_many_arguments)]
fn adapter_failed_bytecode(
    selector: MethodSelector,
    code: &Code<'_>,
    code_span: ByteSpan,
    instructions: Vec<InstructionFact>,
    handlers: Vec<ExceptionHandlerFact>,
    bci: u32,
    error: Error,
    budget: &Budget,
) -> Result<BytecodeInspection> {
    let Error::InvalidInput {
        code: stable_code,
        message,
    } = error
    else {
        return Err(error);
    };
    let stopped_at = bytecode_stop(
        &code_span,
        BytecodeStopPosition::Instruction(bci),
        &stable_code,
    )?;
    Ok(BytecodeInspection {
        selector,
        max_stack: code.max_stack(),
        max_locals: code.max_locals(),
        code_span,
        instructions,
        exception_handlers: handlers,
        exception_handler_count: u32::try_from(code.exception_handlers().count())
            .expect("Code exception count is bounded by u16"),
        execution: ExecutionReport::Partial {
            reason: TerminationReason::Error {
                code: stable_code.clone(),
            },
            usage: budget.usage(),
        },
        diagnostics: vec![version_diagnostic(
            &stable_code,
            DiagnosticSeverity::Error,
            message,
        )],
        verification: VerificationStatus::NotPerformed,
        stopped_at: Some(stopped_at),
    })
}

#[allow(clippy::too_many_arguments)]
fn terminated_bytecode(
    selector: MethodSelector,
    code: &Code<'_>,
    code_span: ByteSpan,
    instructions: Vec<InstructionFact>,
    handlers: Vec<ExceptionHandlerFact>,
    position: BytecodeStopPosition,
    error: Error,
    budget: &Budget,
) -> Result<BytecodeInspection> {
    // Termination diagnostics and stopped_at explain an exhausted request and, like artifact
    // enumeration termination metadata, are intentionally not additional result items.
    let (execution, stable_code, severity) = match error {
        Error::Cancelled { .. } => (
            ExecutionReport::Cancelled {
                usage: budget.usage(),
            },
            "classfile_bytecode_cancelled",
            DiagnosticSeverity::Warning,
        ),
        Error::BudgetExceeded { dimension, .. } => (
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded { dimension },
                usage: budget.usage(),
            },
            "classfile_bytecode_budget_exceeded",
            DiagnosticSeverity::Warning,
        ),
        _ => (
            ExecutionReport::Partial {
                reason: TerminationReason::Error {
                    code: "classfile_bytecode_failed".to_owned(),
                },
                usage: budget.usage(),
            },
            "classfile_bytecode_failed",
            DiagnosticSeverity::Error,
        ),
    };
    let stopped_at = bytecode_stop(&code_span, position, stable_code)?;
    Ok(BytecodeInspection {
        selector,
        max_stack: code.max_stack(),
        max_locals: code.max_locals(),
        code_span,
        instructions,
        exception_handlers: handlers,
        exception_handler_count: u32::try_from(code.exception_handlers().count())
            .expect("Code exception count is bounded by u16"),
        execution,
        diagnostics: vec![version_diagnostic(
            stable_code,
            severity,
            "bytecode inspection stopped before completion".to_owned(),
        )],
        verification: VerificationStatus::NotPerformed,
        stopped_at: Some(stopped_at),
    })
}

/// Whether enumerating attribute shells also bills one `ResultItems` per shell.
///
/// `inspect_header` returns the shells themselves, so they are result items
/// there. The crate-private fact layer returns shells as intermediate facts, so
/// it bills only the bytes it reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttributeItemBilling {
    PerShell,
    NoResultItems,
}

fn collect_attributes<'a>(
    bytes: &'a [u8],
    pool: &noak::reader::cpool::ConstantPool<'a>,
    attributes: impl IntoIterator<Item = std::result::Result<Attribute<'a>, noak::error::DecodeError>>,
    budget: &mut Budget,
) -> Result<Vec<AttributeShell>> {
    collect_attribute_shells(
        bytes,
        pool,
        attributes,
        budget,
        AttributeItemBilling::PerShell,
    )
}

fn collect_attribute_shells<'a>(
    bytes: &'a [u8],
    pool: &noak::reader::cpool::ConstantPool<'a>,
    attributes: impl IntoIterator<Item = std::result::Result<Attribute<'a>, noak::error::DecodeError>>,
    budget: &mut Budget,
    billing: AttributeItemBilling,
) -> Result<Vec<AttributeShell>> {
    let mut shells = Vec::new();
    for attribute in attributes {
        budget.poll()?;
        let attribute = attribute.map_err(map_decode_error)?;
        let content_span = span_for_slice(bytes, attribute.content())?;
        let shell_start = content_span
            .start
            .checked_sub(ATTRIBUTE_HEADER_LENGTH as u64)
            .ok_or_else(|| {
                Error::invalid_input(
                    "classfile_invalid_attribute_span",
                    "attribute content begins before its header",
                )
            })?;
        let shell_length = content_span
            .length
            .checked_add(ATTRIBUTE_HEADER_LENGTH as u64)
            .ok_or_else(|| {
                Error::invalid_input("classfile_span_overflow", "attribute shell length overflow")
            })?;
        let span = checked_span(bytes, shell_start, shell_length)?;
        budget.charge(CountedBudgetDimension::AttributeBytes, shell_length)?;
        if billing == AttributeItemBilling::PerShell {
            charge_item(budget)?;
        }
        shells.push(AttributeShell {
            name: jvm_string(attribute.name(), pool)?,
            span,
            content_span,
        });
    }
    Ok(shells)
}

fn jvm_string<'a>(
    index: noak::reader::cpool::Index<noak::reader::cpool::Utf8<'a>>,
    pool: &noak::reader::cpool::ConstantPool<'a>,
) -> Result<JvmString> {
    jvm_string_mstr(pool.get(index).map_err(map_decode_error)?.content)
}

fn jvm_string_mstr(value: &noak::MStr) -> Result<JvmString> {
    let raw = value.as_bytes();
    let mut utf16 = Vec::new();
    let mut offset = 0usize;
    while offset < raw.len() {
        let first = raw[offset];
        let (unit, width) = if first < 0x80 {
            (u16::from(first), 1usize)
        } else if first & 0xe0 == 0xc0 {
            let second = raw[offset + 1];
            ((u16::from(first & 0x1f) << 6) | u16::from(second & 0x3f), 2)
        } else {
            let second = raw[offset + 1];
            let third = raw[offset + 2];
            (
                (u16::from(first & 0x0f) << 12)
                    | (u16::from(second & 0x3f) << 6)
                    | u16::from(third & 0x3f),
                3,
            )
        };
        utf16.push(unit);
        offset = offset.checked_add(width).ok_or_else(|| {
            Error::invalid_input("classfile_span_overflow", "Modified UTF-8 offset overflow")
        })?;
    }
    Ok(JvmString::from_parts(raw.to_vec(), utf16))
}

fn span_for_slice(container: &[u8], slice: &[u8]) -> Result<ByteSpan> {
    let start = (slice.as_ptr() as usize)
        .checked_sub(container.as_ptr() as usize)
        .ok_or_else(|| {
            Error::invalid_input(
                "classfile_invalid_attribute_span",
                "attribute content is outside class bytes",
            )
        })?;
    let end = start.checked_add(slice.len()).ok_or_else(|| {
        Error::invalid_input("classfile_span_overflow", "attribute content span overflow")
    })?;
    if end > container.len() {
        return Err(Error::invalid_input(
            "classfile_invalid_attribute_span",
            "attribute content is outside class bytes",
        ));
    }
    Ok(ByteSpan::new(to_u64(start)?, to_u64(slice.len())?))
}

fn checked_span(bytes: &[u8], start: u64, length: u64) -> Result<ByteSpan> {
    let end = start.checked_add(length).ok_or_else(|| {
        Error::invalid_input("classfile_span_overflow", "class-local span overflow")
    })?;
    if end > to_u64(bytes.len())? {
        return Err(Error::invalid_input(
            "classfile_invalid_attribute_span",
            "attribute shell is outside class bytes",
        ));
    }
    Ok(ByteSpan::new(start, length))
}

pub(crate) fn charge_item(budget: &mut Budget) -> Result<()> {
    budget.charge(CountedBudgetDimension::ResultItems, 1)
}

pub(crate) fn to_u64(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| {
        Error::invalid_input("classfile_size_overflow", "classfile size does not fit u64")
    })
}

fn map_decode_error(error: noak::error::DecodeError) -> Error {
    Error::invalid_input(
        "classfile_decode",
        format!(
            "noak decode failed: kind={}, context={}, position={}",
            error.kind(),
            error.context(),
            error
                .position()
                .map_or_else(|| "unknown".to_owned(), |position| position.to_string())
        ),
    )
}

/// noak 0.7.0 accepts a Long/Double in the final declared constant-pool slot.
/// Reject that malformed two-slot entry before relying on noak's pool representation.
fn validate_constant_pool_slots(bytes: &[u8], budget: &Budget) -> Result<()> {
    validate_constant_pool_slots_with(bytes, budget, |_| {})
}

fn validate_constant_pool_slots_with(
    bytes: &[u8],
    budget: &Budget,
    mut before_entry: impl FnMut(u16),
) -> Result<()> {
    if bytes.len() < 10 || !bytes.starts_with(&0xcafebabe_u32.to_be_bytes()) {
        return Ok(()); // noak owns fixed-header diagnostics.
    }
    let count = read_u16(bytes, 8)?;
    if count == 0 {
        return Ok(()); // noak supplies the canonical InvalidLength diagnostic.
    }
    let mut slot = 1u16;
    let mut offset = 10usize;
    while slot < count {
        before_entry(slot);
        budget.poll()?;
        let tag = *bytes.get(offset).ok_or_else(|| cp_guard_eoi(offset))?;
        offset = offset.checked_add(1).ok_or_else(cp_guard_overflow)?;
        let payload = match tag {
            1 => {
                let length = usize::from(read_u16(bytes, offset)?);
                offset = offset.checked_add(2).ok_or_else(cp_guard_overflow)?;
                length
            }
            3 | 4 => 4,
            5 | 6 => {
                if slot.checked_add(1).is_none_or(|reserved| reserved >= count) {
                    return Err(Error::invalid_input(
                        "classfile_invalid_constant_pool_slots",
                        format!(
                            "constant-pool tag {tag} at slot {slot} has no reserved following slot"
                        ),
                    ));
                }
                slot = slot.checked_add(1).ok_or_else(cp_guard_overflow)?;
                8
            }
            7 | 8 | 16 | 19 | 20 => 2,
            9 | 10 | 11 | 12 | 17 | 18 => 4,
            15 => 3,
            _ => return Ok(()), // noak owns tag diagnostics.
        };
        offset = offset.checked_add(payload).ok_or_else(cp_guard_overflow)?;
        if offset > bytes.len() {
            return Ok(()); // noak owns truncation diagnostics and position/context.
        }
        slot = slot.checked_add(1).ok_or_else(cp_guard_overflow)?;
    }
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset.checked_add(4).ok_or_else(cp_guard_overflow)?;
    let value: [u8; 4] = bytes
        .get(offset..end)
        .ok_or_else(|| cp_guard_eoi(offset))?
        .try_into()
        .expect("slice length was checked");
    Ok(u32::from_be_bytes(value))
}

pub(crate) fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let end = offset.checked_add(2).ok_or_else(cp_guard_overflow)?;
    let pair: [u8; 2] = bytes
        .get(offset..end)
        .ok_or_else(|| cp_guard_eoi(offset))?
        .try_into()
        .expect("slice length was checked");
    Ok(u16::from_be_bytes(pair))
}

fn cp_guard_eoi(position: usize) -> Error {
    Error::invalid_input(
        "classfile_decode",
        format!(
            "noak decode failed: kind=unexpected end of input, context=constant pool, position={position}"
        ),
    )
}

fn cp_guard_overflow() -> Error {
    Error::invalid_input("classfile_span_overflow", "constant-pool offset overflow")
}

// ---------------------------------------------------------------------------
// P1 crate-private reader facts
// ---------------------------------------------------------------------------
//
// The items below are the reader fact layer consumed by the structural XRef
// consumer streams (`xref::code`, `xref::metadata`, `xref::bootstrap`). They stay
// crate-private by design: the P0 public contract (`inspect_header`,
// `inspect_method_bytecode`, `HeaderInspection`, `BytecodeInspection`) is
// unchanged and no public caller ever sees a constant-pool table.
//
// Billing convention, using the same vocabulary as the existing `inspect_*`
// owners:
//
// * `ClassBytes` — the whole class length, charged once per `class_facts` call.
// * `AttributeBytes` — one attribute entry, `6 + content length`, charged when a
//   shell is enumerated and again when `attribute_content` reads that shell,
//   exactly like the existing `Code` read in `inspect_method_bytecode`: every
//   pass over an attribute entry is billed as the bytes that pass reads. A nested
//   attribute inside a `Code` attribute is covered by the one charge on the entry
//   that contains it: `code_nested_attributes` bills that entry once, walks the
//   body without decoding an instruction, and never bills its nested content again
//   (`attribute_slice` is the non-billing slice for exactly that content).
// * `ResultItems` — never charged here. `ResultItems` counts items returned by a
//   public result; the query layer charges it per emitted item. Billing
//   intermediate facts would let facts that are never returned exhaust a page
//   budget.
//
// This layer has no partial-prefix shape: cancellation and budget exhaustion
// return structured errors, so a truncated fact set can never be mistaken for a
// complete one.
//
// Every crate-private item below carries `#[allow(dead_code)]` because the layer
// lands before its in-crate consumers; the allowance is scope, not a permanent
// property, and it goes away as the consumers start using the item. The private
// helpers and tag constants are reachable from those annotated items, so rustc
// keeps them alive.
//
// Two rules decide whether a fact is stored raw or resolved:
//
// * A field whose declared type is a resolved value (a name, a descriptor) is
//   resolved here, so a broken index or tag becomes a structured error instead
//   of a fact the consumer cannot use.
// * A field whose declared type is an index (`constant_value`, `enclosing_method`,
//   `MethodHandle::reference_index`, `Dynamic::bootstrap_method_attr_index`) stays
//   raw; the consumer resolves it through `cp_entry`, which owns that error
//   contract.

/// A recorded constant-pool index whose entry is resolved by the consumer.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CpIndexOf(pub u16);

/// Payload of one constant-pool entry.
///
/// Field names follow the JVMS constant-pool layouts. `owner`, `name`,
/// `descriptor` and every other resolved value are the raw Modified UTF-8 bytes
/// held by the class file: they are never text-decoded and never normalised, so
/// a `.` is not rewritten into `/` and an array owner differs from its element
/// owner.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CpEntryKind {
    Utf8 {
        bytes: JvmBytes,
    },
    Integer {
        value: i32,
    },
    Float {
        bits: u32,
    },
    Long {
        value: i64,
    },
    Double {
        bits: u64,
    },
    Class {
        name_index: u16,
        name: JvmBytes,
    },
    String {
        utf8_index: u16,
        value: JvmBytes,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
        owner: JvmBytes,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
        owner: JvmBytes,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
        owner: JvmBytes,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
        descriptor: JvmBytes,
    },
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    Module {
        name_index: u16,
        name: JvmBytes,
    },
    Package {
        name_index: u16,
        name: JvmBytes,
    },
}

/// One constant-pool entry with its class-file byte range.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CpEntryFacts {
    /// 1-based constant-pool index, as used by instructions and attributes.
    pub index: u16,
    /// Byte range of the whole entry, tag byte plus payload, in class-file
    /// coordinates. Slots reserved by a preceding `Long`/`Double` have no entry
    /// and therefore no span.
    pub span: ByteSpan,
    pub kind: CpEntryKind,
}

/// Declaration-level facts plus the constant pool they refer to.
///
/// Field values are produced by the same noak-backed reads as `ClassHeader`,
/// but `class_facts` never runs the version gate and never rejects a version, so
/// forensic and strict callers both keep the structural view.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassFacts {
    pub major_version: u16,
    pub minor_version: u16,
    pub access_flags: u16,
    pub this_class: JvmString,
    pub super_class: Option<JvmString>,
    pub interfaces: Vec<JvmString>,
    pub fields: Vec<MemberHeader>,
    pub methods: Vec<MemberHeader>,
    pub attributes: Vec<AttributeShell>,
    pub constant_pool: Vec<CpEntryFacts>,
}

/// One `InnerClasses` entry.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InnerClassFacts {
    /// Index of the inner class; resolve it with `cp_class_name`.
    pub class_index: u16,
    /// Index of the enclosing class, or 0 when the class is not a member.
    pub outer_class_index: u16,
    /// Simple name from `inner_name_index`; `None` when that index is 0, which is
    /// the anonymous-class case and not a missing fact.
    pub inner_name: Option<JvmBytes>,
    pub access_flags: u16,
}

/// One `EnclosingMethod` attribute.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnclosingMethodFacts {
    /// Index of the immediately enclosing class; resolve it with `cp_class_name`.
    pub class_index: u16,
    /// Index of the enclosing `NameAndType`, or 0 when the class is not
    /// immediately enclosed by a method or constructor.
    pub method_index: u16,
}

/// One `Module` `provides` entry.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvidesFacts {
    /// Service interface internal name, expanded from `CONSTANT_Class`.
    pub service: JvmBytes,
    /// Provider internal names, in declaration order.
    pub implementations: Vec<JvmBytes>,
}

/// `Module` uses/provides facts. Requires, exports and opens are skipped as byte
/// ranges because no P1 consumer reads them.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleFacts {
    /// Service interfaces declared with `uses`, in declaration order.
    pub uses: Vec<JvmBytes>,
    pub provides: Vec<ProvidesFacts>,
}

/// Typed facts of the simple class-level and member-level attributes.
///
/// `None`/empty means the attribute was absent, never "unknown": a recognised
/// single-valued attribute that appears twice, and attribute content that does
/// not match its declared structure, are structured errors.
///
/// `Signature` strings and annotation content are deliberately not parsed here;
/// generic signature syntax and annotation element values belong to the
/// metadata consumer that owns those rules.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AttributeFacts {
    /// `Signature` (class, field, method): raw signature bytes.
    pub signature: Option<JvmBytes>,
    /// `Exceptions` (method): internal names, expanded from `CONSTANT_Class`.
    pub exceptions: Vec<JvmBytes>,
    /// `InnerClasses` (class).
    pub inner_classes: Vec<InnerClassFacts>,
    /// `EnclosingMethod` (class).
    pub enclosing_method: Option<EnclosingMethodFacts>,
    /// `NestHost` (class): internal name.
    pub nest_host: Option<JvmBytes>,
    /// `NestMembers` (class): internal names.
    pub nest_members: Vec<JvmBytes>,
    /// `PermittedSubclasses` (class): internal names.
    pub permitted_subclasses: Vec<JvmBytes>,
    /// `ConstantValue` (field): the raw index.
    pub constant_value: Option<CpIndexOf>,
    /// `Module` (class).
    pub module: Option<ModuleFacts>,
}

/// One `BootstrapMethods` entry.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapMethodFacts {
    /// Index of the bootstrap method handle; resolve it with `cp_entry`.
    pub method_ref: u16,
    /// Argument indexes in declaration order; each one is a loadable constant.
    pub arguments: Vec<u16>,
}

/// Reads declaration structure and the constant pool as reusable facts.
///
/// The class-shaped pre-checks (`validate_constant_pool_slots`, noak decode,
/// trailing-byte rejection) are the same ones the `inspect_*` owners run, so a
/// class accepted here is accepted there. Attribute content and instructions are
/// not decoded: callers ask for those per attribute or per method.
#[allow(dead_code)]
pub fn class_facts(bytes: &[u8], budget: &mut Budget) -> Result<ClassFacts> {
    budget.poll()?;
    budget.charge(CountedBudgetDimension::ClassBytes, to_u64(bytes.len())?)?;
    validate_constant_pool_slots(bytes, budget)?;
    let class = Class::new(bytes).map_err(map_decode_error)?;
    if class.buffer_size() != bytes.len() {
        return Err(Error::invalid_input(
            "classfile_trailing_bytes",
            "class structure has trailing bytes",
        ));
    }
    let layout = measure_constant_pool(bytes, budget)?;
    verify_constant_pool_layout(bytes, &layout, &class)?;
    let constant_pool = resolve_constant_pool(bytes, &layout, budget)?;

    let pool = class.pool();
    let this_class = jvm_string_mstr(
        pool.retrieve(class.this_class())
            .map_err(map_decode_error)?
            .name,
    )?;
    let super_class = class
        .super_class()
        .map(|index| {
            let name = pool.retrieve(index).map_err(map_decode_error)?.name;
            jvm_string_mstr(name)
        })
        .transpose()?;

    let mut interfaces = Vec::new();
    for interface in class.interfaces() {
        budget.poll()?;
        let name = pool
            .retrieve(interface.map_err(map_decode_error)?)
            .map_err(map_decode_error)?
            .name;
        interfaces.push(jvm_string_mstr(name)?);
    }

    let mut fields = Vec::new();
    for field in class.fields() {
        budget.poll()?;
        let field = field.map_err(map_decode_error)?;
        let attributes = collect_attribute_shells(
            bytes,
            pool,
            field.attributes(),
            budget,
            AttributeItemBilling::NoResultItems,
        )?;
        fields.push(MemberHeader {
            name: jvm_string(field.name(), pool)?,
            descriptor: jvm_string(field.descriptor(), pool)?,
            access_flags: field.access_flags().bits(),
            attributes,
        });
    }

    let mut methods = Vec::new();
    for method in class.methods() {
        budget.poll()?;
        let method = method.map_err(map_decode_error)?;
        let attributes = collect_attribute_shells(
            bytes,
            pool,
            method.attributes(),
            budget,
            AttributeItemBilling::NoResultItems,
        )?;
        methods.push(MemberHeader {
            name: jvm_string(method.name(), pool)?,
            descriptor: jvm_string(method.descriptor(), pool)?,
            access_flags: method.access_flags().bits(),
            attributes,
        });
    }

    let attributes = collect_attribute_shells(
        bytes,
        pool,
        class.attributes(),
        budget,
        AttributeItemBilling::NoResultItems,
    )?;

    Ok(ClassFacts {
        major_version: class.version().major,
        minor_version: class.version().minor,
        access_flags: class.access_flags().bits(),
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
        constant_pool,
    })
}

/// Slices one attribute content out of the class bytes and bills that entry.
///
/// The shell is taken from `ClassFacts`; a shell whose span does not match its
/// content span, or whose content leaves the class bytes, is a structured error
/// instead of a silently wrong slice. No semantic decoding happens here.
///
/// Nested attribute content that is already covered by the charge of the attribute
/// containing it is sliced with [`attribute_slice`] instead: it is the same
/// consistency check without a second charge.
#[allow(dead_code)]
pub fn attribute_content<'a>(
    bytes: &'a [u8],
    shell: &AttributeShell,
    budget: &mut Budget,
) -> Result<&'a [u8]> {
    budget.poll()?;
    let shell_length = attribute_shell_length(&shell.content_span)?;
    let content = attribute_slice(bytes, &shell.span, &shell.content_span)?;
    budget.charge(CountedBudgetDimension::AttributeBytes, shell_length)?;
    Ok(content)
}

/// Slices one attribute content out of the class bytes without billing it.
///
/// The header span and the content span must describe one `attribute_info`: the header
/// span covers the two indexes and the length, and the content span starts right after
/// them and is inside the class bytes. A shell that disagrees with itself or points
/// outside the class is a structured error instead of a silently wrong slice.
///
/// This is the non-billing entry point for content the caller already paid for — a
/// nested attribute inside a `Code` attribute, whose bytes are charged once as part of
/// the entry that contains it (see [`code_nested_attributes`]).
#[allow(dead_code)]
pub fn attribute_slice<'a>(
    bytes: &'a [u8],
    span: &ByteSpan,
    content_span: &ByteSpan,
) -> Result<&'a [u8]> {
    let shell_length = attribute_shell_length(content_span)?;
    if span.length != shell_length
        || span.start.checked_add(ATTRIBUTE_HEADER_LENGTH as u64) != Some(content_span.start)
    {
        return Err(Error::invalid_input(
            "classfile_invalid_attribute_span",
            "attribute shell span does not match its content span",
        ));
    }
    span_slice(bytes, content_span, "attribute content")
}

/// `6 + content length` of one attribute entry: the unit one pass over an
/// `attribute_info` is billed in.
pub(crate) fn attribute_shell_length(content_span: &ByteSpan) -> Result<u64> {
    content_span
        .length
        .checked_add(ATTRIBUTE_HEADER_LENGTH as u64)
        .ok_or_else(|| {
            Error::invalid_input("classfile_span_overflow", "attribute shell length overflow")
        })
}

/// Reads the simple class-level or member-level attributes of one shell list.
///
/// Only the recognised attributes below are read; every other attribute,
/// including `Code` and its nested `LineNumberTable`/`LocalVariableTable`/
/// `StackMapTable`, is left untouched. `MemberHeader::attributes` from
/// `class_facts` is the intended input for the member-level call.
#[allow(dead_code)]
pub fn attribute_facts(
    bytes: &[u8],
    shells: &[AttributeShell],
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<AttributeFacts> {
    let mut facts = AttributeFacts::default();
    let mut seen: Vec<&str> = Vec::new();
    for shell in shells {
        budget.poll()?;
        match shell.name.raw().0.as_slice() {
            b"Signature" => {
                ensure_unique(&mut seen, "Signature")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                let signature_index = reader.u16()?;
                reader.expect_end()?;
                facts.signature = Some(cp_utf8(pool, signature_index)?);
            }
            b"Exceptions" => {
                ensure_unique(&mut seen, "Exceptions")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                facts.exceptions = read_class_name_list(&mut reader, pool, budget)?;
                reader.expect_end()?;
            }
            b"InnerClasses" => {
                ensure_unique(&mut seen, "InnerClasses")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                let count = reader.u16()?;
                let mut inner_classes = Vec::new();
                for _ in 0..count {
                    budget.poll()?;
                    let class_index = reader.u16()?;
                    let outer_class_index = reader.u16()?;
                    let inner_name_index = reader.u16()?;
                    let access_flags = reader.u16()?;
                    let inner_name = if inner_name_index == 0 {
                        None
                    } else {
                        Some(cp_utf8(pool, inner_name_index)?)
                    };
                    inner_classes.push(InnerClassFacts {
                        class_index,
                        outer_class_index,
                        inner_name,
                        access_flags,
                    });
                }
                reader.expect_end()?;
                facts.inner_classes = inner_classes;
            }
            b"EnclosingMethod" => {
                ensure_unique(&mut seen, "EnclosingMethod")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                let class_index = reader.u16()?;
                let method_index = reader.u16()?;
                reader.expect_end()?;
                facts.enclosing_method = Some(EnclosingMethodFacts {
                    class_index,
                    method_index,
                });
            }
            b"NestHost" => {
                ensure_unique(&mut seen, "NestHost")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                let host_class_index = reader.u16()?;
                reader.expect_end()?;
                facts.nest_host = Some(cp_class_name(pool, host_class_index)?);
            }
            b"NestMembers" => {
                ensure_unique(&mut seen, "NestMembers")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                facts.nest_members = read_class_name_list(&mut reader, pool, budget)?;
                reader.expect_end()?;
            }
            b"PermittedSubclasses" => {
                ensure_unique(&mut seen, "PermittedSubclasses")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                facts.permitted_subclasses = read_class_name_list(&mut reader, pool, budget)?;
                reader.expect_end()?;
            }
            b"ConstantValue" => {
                ensure_unique(&mut seen, "ConstantValue")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                let constant_value_index = reader.u16()?;
                reader.expect_end()?;
                facts.constant_value = Some(CpIndexOf(constant_value_index));
            }
            b"Module" => {
                ensure_unique(&mut seen, "Module")?;
                let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
                facts.module = Some(read_module_facts(&mut reader, pool, budget)?);
                reader.expect_end()?;
            }
            _ => {}
        }
    }
    Ok(facts)
}

/// Reads one `BootstrapMethods` attribute.
///
/// The attribute is read on its own so the caller can decide when a deferred
/// bootstrap graph needs it; the class-level enumeration never reads it. Both
/// the bootstrap handle and every argument are validated as loadable constant
/// shapes, because a dynamic site that cannot reach a loadable constant is not a
/// fact a consumer may act on.
#[allow(dead_code)]
pub fn bootstrap_methods(
    bytes: &[u8],
    shell: &AttributeShell,
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<Vec<BootstrapMethodFacts>> {
    let mut reader = AttributeReader::new(attribute_content(bytes, shell, budget)?);
    let count = reader.u16()?;
    let mut methods = Vec::new();
    for _ in 0..count {
        budget.poll()?;
        let method_ref = reader.u16()?;
        match &cp_entry(pool, method_ref)?.kind {
            CpEntryKind::MethodHandle { .. } => {}
            _ => {
                return Err(cp_fact_tag_mismatch("CONSTANT_MethodHandle", method_ref));
            }
        }
        let argument_count = reader.u16()?;
        let mut arguments = Vec::new();
        for _ in 0..argument_count {
            let index = reader.u16()?;
            if !matches!(
                cp_entry(pool, index)?.kind,
                CpEntryKind::String { .. }
                    | CpEntryKind::Class { .. }
                    | CpEntryKind::Integer { .. }
                    | CpEntryKind::Long { .. }
                    | CpEntryKind::Float { .. }
                    | CpEntryKind::Double { .. }
                    | CpEntryKind::MethodHandle { .. }
                    | CpEntryKind::MethodType { .. }
                    | CpEntryKind::Dynamic { .. }
            ) {
                return Err(cp_fact_tag_mismatch("loadable constant", index));
            }
            arguments.push(index);
        }
        methods.push(BootstrapMethodFacts {
            method_ref,
            arguments,
        });
    }
    reader.expect_end()?;
    Ok(methods)
}

// ---------------------------------------------------------------------------
// Descriptor facts
// ---------------------------------------------------------------------------

/// Stable code of a descriptor that does not parse as its own production.
const DESCRIPTOR_CODE: &str = "query_descriptor_malformed";

/// Which descriptor production to accept (JVMS 4.3).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DescriptorKind {
    /// `FieldDescriptor`.
    Field,
    /// `MethodDescriptor`: parameters and result, `V` allowed.
    Method,
    /// `ReturnDescriptor`: a field type or `V` (annotation class literals).
    Return,
}

/// A descriptor one constant-pool entry carries, with the production it belongs to.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryDescriptor {
    /// Raw descriptor bytes exactly as the class file holds them.
    pub descriptor: JvmBytes,
    pub kind: DescriptorKind,
}

/// The descriptor one constant-pool entry carries, when it carries one at all.
///
/// JVMS 4.4.9/4.4.10: a `MethodType` holds a **method** descriptor, a `Dynamic` holds a
/// **field** descriptor, an `InvokeDynamic` holds a method descriptor, and a member
/// reference holds the descriptor of the member it names. A `MethodHandle` carries none
/// of its own — the entry its `reference_index` names does, and the caller resolves that
/// hop. Every other kind carries no descriptor, including a `Class` entry, whose type is
/// the entry itself rather than a descriptor's contents.
#[allow(dead_code)]
pub fn entry_descriptor(kind: &CpEntryKind) -> Option<EntryDescriptor> {
    let (descriptor, kind) = match kind {
        CpEntryKind::FieldRef { descriptor, .. } => (descriptor, DescriptorKind::Field),
        CpEntryKind::MethodRef { descriptor, .. }
        | CpEntryKind::InterfaceMethodRef { descriptor, .. } => {
            (descriptor, DescriptorKind::Method)
        }
        CpEntryKind::MethodType { descriptor, .. } => (descriptor, DescriptorKind::Method),
        CpEntryKind::Dynamic { descriptor, .. } => (descriptor, DescriptorKind::Field),
        CpEntryKind::InvokeDynamic { descriptor, .. } => (descriptor, DescriptorKind::Method),
        _ => return None,
    };
    Some(EntryDescriptor {
        descriptor: descriptor.clone(),
        kind,
    })
}

/// Object types a field, method or return descriptor names, in first-appearance order and
/// once per descriptor.
///
/// JVMS 4.3: a descriptor is built from base types, `L<internal name>;` and `[` arrays.
/// Only the object types are class references, and an array names its element type, so
/// `[[Ljava/lang/String;` mentions `java/lang/String`. A descriptor that does not parse
/// exactly is a structured error instead of a partial type set, so a caller can never
/// publish the types of a descriptor it did not read completely.
#[allow(dead_code)]
pub fn descriptor_types(descriptor: &[u8], kind: DescriptorKind) -> Result<Vec<JvmBytes>> {
    let mut reader = DescriptorReader::new(descriptor);
    let mut types = Vec::new();
    match kind {
        DescriptorKind::Field => descriptor_field_type(&mut reader, &mut types)?,
        DescriptorKind::Return => descriptor_return_type(&mut reader, &mut types)?,
        DescriptorKind::Method => {
            reader.expect_byte(b'(')?;
            while reader.peek() != Some(b')') {
                descriptor_field_type(&mut reader, &mut types)?;
            }
            reader.expect_byte(b')')?;
            descriptor_return_type(&mut reader, &mut types)?;
        }
    }
    reader.expect_end()?;
    Ok(types)
}

/// Adds an object type once per descriptor.
///
/// Descriptor productions and generic signatures both name a type once, in the order the
/// bytes write it, so they share this rule.
#[allow(dead_code)]
pub fn push_unique(types: &mut Vec<JvmBytes>, name: Vec<u8>) {
    if !types.iter().any(|existing| existing.0 == name) {
        types.push(JvmBytes(name));
    }
}

/// `ReturnDescriptor`: `V` or a field type.
fn descriptor_return_type(
    reader: &mut DescriptorReader<'_>,
    types: &mut Vec<JvmBytes>,
) -> Result<()> {
    if reader.peek() == Some(b'V') {
        reader.skip(1)?;
        return Ok(());
    }
    descriptor_field_type(reader, types)
}

/// One field type, recording its object type when it has one.
fn descriptor_field_type(
    reader: &mut DescriptorReader<'_>,
    types: &mut Vec<JvmBytes>,
) -> Result<()> {
    while reader.peek() == Some(b'[') {
        reader.skip(1)?;
    }
    match reader.peek() {
        Some(b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z') => reader.skip(1),
        Some(b'L') => {
            let name = reader.object_name()?;
            push_unique(types, name);
            Ok(())
        }
        _ => Err(reader.malformed("descriptor component is not a field type")),
    }
}

/// Bounded cursor over one descriptor.
///
/// The reader owns the same guarantees as the metadata consumer's region reader: every
/// read is bounds-checked, and a caller must reach [`DescriptorReader::expect_end`], so a
/// descriptor that does not end exactly at its end is an error instead of a truncated
/// type set.
struct DescriptorReader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> DescriptorReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip(&mut self, count: usize) -> Result<()> {
        let end = self
            .at
            .checked_add(count)
            .ok_or_else(|| self.malformed("region length overflow"))?;
        self.bytes
            .get(self.at..end)
            .ok_or_else(|| self.malformed("region ends before the structure does"))?;
        self.at = end;
        Ok(())
    }

    fn expect_byte(&mut self, expected: u8) -> Result<()> {
        let found = self
            .peek()
            .ok_or_else(|| self.malformed("region ends before the structure does"))?;
        if found == expected {
            self.at += 1;
            Ok(())
        } else {
            Err(self.malformed(&format!(
                "expected byte {:?} but found {:?}",
                char::from(expected),
                char::from(found)
            )))
        }
    }

    fn expect_end(&self) -> Result<()> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(self.malformed("structure does not end at the region end"))
        }
    }

    /// One descriptor object type name: `L <internal name> ;`.
    fn object_name(&mut self) -> Result<Vec<u8>> {
        self.expect_byte(b'L')?;
        let start = self.at;
        while let Some(byte) = self.peek() {
            if byte == b';' {
                break;
            }
            self.at += 1;
        }
        if start == self.at {
            return Err(self.malformed("object type name is empty"));
        }
        let name = self.bytes[start..self.at].to_vec();
        self.expect_byte(b';')?;
        Ok(name)
    }

    fn malformed(&self, message: &str) -> Error {
        Error::invalid_input(
            DESCRIPTOR_CODE,
            format!("{message} (at byte {} of the region)", self.at),
        )
    }
}

// ---------------------------------------------------------------------------
// Nested attributes
// ---------------------------------------------------------------------------

/// One nested `attribute_info`: inside a `Code` attribute or inside one `record_component_info`
/// (JVMS 4.7.30).
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedAttributeFact {
    /// `attribute_name_index` exactly as the declaration records it.
    pub name_index: u16,
    /// Raw name bytes of that index; a name is never text-decoded here.
    pub name: JvmBytes,
    /// Byte range of the whole nested `attribute_info`, header included.
    pub span: ByteSpan,
    /// Byte range of the nested attribute content.
    pub content_span: ByteSpan,
}

/// Reads the nested `attribute_info` list of one method's `Code` attribute (JVMS 4.7.3).
///
/// # Scope
///
/// A method body holds metadata of its own — `LineNumberTable`, `LocalVariableTable`,
/// `StackMapTable` and the type annotations a compiler moves inside the body — and a
/// consumer that answers for a standard position there cannot invent that structure from
/// the member header's shells. This is the reader fact that walks it:
/// `max_stack`, `max_locals`, `code_length` and the code array are **measured, never
/// decoded** (no instruction is interpreted, no `CodeBytes` is charged, and no graph is
/// built), then the exception table, the attribute count and each nested declaration.
/// The nested attributes are read only because this function is called at all: no other
/// reader fact touches them.
///
/// `shells` is the `MemberHeader::attributes` of the method, the same input
/// [`attribute_facts`] takes. A member that declares no `Code` yields an empty list: no
/// body is a normal declaration shape, not a failure. Two `Code` attributes are refused
/// with `classfile_duplicate_code_attribute`, because no single body is meant then.
///
/// # Billing
///
/// One `AttributeBytes` charge for the `Code` entry, `6 + content length`, exactly like
/// [`attribute_content`] and [`method_code_facts`] bill the same entry. The nested
/// content is inside that charge and is read through [`attribute_slice`], which bills
/// nothing, so walking a body's metadata costs one pass over the `Code` entry.
///
/// # Errors
///
/// * `classfile_duplicate_code_attribute` — the member declares more than one `Code`.
/// * `classfile_code_out_of_bounds` — the declared code array does not fit the content.
/// * `classfile_invalid_attribute_content` — the declared structure ends before the
///   content does (an exception table, an attribute count or a nested declaration that
///   the remaining bytes cannot hold), or the content has trailing bytes no declaration
///   covers.
///
/// Every nested `attribute_info` is reported by its own `attribute_name_index`, raw name
/// bytes and class-file spans, so a caller can slice both the content and the entry out
/// of the class bytes and check its own reading of them.
#[allow(dead_code)]
pub fn code_nested_attributes(
    bytes: &[u8],
    shells: &[AttributeShell],
    pool: &[CpEntryFacts],
    budget: &mut Budget,
) -> Result<Vec<NestedAttributeFact>> {
    let mut code_shells = shells
        .iter()
        .filter(|shell| shell.name.raw().0.as_slice() == b"Code");
    let shell = match (code_shells.next(), code_shells.next()) {
        // An abstract or native member has no body at all: nothing to read, and nothing
        // to report as damage.
        (None, _) => return Ok(Vec::new()),
        (Some(_), Some(_)) => {
            return Err(Error::invalid_input(
                "classfile_duplicate_code_attribute",
                "selected method has multiple Code attributes",
            ));
        }
        (Some(shell), None) => shell,
    };
    budget.poll()?;
    let content = attribute_slice(bytes, &shell.span, &shell.content_span)?;
    budget.charge(
        CountedBudgetDimension::AttributeBytes,
        attribute_shell_length(&shell.content_span)?,
    )?;
    let mut reader = AttributeReader::new(content);
    reader.skip(4)?; // max_stack, max_locals
    let code_length = usize::try_from(reader.u32()?).map_err(|_| {
        Error::invalid_input(
            "classfile_code_out_of_bounds",
            "code length does not fit a byte range",
        )
    })?;
    let code_end = reader.position.checked_add(code_length).ok_or_else(|| {
        Error::invalid_input("classfile_span_overflow", "code array end overflow")
    })?;
    if code_end > content.len() {
        return Err(Error::invalid_input(
            "classfile_code_out_of_bounds",
            "code array exceeds Code attribute content",
        ));
    }
    reader.skip(code_length)?;
    let handlers = usize::from(reader.u16()?);
    reader.skip(
        handlers
            .checked_mul(8)
            .ok_or_else(|| span_overflow("exception table length"))?,
    )?;
    let attributes = usize::from(reader.u16()?);
    let mut nested = Vec::new();
    for _ in 0..attributes {
        budget.poll()?;
        let start = entry_offset(&shell.content_span, reader.position)?;
        let name_index = reader.u16()?;
        let length = u64::from(reader.u32()?);
        let length_usize = usize::try_from(length).map_err(|_| {
            Error::invalid_input(
                "classfile_invalid_attribute_content",
                "nested attribute length does not fit a byte range",
            )
        })?;
        let content_start = entry_offset(&shell.content_span, reader.position)?;
        reader.skip(length_usize)?;
        let end = entry_offset(&shell.content_span, reader.position)?;
        nested.push(NestedAttributeFact {
            name_index,
            name: cp_utf8(pool, name_index)?,
            span: ByteSpan::new(start, end - start),
            content_span: ByteSpan::new(content_start, length),
        });
    }
    reader.expect_end()?;
    Ok(nested)
}

/// Class-file offset of one position inside an attribute content.
pub(crate) fn entry_offset(content_span: &ByteSpan, position: usize) -> Result<u64> {
    let position = u64::try_from(position).map_err(|_| {
        Error::invalid_input(
            "classfile_span_overflow",
            "attribute content position does not fit u64",
        )
    })?;
    content_span
        .start
        .checked_add(position)
        .ok_or_else(|| span_overflow("nested attribute offset"))
}

/// One method body decoded as reusable facts.
///
/// The instructions are the same facts `inspect_method_bytecode` returns for the same
/// method — same noak-backed cursor, same width, span and constant-pool index
/// derivation — because both go through the same internal decode path. What differs is
/// the contract around them: the caller already billed `ClassBytes` through
/// `class_facts`, `ResultItems` is never charged here, and a body that stopped before
/// the end of its `Code` attribute returns its reliable prefix together with
/// `execution` and `stopped_at` instead of a [`BytecodeInspection`].
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodCodeFacts {
    pub max_stack: u16,
    pub max_locals: u16,
    /// Class-file range of the instruction array: `code_length` bytes starting at the
    /// first opcode, so a BCI becomes a class offset by adding `code_span.start`.
    pub code_span: ByteSpan,
    pub instructions: Vec<InstructionFact>,
    /// Typed operands of [`Self::instructions`], in lockstep: `operands[i]` describes
    /// `instructions[i]`, and any stop returns the same reliable prefix of both.
    operands: Vec<InstructionOperands>,
    /// The whole exception table. Handlers are decoded before instructions, exactly
    /// like `inspect_method_bytecode`, so these are complete even when the instruction
    /// stream is the phase that stopped.
    pub exception_handlers: Vec<ExceptionHandlerFact>,
    /// Exception-table size the `Code` attribute declares.
    ///
    /// The same fact [`BytecodeInspection::exception_handler_count`] carries: with
    /// [`Self::stopped_at`] in [`BytecodeStopPhase::ExceptionHandlers`] the list above is a
    /// prefix of this count, and a consumer that publishes a coverage plane needs the total
    /// to name the range it did not read.
    pub exception_handler_count: u32,
    pub execution: ExecutionReport,
    pub stopped_at: Option<BytecodeStop>,
    /// The body's own debug names, read from the `LocalVariableTable` this `Code` attribute may
    /// declare (P3 3.1).
    ///
    /// The table is **inside** the `Code` entry this read already decoded and charged for, so reading
    /// it here is the same read: one slice of the bytes, one `AttributeBytes` charge, no second truth
    /// about one body. `MethodParameters` is deliberately not read: it is a *method-level* attribute
    /// whose content lies outside the `Code` entry, so decoding it here would charge bytes this read
    /// has not billed — that is a reader-contract decision with its own accounting, not something to
    /// slip in beside a body read.
    debug: LocalDebugTable,
}

/// The debug names one method body's `Code` attribute states (P3 3.1).
///
/// Three states, because "no name" and "no table" are different facts: a body compiled without debug
/// metadata declares nothing, and a read that did not produce a table (its content does not decode,
/// or the body stopped before the walk) states none. Neither is a failure of the body — the
/// instructions decoded — and the presentation names the slots by their ordinals in both cases (A10).
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalDebugTable {
    /// The `Code` attribute declares no `LocalVariableTable`: the walk completed and found none.
    Absent,
    /// The table as the one read decoded it, in declaration order.
    Read(Vec<LocalDebugName>),
    /// The walk produced no table: a declared table whose content the bytes cannot hold, or a body
    /// read that stopped before it walked the nested attributes. No name is stated for either.
    Unstated,
}

/// One record of a `LocalVariableTable` (JVMS 4.7.13).
///
/// The record names one slot **over a range of bytecode**: a compiler that reuses a slot for two
/// variables in disjoint scopes states two records for it, which is why the range is kept — it is
/// what tells one reused slot from one variable named twice.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDebugName {
    /// The local slot the record names.
    pub slot: u16,
    /// The first bytecode index the record covers.
    pub start_bci: u32,
    /// One past the last bytecode index the record covers.
    pub end_bci: u32,
    /// The name, exactly as the class file's own constant spells it (JVM bytes, never text-decoded
    /// here).
    pub name: JvmBytes,
}

impl LocalDebugName {
    /// The name, as the text a presentation would write when it is one.
    ///
    /// A name that is not UTF-8 is not text this layer can spell: the caller decides what to do with
    /// it (a name no Java identifier grammar accepts is aliased, never dropped — see
    /// `jarde-java`'s naming rules).
    pub fn name_lossy(&self) -> String {
        String::from_utf8_lossy(&self.name.0).into_owned()
    }
}

impl MethodCodeFacts {
    /// Typed operands of [`Self::instructions`], in lockstep with them.
    ///
    /// `operands()[i]` describes `instructions[i]`; the two lists are decoded together and a
    /// stop returns the same reliable prefix of both. The field itself stays private so that no
    /// caller can build the two lists apart: the length agreement is the invariant the decode
    /// path asserts after every body it reads, and a public field would let a consumer assemble
    /// facts the reader never produced.
    pub fn operands(&self) -> &[InstructionOperands] {
        &self.operands
    }

    /// The body's own debug names, as the same read decoded them (P3 3.1).
    ///
    /// The `LocalVariableTable` sits inside the `Code` entry this read already charged for, so this
    /// is not a second read: the bytes are the ones [`Self::code_span`] and the instruction array
    /// came from, and no additional dimension is billed for them.
    pub fn debug(&self) -> &LocalDebugTable {
        &self.debug
    }

    /// Assembles a body from parts a test names, without a decode; test support only.
    ///
    /// This is the one way to build facts the decode path did not produce, and it exists for the
    /// same reason [`Self::operands`] is an accessor instead of a field: the fixtures of a
    /// dependent crate's analysis tests (CFG, call contexts) need a body, and their `cfg(test)`
    /// cannot see this crate. The instruction/operand pair list is the parameter, not two
    /// separate lists, so the lockstep survives even on this path.
    ///
    /// It is compiled only under `test-support`, which no normal dependency enables: a
    /// production build cannot fabricate a `MethodCodeFacts`, and `--all-features` builds (CI)
    /// are the only ones that see it at all.
    #[cfg(any(test, feature = "test-support"))]
    #[expect(
        clippy::too_many_arguments,
        reason = "a test-only assembler mirrors the fields of the facts it builds; bundling \
                  them into a second public struct would grow the test-support API for no gain"
    )]
    pub fn from_parts(
        max_stack: u16,
        max_locals: u16,
        code_span: ByteSpan,
        code: Vec<(InstructionFact, InstructionOperands)>,
        exception_handlers: Vec<ExceptionHandlerFact>,
        exception_handler_count: u32,
        execution: ExecutionReport,
        stopped_at: Option<BytecodeStop>,
        // The debug names the assembled body stands for, if the fixture states any.
        debug: LocalDebugTable,
    ) -> Self {
        let (instructions, operands) = code.into_iter().unzip();
        Self {
            max_stack,
            max_locals,
            code_span,
            instructions,
            operands,
            exception_handlers,
            exception_handler_count,
            execution,
            stopped_at,
            debug,
        }
    }
}

/// Typed operands of one instruction, taken from the same noak event and the same
/// instruction byte range that produced the matching [`InstructionFact`].
///
/// The mapping is total: a field is `None` exactly when the instruction has no operand
/// of that kind, so an instruction without operands (`pop`, `return`, `dup`, …) is
/// all-`None` rather than absent. Implicit operands are facts too — the `_0`..`_3`
/// load/store forms name their local, the `iconst`/`lconst`/`fconst`/`dconst` forms
/// name their constant, and `wide` marks the widened form — so 3.x never has to
/// re-derive them from the raw opcode or from rendered text.
///
/// [`InstructionOperands::default`] is that all-`None` shape, and its
/// [`Self::effective_opcode`] is `0x00`: a fixture that builds operands from `Default`
/// must set the effective opcode it stands for, while the reader always fills it from the
/// event it decoded.
///
/// [`Self::atype`], [`Self::dimensions`] and [`Self::interface_count`] are payload facts of
/// three specific opcodes, deliberately not folded into [`Self::immediate`]: an element
/// type code is not a value an instruction pushes, a dimension count is not a constant pool
/// entry, and an `invokeinterface` count is a claim the verifier has to reconcile with the
/// descriptor instead of trusting.
///
/// This is crate-private reader data: it does not appear in [`BytecodeInspection`] and
/// it is covered by the `CodeBytes` charge of the instruction it belongs to, with no
/// extra dimension and no second charge.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstructionOperands {
    /// `bipush`/`sipush`, the `ldc` family, and the constant forms encoded in the opcode.
    pub immediate: Option<ImmediateValue>,
    /// The local of a load, store, `iinc` or `ret`, including the implicit forms.
    pub local: Option<LocalOperand>,
    /// Signed increment of `iinc` and `wide iinc`; `None` for every other opcode.
    pub increment: Option<i32>,
    /// Same value as [`InstructionFact::constant_pool_index`] of the same instruction.
    pub constant_pool_index: Option<u16>,
    /// Relative offset exactly as encoded by a branch, `goto_w`, `jsr` or `jsr_w`:
    /// relative to the BCI of the instruction itself.
    pub branch_offset: Option<i32>,
    /// Payload of `tableswitch`/`lookupswitch`; `None` for every other opcode.
    pub switch: Option<SwitchOperands>,
    /// The opcode this instruction *is*: the opcode a `wide` form wraps (`wide iload` is
    /// `0x15`, `wide iinc` is `0x84`), and the raw opcode for every other instruction.
    ///
    /// This is the opcode classifications read — block ends, transfers, `may_throw`, local
    /// read/write direction, stack deltas and the legacy dialect — because `0xc4` is a prefix
    /// (JVMS 6.5), not an instruction: it decides none of them on its own. The raw opcode stays
    /// what it always was in [`InstructionFact::opcode`], together with the width and the span.
    pub effective_opcode: u8,
    /// `newarray`'s element type code (JVMS `atype`, 4..=11); `None` for every other opcode.
    /// The reader records it as encoded: an out-of-range code never reaches here, because the
    /// instruction does not decode at all.
    pub atype: Option<u8>,
    /// `multianewarray`'s dimension count exactly as encoded; `None` for every other opcode.
    /// Whether the count is a legal dimension is the verifier's question (4.x), not a reason
    /// for the reader to drop the fact or to fail the instruction.
    pub dimensions: Option<u8>,
    /// `invokeinterface`'s encoded `count`; `None` for every other opcode.
    ///
    /// A validation *fact*, not a trust source: the count is kept as encoded — including a
    /// count that disagrees with the descriptor's argument slots — so 5.1 can reconcile it
    /// against this project's own descriptor derivation.
    pub interface_count: Option<u8>,
}

/// The literal one instruction pushes directly.
///
/// Floating point keeps its bit pattern, so a NaN payload or a negative zero survives
/// the fact. Reference constants are not literals here: `ldc` of a `String`, `Class`,
/// `MethodHandle`, `MethodType` or dynamic constant keeps its
/// [`InstructionOperands::constant_pool_index`] for the resolver instead.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImmediateValue {
    Int(i32),
    Long(i64),
    Float(u32),
    Double(u64),
}

/// One local variable slot named by an instruction.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalOperand {
    /// Local slot index in the method's frame.
    pub index: u16,
    /// `true` for the `wide` prefixed form, `false` for the short form and for the
    /// `_0`..`_3` forms, whose index is encoded in the opcode.
    pub wide: bool,
}

/// Payload of one `tableswitch`/`lookupswitch`.
///
/// The per-entry offsets are read from the instruction's own byte range instead of
/// iterating noak's pair iterators: `TablePairs` walks an `i32` key toward a hostile
/// `high`, which is the upper-bound risk the reader adapter deliberately keeps out.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwitchOperands {
    Table {
        default_offset: i32,
        low: i32,
        high: i32,
        /// One relative offset per key, in payload order (`low` first).
        offsets: Vec<i32>,
    },
    Lookup {
        default_offset: i32,
        /// `(match, offset)` in payload order.
        pairs: Vec<(i32, i32)>,
    },
}

/// One control-flow target of one method, validated but not yet a graph edge.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlFlowTarget {
    /// The instruction this row belongs to: the branch or switch instruction that
    /// encodes the offset, and the handler entry for [`ControlFlowTargetKind::Handler`]
    /// — an exception table record is not an instruction, and its `ordinal` in the
    /// kind identifies the record.
    pub instruction_bci: u32,
    pub kind: ControlFlowTargetKind,
    /// Absolute BCI the target must land on: an instruction start inside the code array.
    pub target_bci: u32,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlFlowTargetKind {
    /// A relative branch offset, kept exactly as encoded.
    Branch {
        offset: i32,
    },
    SwitchDefault,
    /// `index` is the position in the switch payload; `key` is `low + index` for
    /// `tableswitch` and the encoded `match` for `lookupswitch`.
    SwitchCase {
        index: u32,
        key: i32,
    },
    /// Exception table record `ordinal`.
    Handler {
        ordinal: u32,
    },
}

impl MethodCodeFacts {
    /// Every control-flow target of this method, validated against the instruction
    /// starts of the same facts. No CFG is built: these are the evidenced targets 3.x
    /// consumes when it builds one.
    ///
    /// Rows come out in instruction order — a switch emits its default before its
    /// cases — followed by the exception table in ordinal order. Validation is per row
    /// and stops at the first failure, so the error always carries the original BCI:
    ///
    /// * a relative offset that leaves the code address space is
    ///   `classfile_code_span_overflow`, and one that lands before BCI 0 is
    ///   `classfile_instruction_target_out_of_bounds`;
    /// * a target at or past `code_length` is `classfile_instruction_target_out_of_bounds`;
    /// * a target that is not an instruction start is `classfile_instruction_invalid_target`;
    /// * a protected range that ends past `code_length`, is inverted, or is empty is
    ///   `classfile_exception_range_invalid`, while a range endpoint that is not an
    ///   instruction start is `classfile_instruction_invalid_target`; `end == code_length`
    ///   is the legal half-open end of the code array and needs no instruction start.
    ///
    /// A body whose instruction stream stopped early is validated on the facts it has:
    /// this is a **reliable but incomplete** view. It never accepts an illegal target,
    /// but a legal target inside the unread suffix reads as invalid. A caller MUST check
    /// `execution`/`stopped_at` before reporting this `Err` as a corrupt method; 3.x
    /// tightens the type-level guarantee only if it needs one.
    #[allow(dead_code)]
    pub fn control_flow_targets(&self) -> Result<Vec<ControlFlowTarget>> {
        debug_assert_eq!(
            self.instructions.len(),
            self.operands.len(),
            "operand facts are produced in lockstep with instructions"
        );
        let code_length = u32::try_from(self.code_span.length).map_err(|_| {
            Error::invalid_input(
                "classfile_code_span_overflow",
                "code length does not fit u32",
            )
        })?;
        let starts = InstructionStarts::new(&self.instructions);
        let mut targets = Vec::new();
        for (fact, operands) in self.instructions.iter().zip(self.operands.iter()) {
            if let Some(offset) = operands.branch_offset {
                let target = relative_target_bci(fact.bci, offset)?;
                check_target(
                    &starts,
                    code_length,
                    target,
                    &format!("branch from BCI {} with offset {offset}", fact.bci),
                )?;
                targets.push(ControlFlowTarget {
                    instruction_bci: fact.bci,
                    kind: ControlFlowTargetKind::Branch { offset },
                    target_bci: target,
                });
            }
            match &operands.switch {
                None => {}
                Some(SwitchOperands::Table {
                    default_offset,
                    low,
                    offsets,
                    ..
                }) => {
                    let default = relative_target_bci(fact.bci, *default_offset)?;
                    check_target(
                        &starts,
                        code_length,
                        default,
                        &format!("switch default from BCI {}", fact.bci),
                    )?;
                    targets.push(ControlFlowTarget {
                        instruction_bci: fact.bci,
                        kind: ControlFlowTargetKind::SwitchDefault,
                        target_bci: default,
                    });
                    for (index, offset) in (0u32..).zip(offsets.iter().copied()) {
                        let key = table_key(*low, index)?;
                        let target = relative_target_bci(fact.bci, offset)?;
                        check_target(
                            &starts,
                            code_length,
                            target,
                            &format!("switch case {index} from BCI {}", fact.bci),
                        )?;
                        targets.push(ControlFlowTarget {
                            instruction_bci: fact.bci,
                            kind: ControlFlowTargetKind::SwitchCase { index, key },
                            target_bci: target,
                        });
                    }
                }
                Some(SwitchOperands::Lookup {
                    default_offset,
                    pairs,
                }) => {
                    let default = relative_target_bci(fact.bci, *default_offset)?;
                    check_target(
                        &starts,
                        code_length,
                        default,
                        &format!("switch default from BCI {}", fact.bci),
                    )?;
                    targets.push(ControlFlowTarget {
                        instruction_bci: fact.bci,
                        kind: ControlFlowTargetKind::SwitchDefault,
                        target_bci: default,
                    });
                    for (index, (key, offset)) in (0u32..).zip(pairs.iter().copied()) {
                        let target = relative_target_bci(fact.bci, offset)?;
                        check_target(
                            &starts,
                            code_length,
                            target,
                            &format!("switch case {index} from BCI {}", fact.bci),
                        )?;
                        targets.push(ControlFlowTarget {
                            instruction_bci: fact.bci,
                            kind: ControlFlowTargetKind::SwitchCase { index, key },
                            target_bci: target,
                        });
                    }
                }
            }
        }
        for handler in &self.exception_handlers {
            check_protected_range(&starts, code_length, handler)?;
            targets.push(ControlFlowTarget {
                instruction_bci: handler.handler_bci,
                kind: ControlFlowTargetKind::Handler {
                    ordinal: handler.ordinal,
                },
                target_bci: handler.handler_bci,
            });
        }
        Ok(targets)
    }
}

/// Validates one exception table record: a non-empty half-open protected range of
/// instruction boundaries, and an entry that is a target like any other.
///
/// JVMS 4.7.3 requires `start_pc < end_pc <= code_length`; the endpoints are also
/// instruction boundaries, because an exception region covers instructions and not
/// bytes inside operands. A range that ends after the code or is inverted or empty is
/// a range-relation error (`classfile_exception_range_invalid`); an endpoint that is
/// not a boundary is a target error (`classfile_instruction_invalid_target`), so a
/// caller can tell "this record contradicts the code array" from "this record does not
/// line up with the instruction starts". Only `end_pc == code_length` is legal without
/// being an instruction start: it is the half-open end of the code array.
fn check_protected_range(
    starts: &InstructionStarts<'_>,
    code_length: u32,
    handler: &ExceptionHandlerFact,
) -> Result<()> {
    let ordinal = handler.ordinal;
    if handler.end_bci > code_length {
        return Err(Error::invalid_input(
            "classfile_exception_range_invalid",
            format!(
                "exception handler {ordinal} protected range {}..{} ends past code_length {code_length}",
                handler.start_bci, handler.end_bci
            ),
        ));
    }
    if handler.start_bci > handler.end_bci {
        return Err(Error::invalid_input(
            "classfile_exception_range_invalid",
            format!(
                "exception handler {ordinal} protected range {}..{} starts after its end",
                handler.start_bci, handler.end_bci
            ),
        ));
    }
    if handler.start_bci == handler.end_bci {
        return Err(Error::invalid_input(
            "classfile_exception_range_invalid",
            format!(
                "exception handler {ordinal} protected range at BCI {} is empty",
                handler.start_bci
            ),
        ));
    }
    // `start < end <= code_length` holds here, so both endpoints are inside the code
    // array and can be compared against the instruction starts.
    if !starts.contains(handler.start_bci) {
        return Err(Error::invalid_input(
            "classfile_instruction_invalid_target",
            format!(
                "exception handler {ordinal} protected range start {} is not an instruction start",
                handler.start_bci
            ),
        ));
    }
    if handler.end_bci != code_length && !starts.contains(handler.end_bci) {
        return Err(Error::invalid_input(
            "classfile_instruction_invalid_target",
            format!(
                "exception handler {ordinal} protected range end {} is neither an instruction start nor code_length {code_length}",
                handler.end_bci
            ),
        ));
    }
    check_target(
        starts,
        code_length,
        handler.handler_bci,
        &format!("exception handler {ordinal} entry"),
    )
}

/// The instruction-start set of one method body.
///
/// The facts are ordered by BCI and non-overlapping, so one binary search decides
/// whether a target is an instruction boundary or lands inside an operand.
struct InstructionStarts<'a> {
    instructions: &'a [InstructionFact],
}

impl<'a> InstructionStarts<'a> {
    const fn new(instructions: &'a [InstructionFact]) -> Self {
        Self { instructions }
    }

    fn contains(&self, bci: u32) -> bool {
        self.instructions
            .binary_search_by_key(&bci, |fact| fact.bci)
            .is_ok()
    }
}

/// Validates one absolute target against the code array and the start set.
fn check_target(
    starts: &InstructionStarts<'_>,
    code_length: u32,
    target: u32,
    label: &str,
) -> Result<()> {
    if target >= code_length {
        return Err(Error::invalid_input(
            "classfile_instruction_target_out_of_bounds",
            format!("{label} target {target} is outside 0..{code_length}"),
        ));
    }
    if !starts.contains(target) {
        return Err(Error::invalid_input(
            "classfile_instruction_invalid_target",
            format!("{label} target {target} is not an instruction start"),
        ));
    }
    Ok(())
}

/// Absolute BCI of one encoded relative offset, checked against the unsigned address
/// space in both directions.
fn relative_target_bci(instruction_bci: u32, offset: i32) -> Result<u32> {
    if offset >= 0 {
        instruction_bci
            .checked_add(offset.unsigned_abs())
            .ok_or_else(|| {
                Error::invalid_input(
                    "classfile_code_span_overflow",
                    format!(
                        "relative offset {offset} at BCI {instruction_bci} overflows the code address space"
                    ),
                )
            })
    } else {
        instruction_bci
            .checked_sub(offset.unsigned_abs())
            .ok_or_else(|| {
                Error::invalid_input(
                    "classfile_instruction_target_out_of_bounds",
                    format!(
                        "relative offset {offset} at BCI {instruction_bci} targets a BCI before 0"
                    ),
                )
            })
    }
}

/// `tableswitch` key of one payload position: `low + index`, checked.
fn table_key(low: i32, index: u32) -> Result<i32> {
    let key = i64::from(low) + i64::from(index);
    i32::try_from(key).map_err(|_| {
        Error::invalid_input(
            "classfile_instruction_invalid_range",
            format!("tableswitch key {key} at index {index} does not fit i32"),
        )
    })
}

/// Padding before the first operand field of a `tableswitch`/`lookupswitch`, relative
/// to the instruction's own BCI: its four-byte fields start at the next four-byte
/// boundary after the opcode byte.
fn switch_padding(bci: u32) -> u32 {
    (4 - ((bci + 1) & 3)) & 3
}

/// The `wide` prefix (JVMS 6.5): a modifier of the instruction that follows it, not an
/// instruction of its own.
const OPCODE_WIDE: u8 = 0xc4;

/// The `atype` byte of a `newarray`, from noak's own element type.
///
/// The enum's declaration order is not the encoded one, so the code is mapped explicitly
/// instead of cast: `boolean` is `4`, `long` is `11` (JVMS 6.5 `newarray`).
fn array_type_code(atype: ArrayType) -> u8 {
    match atype {
        ArrayType::Boolean => 4,
        ArrayType::Char => 5,
        ArrayType::Float => 6,
        ArrayType::Double => 7,
        ArrayType::Byte => 8,
        ArrayType::Short => 9,
        ArrayType::Int => 10,
        ArrayType::Long => 11,
    }
}

/// The opcode a decoded instruction really is: the opcode a `wide` form wraps, and the raw
/// opcode of the instruction itself for every other event.
///
/// JVMS 6.5 lets `wide` wrap exactly twelve opcodes, all of which noak decodes into their own
/// event variants, so the mapping is total over the events that carry the prefix; the debug
/// assertion pins that totality if a future noak adds a wrapping this build does not know.
fn effective_opcode(opcode: u8, instruction: &RawInstruction<'_>) -> u8 {
    let effective = match instruction {
        RawInstruction::ILoadW { .. } => 0x15,
        RawInstruction::LLoadW { .. } => 0x16,
        RawInstruction::FLoadW { .. } => 0x17,
        RawInstruction::DLoadW { .. } => 0x18,
        RawInstruction::ALoadW { .. } => 0x19,
        RawInstruction::IStoreW { .. } => 0x36,
        RawInstruction::LStoreW { .. } => 0x37,
        RawInstruction::FStoreW { .. } => 0x38,
        RawInstruction::DStoreW { .. } => 0x39,
        RawInstruction::AStoreW { .. } => 0x3a,
        RawInstruction::IIncW { .. } => 0x84,
        RawInstruction::RetW { .. } => 0xa9,
        _ => opcode,
    };
    debug_assert!(
        opcode != OPCODE_WIDE || effective != opcode,
        "every `wide` event must name the opcode it wraps"
    );
    effective
}

/// Typed operands of one instruction, from the same noak event that produced the
/// matching [`InstructionFact`] (`instruction`) and from the same instruction byte
/// range (`bytes`, exactly the fact's `width`).
///
/// Nothing here walks the code array again or recovers meaning from rendered text: the
/// scalars come from the decoded event and only the switch entry lists, which noak
/// exposes through iterators this adapter does not use, are read from the recorded
/// bytes with checked offsets. Either way the operand region is the region the width
/// already charged as `CodeBytes`.
fn instruction_operands(
    opcode: u8,
    bci: u32,
    instruction: &RawInstruction<'_>,
    bytes: &[u8],
    pool: &noak::reader::cpool::ConstantPool<'_>,
) -> Result<InstructionOperands> {
    let mut operands = InstructionOperands {
        immediate: None,
        local: None,
        increment: None,
        // The same helper and the same slice `InstructionFact` uses, so the two facts
        // cannot disagree.
        constant_pool_index: constant_pool_index(opcode, bytes),
        branch_offset: None,
        switch: None,
        effective_opcode: effective_opcode(opcode, instruction),
        atype: None,
        dimensions: None,
        interface_count: None,
    };
    match instruction {
        // Constants encoded in the opcode itself.
        RawInstruction::IConstM1 => operands.immediate = Some(ImmediateValue::Int(-1)),
        RawInstruction::IConst0 => operands.immediate = Some(ImmediateValue::Int(0)),
        RawInstruction::IConst1 => operands.immediate = Some(ImmediateValue::Int(1)),
        RawInstruction::IConst2 => operands.immediate = Some(ImmediateValue::Int(2)),
        RawInstruction::IConst3 => operands.immediate = Some(ImmediateValue::Int(3)),
        RawInstruction::IConst4 => operands.immediate = Some(ImmediateValue::Int(4)),
        RawInstruction::IConst5 => operands.immediate = Some(ImmediateValue::Int(5)),
        RawInstruction::LConst0 => operands.immediate = Some(ImmediateValue::Long(0)),
        RawInstruction::LConst1 => operands.immediate = Some(ImmediateValue::Long(1)),
        RawInstruction::FConst0 => operands.immediate = Some(ImmediateValue::Float(0f32.to_bits())),
        RawInstruction::FConst1 => operands.immediate = Some(ImmediateValue::Float(1f32.to_bits())),
        RawInstruction::FConst2 => operands.immediate = Some(ImmediateValue::Float(2f32.to_bits())),
        RawInstruction::DConst0 => {
            operands.immediate = Some(ImmediateValue::Double(0f64.to_bits()))
        }
        RawInstruction::DConst1 => {
            operands.immediate = Some(ImmediateValue::Double(1f64.to_bits()))
        }
        // Constants encoded as an operand byte or as a constant-pool index.
        RawInstruction::BIPush { value } => {
            operands.immediate = Some(ImmediateValue::Int(i32::from(*value)));
        }
        RawInstruction::SIPush { value } => {
            operands.immediate = Some(ImmediateValue::Int(i32::from(*value)));
        }
        RawInstruction::LdC { .. } | RawInstruction::LdCW { .. } | RawInstruction::LdC2W { .. } => {
            operands.immediate = operands
                .constant_pool_index
                .and_then(|index| pool_literal(pool, index));
        }
        // Local variable operands named by an operand byte.
        RawInstruction::ALoad { index } => operands.local = Some(short_local(*index)),
        RawInstruction::ALoadW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::AStore { index } => operands.local = Some(short_local(*index)),
        RawInstruction::AStoreW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::ILoad { index } => operands.local = Some(short_local(*index)),
        RawInstruction::ILoadW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::IStore { index } => operands.local = Some(short_local(*index)),
        RawInstruction::IStoreW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::LLoad { index } => operands.local = Some(short_local(*index)),
        RawInstruction::LLoadW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::LStore { index } => operands.local = Some(short_local(*index)),
        RawInstruction::LStoreW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::FLoad { index } => operands.local = Some(short_local(*index)),
        RawInstruction::FLoadW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::FStore { index } => operands.local = Some(short_local(*index)),
        RawInstruction::FStoreW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::DLoad { index } => operands.local = Some(short_local(*index)),
        RawInstruction::DLoadW { index } => operands.local = Some(wide_local(*index)),
        RawInstruction::DStore { index } => operands.local = Some(short_local(*index)),
        RawInstruction::DStoreW { index } => operands.local = Some(wide_local(*index)),
        // `iinc` names a local and carries a signed increment.
        RawInstruction::IInc { index, value } => {
            operands.local = Some(short_local(*index));
            operands.increment = Some(i32::from(*value));
        }
        RawInstruction::IIncW { index, value } => {
            operands.local = Some(wide_local(*index));
            operands.increment = Some(i32::from(*value));
        }
        // `ret` names the local holding the return address.
        RawInstruction::Ret { index } => operands.local = Some(short_local(*index)),
        RawInstruction::RetW { index } => operands.local = Some(wide_local(*index)),
        // Array creation: the element type code and the dimension count are payloads of their
        // own kind, taken from the event. Neither is an immediate, and neither is validated
        // here: an element type code outside `4..=11` never reaches this adapter (the
        // instruction does not decode), and a dimension count of zero is a fact 4.x judges.
        RawInstruction::NewArray { atype } => operands.atype = Some(array_type_code(*atype)),
        RawInstruction::MultiANewArray { dimensions, .. } => {
            operands.dimensions = Some(*dimensions);
        }
        // The encoded `count` of an interface call: the third payload byte the event carries.
        RawInstruction::InvokeInterface { count, .. } => {
            operands.interface_count = Some(*count);
        }
        // Relative branch offsets, in the width the opcode encodes.
        RawInstruction::Goto { offset }
        | RawInstruction::JSr { offset }
        | RawInstruction::IfACmpEq { offset }
        | RawInstruction::IfACmpNe { offset }
        | RawInstruction::IfICmpEq { offset }
        | RawInstruction::IfICmpNe { offset }
        | RawInstruction::IfICmpLt { offset }
        | RawInstruction::IfICmpGe { offset }
        | RawInstruction::IfICmpGt { offset }
        | RawInstruction::IfICmpLe { offset }
        | RawInstruction::IfEq { offset }
        | RawInstruction::IfNe { offset }
        | RawInstruction::IfLt { offset }
        | RawInstruction::IfGe { offset }
        | RawInstruction::IfGt { offset }
        | RawInstruction::IfLe { offset }
        | RawInstruction::IfNull { offset }
        | RawInstruction::IfNonNull { offset } => {
            operands.branch_offset = Some(i32::from(*offset));
        }
        RawInstruction::GotoW { offset } | RawInstruction::JSrW { offset } => {
            operands.branch_offset = Some(*offset);
        }
        // Switch payloads: scalars from the event, entry lists from the bytes.
        RawInstruction::TableSwitch(table) => {
            operands.switch = Some(table_switch_operands(
                bci,
                bytes,
                table.default_offset(),
                table.low(),
                table.high(),
            )?);
        }
        RawInstruction::LookupSwitch(lookup) => {
            let pairs = u32::try_from(lookup.pairs().count()).map_err(|_| {
                Error::invalid_input(
                    "classfile_instruction_width_overflow",
                    "lookupswitch pair count overflow",
                )
            })?;
            operands.switch = Some(lookup_switch_operands(
                bci,
                bytes,
                lookup.default_offset(),
                pairs,
            )?);
        }
        _ => {}
    }
    // `xload_0`..`xstore_3` name their local in the opcode: each of the ten families is
    // a block of four opcodes (JVMS 6.2/6.4), so the index is the offset in the block.
    match opcode {
        0x1a..=0x2d => {
            operands.local = Some(LocalOperand {
                index: u16::from((opcode - 0x1a) % 4),
                wide: false,
            })
        }
        0x3b..=0x4e => {
            operands.local = Some(LocalOperand {
                index: u16::from((opcode - 0x3b) % 4),
                wide: false,
            })
        }
        _ => {}
    }
    Ok(operands)
}

/// A short-form local operand: the index fits the byte the opcode encodes.
fn short_local(index: u8) -> LocalOperand {
    LocalOperand {
        index: u16::from(index),
        wide: false,
    }
}

/// A `wide` local operand.
fn wide_local(index: u16) -> LocalOperand {
    LocalOperand { index, wide: true }
}

/// The literal one `ldc`-family index names, when the pool entry is one of the four
/// literal kinds.
///
/// An index the pool does not hold, an entry of another tag, and the reserved slot of a
/// long or double all mean "no literal here": the index itself is the recorded fact,
/// and rejecting the instruction would fail classes the public reader accepts. Resolving
/// those constants is the resolver's job.
fn pool_literal(
    pool: &noak::reader::cpool::ConstantPool<'_>,
    index: u16,
) -> Option<ImmediateValue> {
    use noak::reader::cpool::{Index, Item};
    let at = Index::<Item<'_>>::new(index).ok()?;
    match pool.get(at).ok()? {
        Item::Integer(value) => Some(ImmediateValue::Int(value.value)),
        Item::Long(value) => Some(ImmediateValue::Long(value.value)),
        Item::Float(value) => Some(ImmediateValue::Float(value.value.to_bits())),
        Item::Double(value) => Some(ImmediateValue::Double(value.value.to_bits())),
        _ => None,
    }
}

/// `tableswitch` operands: `default_offset`, `low` and `high` come from the event that
/// also produced the width, and the entry offsets are read from the recorded bytes.
fn table_switch_operands(
    bci: u32,
    bytes: &[u8],
    default_offset: i32,
    low: i32,
    high: i32,
) -> Result<SwitchOperands> {
    let count = i64::from(high)
        .checked_sub(i64::from(low))
        .and_then(|value| value.checked_add(1))
        .filter(|count| *count > 0)
        .ok_or_else(|| {
            Error::invalid_input(
                "classfile_instruction_invalid_range",
                format!("tableswitch at BCI {bci} has a high below its low"),
            )
        })?;
    let count = usize::try_from(count).map_err(|_| {
        Error::invalid_input(
            "classfile_instruction_width_overflow",
            format!("tableswitch entry count at BCI {bci} does not fit usize"),
        )
    })?;
    let entries = switch_entries_start(bci, bytes, 12, count, 4, "tableswitch")?;
    let mut offsets = Vec::with_capacity(count);
    for index in 0..count {
        offsets.push(switch_i32(bytes, bci, entries + index * 4)?);
    }
    Ok(SwitchOperands::Table {
        default_offset,
        low,
        high,
        offsets,
    })
}

/// `lookupswitch` operands: `default_offset` and the pair count come from the event
/// that also produced the width, and the pairs are read from the recorded bytes.
fn lookup_switch_operands(
    bci: u32,
    bytes: &[u8],
    default_offset: i32,
    pairs: u32,
) -> Result<SwitchOperands> {
    let count = usize::try_from(pairs).map_err(|_| {
        Error::invalid_input(
            "classfile_instruction_width_overflow",
            format!("lookupswitch pair count at BCI {bci} does not fit usize"),
        )
    })?;
    let entries = switch_entries_start(bci, bytes, 8, count, 8, "lookupswitch")?;
    let mut collected = Vec::with_capacity(count);
    for index in 0..count {
        let key = switch_i32(bytes, bci, entries + index * 8)?;
        let offset = switch_i32(bytes, bci, entries + index * 8 + 4)?;
        collected.push((key, offset));
    }
    Ok(SwitchOperands::Lookup {
        default_offset,
        pairs: collected,
    })
}

/// Start offset of a switch entry list inside the instruction bytes, after checking
/// that the recorded region is exactly the shape the decoded counts describe.
///
/// The width of the instruction came from the same counts, so a mismatch means the
/// bytes and the event disagree: no operand facts are published for that instruction.
fn switch_entries_start(
    bci: u32,
    bytes: &[u8],
    scalars: usize,
    count: usize,
    unit: usize,
    name: &str,
) -> Result<usize> {
    let start = 1usize
        .checked_add(usize::try_from(switch_padding(bci)).map_err(|_| {
            Error::invalid_input(
                "classfile_code_span_overflow",
                format!("switch padding at BCI {bci} does not fit usize"),
            )
        })?)
        .and_then(|start| start.checked_add(scalars))
        .ok_or_else(|| {
            Error::invalid_input(
                "classfile_code_span_overflow",
                format!("{name} operand offset at BCI {bci} overflows"),
            )
        })?;
    let entries_end = count
        .checked_mul(unit)
        .and_then(|length| start.checked_add(length))
        .ok_or_else(|| {
            Error::invalid_input(
                "classfile_instruction_width_overflow",
                format!("{name} operand region at BCI {bci} overflows"),
            )
        })?;
    if entries_end != bytes.len() {
        return Err(Error::invalid_input(
            "classfile_instruction_shape_mismatch",
            format!(
                "{name} at BCI {bci} has {} recorded bytes but {} entries need {entries_end}",
                bytes.len(),
                count
            ),
        ));
    }
    Ok(start)
}

/// One big-endian `i32` switch operand, checked against the instruction bytes.
fn switch_i32(bytes: &[u8], bci: u32, offset: usize) -> Result<i32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| switch_operand_bounds(bytes, bci, offset))?;
    let value = bytes
        .get(offset..end)
        .ok_or_else(|| switch_operand_bounds(bytes, bci, offset))?;
    Ok(i32::from_be_bytes(
        value.try_into().expect("slice length was checked"),
    ))
}

/// Structured error for a switch operand that leaves the instruction bytes.
fn switch_operand_bounds(bytes: &[u8], bci: u32, offset: usize) -> Error {
    Error::invalid_input(
        "classfile_instruction_shape_mismatch",
        format!(
            "switch operand at byte {offset} of BCI {bci} leaves the {} recorded instruction bytes",
            bytes.len()
        ),
    )
}

/// Decodes one method's `Code` attribute as reusable facts.
///
/// # Billing
///
/// * `AttributeBytes` — one charge for the `Code` attribute entry, `6 + content
///   length`, the same entry charge `attribute_content` makes for the same shell.
/// * `CodeBytes` — one charge per decoded instruction width.
/// * `ClassBytes` — **not** charged: the caller already paid it by calling
///   `class_facts` on these same bytes, and this function only re-parses the class
///   structure to reach noak's instruction cursor.
/// * `ResultItems` — never charged. `ResultItems` counts items a public result
///   returns, and the query scan owns that budget.
///
/// # Identity and scope
///
/// `method` is the `class_facts` header of the body to decode. Its raw name and
/// descriptor select the method, and the located `Code` attribute must be the one the
/// header's shells describe, so a header from another class is a structured error
/// instead of a silently wrong body. Only that attribute content is read; instructions
/// are not turned into a graph and no other attribute is decoded.
///
/// A member that declares no `Code` attribute is `Unsupported
/// { classfile_method_has_no_code }`. That absence is a normal declaration shape, not a
/// decode failure, so the caller decides whether to skip the member; the structural
/// check costs no read (`MemberHeader::attributes` already carries the shells).
pub fn method_code_facts(
    bytes: &[u8],
    method: &MemberHeader,
    budget: &mut Budget,
) -> Result<MethodCodeFacts> {
    budget.poll()?;
    // `class_facts` already ran the class-shaped pre-checks on these bytes, so this
    // pass is the structural read only and charges nothing.
    let class = Class::new(bytes).map_err(map_decode_error)?;
    if class.buffer_size() != bytes.len() {
        return Err(Error::invalid_input(
            "classfile_trailing_bytes",
            "class structure has trailing bytes",
        ));
    }
    let pool = class.pool();
    let mut matches = Vec::new();
    for entry in class.methods() {
        budget.poll()?;
        let entry = entry.map_err(map_decode_error)?;
        let name = pool
            .get(entry.name())
            .map_err(map_decode_error)?
            .content
            .as_bytes();
        let descriptor = pool
            .get(entry.descriptor())
            .map_err(map_decode_error)?
            .content
            .as_bytes();
        if name == method.name.raw().0.as_slice()
            && descriptor == method.descriptor.raw().0.as_slice()
        {
            matches.push(entry);
        }
    }
    let candidate = match matches.len() {
        0 => {
            return Err(Error::invalid_input(
                "classfile_method_not_found",
                "no method exactly matched the raw name and descriptor",
            ));
        }
        1 => matches.pop().expect("length checked"),
        _ => {
            return Err(Error::invalid_input(
                "classfile_method_ambiguous",
                "multiple methods exactly matched the raw name and descriptor",
            ));
        }
    };
    let mut code_attributes = Vec::new();
    for attribute in candidate.attributes() {
        budget.poll()?;
        let attribute = attribute.map_err(map_decode_error)?;
        let name = pool
            .get(attribute.name())
            .map_err(map_decode_error)?
            .content
            .as_bytes();
        if name == b"Code" {
            code_attributes.push(attribute);
        }
    }
    let attribute = match code_attributes.len() {
        0 => {
            return Err(Error::unsupported(
                "classfile_method_has_no_code",
                "selected method has no Code attribute",
            ));
        }
        1 => code_attributes.pop().expect("length checked"),
        _ => {
            return Err(Error::invalid_input(
                "classfile_duplicate_code_attribute",
                "selected method has multiple Code attributes",
            ));
        }
    };
    let content = attribute.content();
    let content_span = span_for_slice(bytes, content)?;
    if !member_has_code_shell(method, &content_span) {
        return Err(Error::invalid_input(
            "classfile_method_header_mismatch",
            "the Code attribute of this method does not match the member header's shells",
        ));
    }
    let shell_length = content_span
        .length
        .checked_add(ATTRIBUTE_HEADER_LENGTH as u64)
        .ok_or_else(|| {
            Error::invalid_input("classfile_span_overflow", "Code shell length overflow")
        })?;
    budget.charge(CountedBudgetDimension::AttributeBytes, shell_length)?;

    // `Code` content: max_stack(2) max_locals(2) code_length(4) code[] ...
    let code_offset = 8usize;
    let code_length = read_u32(content, 4)?;
    let code_length_usize = usize::try_from(code_length).map_err(|_| {
        Error::invalid_input(
            "classfile_code_span_overflow",
            "code length does not fit usize",
        )
    })?;
    let code_end = code_offset.checked_add(code_length_usize).ok_or_else(|| {
        Error::invalid_input("classfile_code_span_overflow", "code span overflow")
    })?;
    let code_bytes = content.get(code_offset..code_end).ok_or_else(|| {
        Error::invalid_input(
            "classfile_code_out_of_bounds",
            "code array exceeds Code attribute content",
        )
    })?;
    let code_start = content_span
        .start
        .checked_add(code_offset as u64)
        .ok_or_else(|| {
            Error::invalid_input("classfile_code_span_overflow", "code start overflow")
        })?;
    let code_span = checked_span(bytes, code_start, u64::from(code_length))?;
    let decoded = attribute.read_content(pool).map_err(map_decode_error)?;
    let code = match decoded {
        noak::reader::AttributeContent::Code(code) => code,
        _ => unreachable!("raw Code name selected"),
    };
    // The body's own debug names, from the `LocalVariableTable` this same `Code` entry may carry
    // (P3 3.1). The nested bytes are inside the entry this read already charged for, so the walk costs
    // no dimension and reads nothing twice; a table whose content does not decode states no name
    // instead of failing a body whose instructions decoded.
    let debug = local_debug_table(&code, pool);

    let exception_handler_count =
        u32::try_from(code.exception_handlers().count()).map_err(|_| {
            Error::invalid_input(
                "classfile_result_overflow",
                "handler count does not fit u32",
            )
        })?;
    let mut handlers = Vec::new();
    for (ordinal, handler) in code.exception_handlers().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| {
            Error::invalid_input("classfile_result_overflow", "handler ordinal overflow")
        })?;
        if let Err(error) = budget.poll() {
            return stopped_code_facts(
                &code,
                code_span,
                Vec::new(),
                Vec::new(),
                handlers,
                BytecodeStopPosition::ExceptionHandler(ordinal),
                StopFailure::Budget(&error),
                budget,
            );
        }
        handlers.push(ExceptionHandlerFact {
            ordinal,
            start_bci: handler.start().as_u32(),
            end_bci: handler.end().as_u32(),
            handler_bci: handler.handler().as_u32(),
            catch_type_index: handler.catch_type().map(|index| index.as_u16()),
        });
    }

    let mut instructions = Vec::new();
    let mut operands = Vec::new();
    let mut cursor_bci = 0u32;
    for item in code.raw_instructions() {
        if let Err(error) = budget.poll() {
            return stopped_code_facts(
                &code,
                code_span,
                instructions,
                operands,
                handlers,
                BytecodeStopPosition::Instruction(cursor_bci),
                StopFailure::Budget(&error),
                budget,
            );
        }
        let (index, instruction) = match item {
            Ok(value) => value,
            // The noak decode detail is not carried into these facts: the stable code
            // and the stop position are what a consumer reports, and the public path
            // owns the diagnostic rendering of the adapter error.
            Err(_error) => {
                return stopped_code_facts(
                    &code,
                    code_span,
                    instructions,
                    operands,
                    handlers,
                    BytecodeStopPosition::Instruction(cursor_bci),
                    StopFailure::Decode("classfile_instruction_decode"),
                    budget,
                );
            }
        };
        if index.as_u32() != cursor_bci {
            return Err(Error::invalid_input(
                "classfile_instruction_cursor_mismatch",
                format!(
                    "noak returned BCI {} but expected {cursor_bci}",
                    index.as_u32()
                ),
            ));
        }
        let start = usize::try_from(cursor_bci).map_err(|_| {
            Error::invalid_input("classfile_code_span_overflow", "BCI does not fit usize")
        })?;
        let opcode = *code_bytes.get(start).ok_or_else(|| {
            Error::invalid_input(
                "classfile_instruction_out_of_bounds",
                "instruction opcode is outside code array",
            )
        })?;
        let width = match instruction_width(opcode, cursor_bci, &instruction) {
            Ok(width) => width,
            Err(error) => {
                let Error::InvalidInput {
                    code: adapter_code, ..
                } = &error
                else {
                    return Err(error);
                };
                return stopped_code_facts(
                    &code,
                    code_span,
                    instructions,
                    operands,
                    handlers,
                    BytecodeStopPosition::Instruction(cursor_bci),
                    StopFailure::Decode(adapter_code),
                    budget,
                );
            }
        };
        let end = match cursor_bci.checked_add(width) {
            Some(end) => end,
            None => {
                return Err(Error::invalid_input(
                    "classfile_instruction_width_overflow",
                    "instruction end overflow",
                ));
            }
        };
        let code_length_u32 = u32::try_from(code_bytes.len()).map_err(|_| {
            Error::invalid_input(
                "classfile_code_span_overflow",
                "code length does not fit u32",
            )
        })?;
        if end > code_length_u32 {
            return Err(Error::invalid_input(
                "classfile_instruction_out_of_bounds",
                "instruction exceeds code array",
            ));
        }
        if let Err(error) = budget.charge(CountedBudgetDimension::CodeBytes, u64::from(width)) {
            return stopped_code_facts(
                &code,
                code_span,
                instructions,
                operands,
                handlers,
                BytecodeStopPosition::Instruction(cursor_bci),
                StopFailure::Budget(&error),
                budget,
            );
        }
        let span_start = code_span
            .start
            .checked_add(u64::from(cursor_bci))
            .ok_or_else(|| {
                Error::invalid_input("classfile_code_span_overflow", "instruction span overflow")
            })?;
        let end_usize = usize::try_from(end).map_err(|_| {
            Error::invalid_input(
                "classfile_code_span_overflow",
                "instruction end does not fit usize",
            )
        })?;
        let instruction_bytes = code_bytes.get(start..end_usize).ok_or_else(|| {
            Error::invalid_input(
                "classfile_instruction_out_of_bounds",
                "instruction slice exceeds code array",
            )
        })?;
        // Operands come out of the same event and the same slice as the fact below, so
        // the two lists stay in lockstep and share the `CodeBytes` charge above.
        let operands_fact =
            match instruction_operands(opcode, cursor_bci, &instruction, instruction_bytes, pool) {
                Ok(fact) => fact,
                Err(error) => {
                    let Error::InvalidInput {
                        code: adapter_code, ..
                    } = &error
                    else {
                        return Err(error);
                    };
                    return stopped_code_facts(
                        &code,
                        code_span,
                        instructions,
                        operands,
                        handlers,
                        BytecodeStopPosition::Instruction(cursor_bci),
                        StopFailure::Decode(adapter_code),
                        budget,
                    );
                }
            };
        instructions.push(InstructionFact {
            bci: cursor_bci,
            opcode,
            width,
            span: ByteSpan::new(span_start, u64::from(width)),
            operands_span: ByteSpan::new(span_start + 1, u64::from(width - 1)),
            constant_pool_index: constant_pool_index(opcode, instruction_bytes),
        });
        operands.push(operands_fact);
        cursor_bci = end;
    }
    if cursor_bci != code_bytes.len() as u32 {
        return Err(Error::invalid_input(
            "classfile_instruction_cursor_mismatch",
            "instruction cursor did not consume the code array",
        ));
    }

    Ok(MethodCodeFacts {
        max_stack: code.max_stack(),
        max_locals: code.max_locals(),
        code_span,
        instructions,
        operands,
        exception_handlers: handlers,
        exception_handler_count,
        execution: ExecutionReport::Complete {
            usage: budget.usage(),
        },
        stopped_at: None,
        debug,
    })
}

/// Reads the `LocalVariableTable` nested inside one already-decoded `Code` attribute.
///
/// The nested attribute list is walked from the `Code` content the caller already decoded and charged
/// for, so nothing is sliced, billed or decoded a second time: the loop runs only because this
/// function is called, and it decodes exactly one nested attribute — the table that names slots. Every
/// other nested attribute (`LineNumberTable`, `StackMapTable`, type annotations) is skipped by name
/// without reading its content.
///
/// A table the bytes cannot hold, or a name index that does not resolve, is [`LocalDebugTable::Unstated`]
/// rather than an error: a body's instructions decoded, and an attribute that only *names* things must
/// not turn that body into a failure. What it must not do either is invent a name, and it does not: no
/// name is stated.
fn local_debug_table(
    code: &Code<'_>,
    pool: &noak::reader::cpool::ConstantPool<'_>,
) -> LocalDebugTable {
    let mut names = Vec::new();
    for attribute in code.attributes() {
        let Ok(attribute) = attribute else {
            return LocalDebugTable::Unstated;
        };
        let Ok(name) = pool.get(attribute.name()) else {
            return LocalDebugTable::Unstated;
        };
        if name.content.as_bytes() != b"LocalVariableTable" {
            continue;
        }
        let Ok(noak::reader::AttributeContent::LocalVariableTable(table)) =
            attribute.read_content(pool)
        else {
            return LocalDebugTable::Unstated;
        };
        for variable in table.locals() {
            let Ok(variable) = variable else {
                return LocalDebugTable::Unstated;
            };
            let Ok(entry) = pool.get(variable.name()) else {
                return LocalDebugTable::Unstated;
            };
            let range = variable.range();
            names.push(LocalDebugName {
                slot: variable.index(),
                start_bci: range.start.as_u32(),
                end_bci: range.end.as_u32(),
                name: JvmBytes(entry.content.as_bytes().to_vec()),
            });
        }
    }
    if names.is_empty() {
        // No table at all, or one that declares no record: both state no name, and the difference is
        // an empty list either way.
        LocalDebugTable::Absent
    } else {
        LocalDebugTable::Read(names)
    }
}

/// Whether one of the header's shells describes the located `Code` content.
fn member_has_code_shell(method: &MemberHeader, content_span: &ByteSpan) -> bool {
    method.attributes.iter().any(|shell| {
        shell.name.raw().0.as_slice() == b"Code" && shell.content_span == *content_span
    })
}

/// Why a method body stopped before the end of its `Code` attribute.
enum StopFailure<'a> {
    /// A budget or cancellation condition raised by a `Budget` call.
    Budget(&'a Error),
    /// The adapter could not decode one instruction; the code names why.
    Decode(&'a str),
}

/// Reliable prefix of a method body that stopped early.
///
/// The stop is turned into the same terminal vocabulary the public
/// `inspect_method_bytecode` path uses, so a cancelled or exhausted decode never reads
/// as a complete body: a cancellation stays a cancellation, a budget stop keeps its
/// dimension, and a decode failure keeps its own code.
#[allow(clippy::too_many_arguments)]
fn stopped_code_facts(
    code: &Code<'_>,
    code_span: ByteSpan,
    instructions: Vec<InstructionFact>,
    operands: Vec<InstructionOperands>,
    handlers: Vec<ExceptionHandlerFact>,
    position: BytecodeStopPosition,
    failure: StopFailure<'_>,
    budget: &Budget,
) -> Result<MethodCodeFacts> {
    let (execution, stable_code) = match failure {
        StopFailure::Budget(error) => match error {
            Error::Cancelled { .. } => (
                ExecutionReport::Cancelled {
                    usage: budget.usage(),
                },
                "classfile_bytecode_cancelled",
            ),
            Error::BudgetExceeded { dimension, .. } => (
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: *dimension,
                    },
                    usage: budget.usage(),
                },
                "classfile_bytecode_budget_exceeded",
            ),
            _ => (
                ExecutionReport::Partial {
                    reason: TerminationReason::Error {
                        code: "classfile_bytecode_failed".to_owned(),
                    },
                    usage: budget.usage(),
                },
                "classfile_bytecode_failed",
            ),
        },
        StopFailure::Decode(code) => (
            ExecutionReport::Partial {
                reason: TerminationReason::Error {
                    code: code.to_owned(),
                },
                usage: budget.usage(),
            },
            code,
        ),
    };
    let stopped_at = bytecode_stop(&code_span, position, stable_code)?;
    Ok(MethodCodeFacts {
        max_stack: code.max_stack(),
        max_locals: code.max_locals(),
        code_span,
        instructions,
        operands,
        exception_handlers: handlers,
        exception_handler_count: u32::try_from(code.exception_handlers().count()).map_err(
            |_| {
                Error::invalid_input(
                    "classfile_result_overflow",
                    "handler count does not fit u32",
                )
            },
        )?,
        execution,
        stopped_at: Some(stopped_at),
        // The body stopped before the nested attributes were walked: this read states no debug name,
        // which is what "no name" means for a record that did not reach them (P3 3.1).
        debug: LocalDebugTable::Unstated,
    })
}

/// Resolves one 1-based constant-pool index inside a fact table.
#[allow(dead_code)]
pub fn cp_entry(pool: &[CpEntryFacts], index: u16) -> Result<&CpEntryFacts> {
    if index == 0 {
        return Err(cp_index_error(index));
    }
    pool.binary_search_by_key(&index, |entry| entry.index)
        .map(|position| &pool[position])
        .map_err(|_| cp_index_error(index))
}

/// Resolves a `CONSTANT_Utf8` index to its raw Modified UTF-8 bytes.
#[allow(dead_code)]
pub fn cp_utf8(pool: &[CpEntryFacts], index: u16) -> Result<JvmBytes> {
    match &cp_entry(pool, index)?.kind {
        CpEntryKind::Utf8 { bytes } => Ok(bytes.clone()),
        _ => Err(cp_fact_tag_mismatch("CONSTANT_Utf8", index)),
    }
}

/// Resolves a `CONSTANT_Class` index to its internal-name bytes.
#[allow(dead_code)]
pub fn cp_class_name(pool: &[CpEntryFacts], index: u16) -> Result<JvmBytes> {
    match &cp_entry(pool, index)?.kind {
        CpEntryKind::Class { name, .. } => Ok(name.clone()),
        _ => Err(cp_fact_tag_mismatch("CONSTANT_Class", index)),
    }
}

const CP_TAG_UTF8: u8 = 1;
const CP_TAG_INTEGER: u8 = 3;
const CP_TAG_FLOAT: u8 = 4;
const CP_TAG_LONG: u8 = 5;
const CP_TAG_DOUBLE: u8 = 6;
const CP_TAG_CLASS: u8 = 7;
const CP_TAG_STRING: u8 = 8;
const CP_TAG_FIELD_REF: u8 = 9;
const CP_TAG_METHOD_REF: u8 = 10;
const CP_TAG_INTERFACE_METHOD_REF: u8 = 11;
const CP_TAG_NAME_AND_TYPE: u8 = 12;
const CP_TAG_METHOD_HANDLE: u8 = 15;
const CP_TAG_METHOD_TYPE: u8 = 16;
const CP_TAG_DYNAMIC: u8 = 17;
const CP_TAG_INVOKE_DYNAMIC: u8 = 18;
const CP_TAG_MODULE: u8 = 19;
const CP_TAG_PACKAGE: u8 = 20;

/// One measured constant-pool slot: enough to slice the entry and to read its
/// payload without re-deriving offsets from noak, whose decoder is private.
#[derive(Debug)]
struct CpSlotLayout {
    index: u16,
    span: ByteSpan,
    tag: u8,
    payload_start: usize,
}

/// Walks the constant pool with an explicit tag-length table.
///
/// A tag this table cannot measure stops the walk (`classfile_unknown_constant_pool_tag`)
/// instead of guessing a width: per the reader's unknown-tag policy, an
/// unmeasurable tag makes the rest of the class unreliable, so neither a partial
/// order nor invented spans are returned.
fn measure_constant_pool(bytes: &[u8], budget: &Budget) -> Result<Vec<CpSlotLayout>> {
    if bytes.len() < 10 || !bytes.starts_with(&0xcafebabe_u32.to_be_bytes()) {
        return Ok(Vec::new()); // noak owns fixed-header diagnostics.
    }
    let count = read_u16(bytes, 8)?;
    if count == 0 {
        return Ok(Vec::new()); // noak supplies the canonical InvalidLength diagnostic.
    }
    let mut slots = Vec::new();
    let mut slot = 1u16;
    let mut offset = 10usize;
    while slot < count {
        budget.poll()?;
        let entry_start = offset;
        let tag = *bytes.get(offset).ok_or_else(|| cp_guard_eoi(offset))?;
        let payload_start = offset_plus(offset, 1)?;
        let payload_length = match tag {
            CP_TAG_UTF8 => usize::from(read_u16(bytes, payload_start)?) + 2,
            CP_TAG_INTEGER | CP_TAG_FLOAT => 4,
            CP_TAG_LONG | CP_TAG_DOUBLE => 8,
            CP_TAG_CLASS | CP_TAG_STRING | CP_TAG_METHOD_TYPE | CP_TAG_MODULE | CP_TAG_PACKAGE => 2,
            CP_TAG_FIELD_REF
            | CP_TAG_METHOD_REF
            | CP_TAG_INTERFACE_METHOD_REF
            | CP_TAG_NAME_AND_TYPE
            | CP_TAG_DYNAMIC
            | CP_TAG_INVOKE_DYNAMIC => 4,
            CP_TAG_METHOD_HANDLE => 3,
            other => {
                return Err(Error::invalid_input(
                    "classfile_unknown_constant_pool_tag",
                    format!(
                        "constant-pool tag {other} at slot {slot} has no known width, so sequential parsing stops here"
                    ),
                ));
            }
        };
        let end = offset_plus(payload_start, payload_length)?;
        if end > bytes.len() {
            return Err(cp_guard_eoi(entry_start));
        }
        slots.push(CpSlotLayout {
            index: slot,
            span: ByteSpan::new(to_u64(entry_start)?, to_u64(end - entry_start)?),
            tag,
            payload_start,
        });
        offset = end;
        let slot_width = if matches!(tag, CP_TAG_LONG | CP_TAG_DOUBLE) {
            2u16
        } else {
            1u16
        };
        slot = slot.checked_add(slot_width).ok_or_else(cp_guard_overflow)?;
    }
    if slot != count {
        return Err(Error::invalid_input(
            "classfile_invalid_constant_pool_slots",
            format!("constant pool declares {count} slots but the walk consumed {slot}"),
        ));
    }
    Ok(slots)
}

/// Cross-checks the measured layout against noak's decode before any fact is built.
///
/// The tag sequence must match entry for entry, and the walk must end exactly
/// where the class-level structure begins; otherwise a wrong width would produce
/// spans and payload reads that look plausible but point at the wrong bytes.
fn verify_constant_pool_layout(
    bytes: &[u8],
    layout: &[CpSlotLayout],
    class: &Class<'_>,
) -> Result<()> {
    let mut seen = 0usize;
    for (index, item) in class.pool().iter_indices() {
        let slot = layout.get(seen).ok_or_else(constant_pool_layout_mismatch)?;
        if slot.index != index.as_u16() || slot.tag != noak_item_tag(item) {
            return Err(constant_pool_layout_mismatch());
        }
        seen += 1;
    }
    if seen != layout.len() {
        return Err(constant_pool_layout_mismatch());
    }
    let cp_end = match layout.last() {
        Some(slot) => span_end(&slot.span)?,
        None => 10,
    };
    let cp_end = usize::try_from(cp_end).map_err(|_| constant_pool_layout_mismatch())?;
    let access_flags = read_u16(bytes, cp_end).map_err(|_| constant_pool_layout_mismatch())?;
    if access_flags != class.access_flags().bits() {
        return Err(constant_pool_layout_mismatch());
    }
    Ok(())
}

fn noak_item_tag(item: &noak::reader::cpool::Item<'_>) -> u8 {
    use noak::reader::cpool::Item;
    match item {
        Item::Utf8(_) => CP_TAG_UTF8,
        Item::Integer(_) => CP_TAG_INTEGER,
        Item::Float(_) => CP_TAG_FLOAT,
        Item::Long(_) => CP_TAG_LONG,
        Item::Double(_) => CP_TAG_DOUBLE,
        Item::Class(_) => CP_TAG_CLASS,
        Item::String(_) => CP_TAG_STRING,
        Item::FieldRef(_) => CP_TAG_FIELD_REF,
        Item::MethodRef(_) => CP_TAG_METHOD_REF,
        Item::InterfaceMethodRef(_) => CP_TAG_INTERFACE_METHOD_REF,
        Item::NameAndType(_) => CP_TAG_NAME_AND_TYPE,
        Item::MethodHandle(_) => CP_TAG_METHOD_HANDLE,
        Item::MethodType(_) => CP_TAG_METHOD_TYPE,
        Item::Dynamic(_) => CP_TAG_DYNAMIC,
        Item::InvokeDynamic(_) => CP_TAG_INVOKE_DYNAMIC,
        Item::Module(_) => CP_TAG_MODULE,
        Item::Package(_) => CP_TAG_PACKAGE,
    }
}

/// Builds the resolved fact view from the measured layout.
fn resolve_constant_pool(
    bytes: &[u8],
    layout: &[CpSlotLayout],
    budget: &Budget,
) -> Result<Vec<CpEntryFacts>> {
    let mut entries = Vec::with_capacity(layout.len());
    for slot in layout {
        budget.poll()?;
        let kind = match slot.tag {
            CP_TAG_UTF8 => CpEntryKind::Utf8 {
                bytes: utf8_payload(bytes, slot)?,
            },
            CP_TAG_INTEGER => CpEntryKind::Integer {
                value: i32::from_be_bytes(read_array(bytes, slot.payload_start)?),
            },
            CP_TAG_FLOAT => CpEntryKind::Float {
                bits: read_u32(bytes, slot.payload_start)?,
            },
            CP_TAG_LONG => CpEntryKind::Long {
                value: i64::from_be_bytes(read_array(bytes, slot.payload_start)?),
            },
            CP_TAG_DOUBLE => CpEntryKind::Double {
                bits: u64::from_be_bytes(read_array(bytes, slot.payload_start)?),
            },
            CP_TAG_CLASS => {
                let name_index = read_u16(bytes, slot.payload_start)?;
                CpEntryKind::Class {
                    name_index,
                    name: utf8_index(bytes, layout, name_index)?,
                }
            }
            CP_TAG_STRING => {
                let index = read_u16(bytes, slot.payload_start)?;
                CpEntryKind::String {
                    utf8_index: index,
                    value: utf8_index(bytes, layout, index)?,
                }
            }
            CP_TAG_FIELD_REF | CP_TAG_METHOD_REF | CP_TAG_INTERFACE_METHOD_REF => {
                let class_index = read_u16(bytes, slot.payload_start)?;
                let name_and_type_index = read_u16(bytes, offset_plus(slot.payload_start, 2)?)?;
                let owner = class_name(bytes, layout, class_index)?;
                let (name, descriptor) = name_and_type(bytes, layout, name_and_type_index)?;
                match slot.tag {
                    CP_TAG_FIELD_REF => CpEntryKind::FieldRef {
                        class_index,
                        name_and_type_index,
                        owner,
                        name,
                        descriptor,
                    },
                    CP_TAG_METHOD_REF => CpEntryKind::MethodRef {
                        class_index,
                        name_and_type_index,
                        owner,
                        name,
                        descriptor,
                    },
                    _ => CpEntryKind::InterfaceMethodRef {
                        class_index,
                        name_and_type_index,
                        owner,
                        name,
                        descriptor,
                    },
                }
            }
            CP_TAG_NAME_AND_TYPE => {
                let name_index = read_u16(bytes, slot.payload_start)?;
                let descriptor_index = read_u16(bytes, offset_plus(slot.payload_start, 2)?)?;
                CpEntryKind::NameAndType {
                    name_index,
                    descriptor_index,
                    name: utf8_index(bytes, layout, name_index)?,
                    descriptor: utf8_index(bytes, layout, descriptor_index)?,
                }
            }
            CP_TAG_METHOD_HANDLE => CpEntryKind::MethodHandle {
                reference_kind: *bytes
                    .get(slot.payload_start)
                    .ok_or_else(|| cp_guard_eoi(slot.payload_start))?,
                reference_index: read_u16(bytes, offset_plus(slot.payload_start, 1)?)?,
            },
            CP_TAG_METHOD_TYPE => {
                let descriptor_index = read_u16(bytes, slot.payload_start)?;
                CpEntryKind::MethodType {
                    descriptor_index,
                    descriptor: utf8_index(bytes, layout, descriptor_index)?,
                }
            }
            CP_TAG_DYNAMIC | CP_TAG_INVOKE_DYNAMIC => {
                let bootstrap_method_attr_index = read_u16(bytes, slot.payload_start)?;
                let name_and_type_index = read_u16(bytes, offset_plus(slot.payload_start, 2)?)?;
                let (name, descriptor) = name_and_type(bytes, layout, name_and_type_index)?;
                if slot.tag == CP_TAG_DYNAMIC {
                    CpEntryKind::Dynamic {
                        bootstrap_method_attr_index,
                        name_and_type_index,
                        name,
                        descriptor,
                    }
                } else {
                    CpEntryKind::InvokeDynamic {
                        bootstrap_method_attr_index,
                        name_and_type_index,
                        name,
                        descriptor,
                    }
                }
            }
            CP_TAG_MODULE | CP_TAG_PACKAGE => {
                let name_index = read_u16(bytes, slot.payload_start)?;
                let name = utf8_index(bytes, layout, name_index)?;
                if slot.tag == CP_TAG_MODULE {
                    CpEntryKind::Module { name_index, name }
                } else {
                    CpEntryKind::Package { name_index, name }
                }
            }
            other => {
                return Err(Error::invalid_input(
                    "classfile_unknown_constant_pool_tag",
                    format!(
                        "constant-pool tag {other} at slot {} has no fact view, so parsing stops here",
                        slot.index
                    ),
                ));
            }
        };
        entries.push(CpEntryFacts {
            index: slot.index,
            span: slot.span.clone(),
            kind,
        });
    }
    Ok(entries)
}

/// Finds one measured slot by its 1-based index.
fn layout_entry(layout: &[CpSlotLayout], index: u16) -> Result<&CpSlotLayout> {
    if index == 0 {
        return Err(cp_index_error(index));
    }
    layout
        .binary_search_by_key(&index, |slot| slot.index)
        .map(|position| &layout[position])
        .map_err(|_| cp_index_error(index))
}

fn utf8_index(bytes: &[u8], layout: &[CpSlotLayout], index: u16) -> Result<JvmBytes> {
    let slot = layout_entry(layout, index)?;
    if slot.tag != CP_TAG_UTF8 {
        return Err(cp_tag_mismatch_error("CONSTANT_Utf8", index, slot.tag));
    }
    utf8_payload(bytes, slot)
}

fn utf8_payload(bytes: &[u8], slot: &CpSlotLayout) -> Result<JvmBytes> {
    let length = usize::from(read_u16(bytes, slot.payload_start)?);
    let payload_start = offset_plus(slot.payload_start, 2)?;
    Ok(JvmBytes(bytes_at(bytes, payload_start, length)?.to_vec()))
}

fn class_name(bytes: &[u8], layout: &[CpSlotLayout], index: u16) -> Result<JvmBytes> {
    let slot = layout_entry(layout, index)?;
    if slot.tag != CP_TAG_CLASS {
        return Err(cp_tag_mismatch_error("CONSTANT_Class", index, slot.tag));
    }
    let name_index = read_u16(bytes, slot.payload_start)?;
    utf8_index(bytes, layout, name_index)
}

fn name_and_type(
    bytes: &[u8],
    layout: &[CpSlotLayout],
    index: u16,
) -> Result<(JvmBytes, JvmBytes)> {
    let slot = layout_entry(layout, index)?;
    if slot.tag != CP_TAG_NAME_AND_TYPE {
        return Err(cp_tag_mismatch_error(
            "CONSTANT_NameAndType",
            index,
            slot.tag,
        ));
    }
    let name_index = read_u16(bytes, slot.payload_start)?;
    let descriptor_index = read_u16(bytes, offset_plus(slot.payload_start, 2)?)?;
    Ok((
        utf8_index(bytes, layout, name_index)?,
        utf8_index(bytes, layout, descriptor_index)?,
    ))
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N]> {
    Ok(bytes_at(bytes, offset, N)?
        .try_into()
        .expect("slice length was checked"))
}

fn bytes_at(bytes: &[u8], offset: usize, length: usize) -> Result<&[u8]> {
    let end = offset_plus(offset, length)?;
    bytes.get(offset..end).ok_or_else(|| cp_guard_eoi(offset))
}

fn span_end(span: &ByteSpan) -> Result<u64> {
    span.start
        .checked_add(span.length)
        .ok_or_else(|| span_overflow("class-file span"))
}

fn offset_plus(offset: usize, delta: usize) -> Result<usize> {
    offset
        .checked_add(delta)
        .ok_or_else(|| span_overflow("class-file offset"))
}

fn span_overflow(what: &str) -> Error {
    Error::invalid_input("classfile_span_overflow", format!("{what} overflow"))
}

fn span_slice<'a>(bytes: &'a [u8], span: &ByteSpan, what: &str) -> Result<&'a [u8]> {
    let start = usize::try_from(span.start).map_err(|_| span_outside(what))?;
    let length = usize::try_from(span.length).map_err(|_| span_outside(what))?;
    let end = start
        .checked_add(length)
        .ok_or_else(|| span_outside(what))?;
    bytes.get(start..end).ok_or_else(|| span_outside(what))
}

fn span_outside(what: &str) -> Error {
    Error::invalid_input(
        "classfile_invalid_attribute_span",
        format!("{what} is outside the class bytes"),
    )
}

fn cp_index_error(index: u16) -> Error {
    Error::invalid_input(
        "classfile_invalid_constant_pool_index",
        format!("constant-pool index {index} does not hold an entry"),
    )
}

fn cp_tag_mismatch_error(required: &str, index: u16, actual: u8) -> Error {
    Error::invalid_input(
        "classfile_constant_pool_tag_mismatch",
        format!("constant-pool index {index} holds tag {actual}, expected {required}"),
    )
}

fn cp_fact_tag_mismatch(required: &str, index: u16) -> Error {
    Error::invalid_input(
        "classfile_constant_pool_tag_mismatch",
        format!("constant-pool index {index} is not a {required} entry"),
    )
}

fn constant_pool_layout_mismatch() -> Error {
    Error::invalid_input(
        "classfile_constant_pool_layout_mismatch",
        "the measured constant-pool layout disagrees with the decoded class structure",
    )
}

/// Bounded cursor over one attribute content.
///
/// A read past the end and unread trailing bytes both stop with
/// `classfile_invalid_attribute_content`: an attribute whose declared structure
/// does not match its content is malformed, not partially usable.
/// A bounded cursor over one attribute's content: the same reader every attribute fact in this
/// module and the modern-fact pass in [`crate::modern`] reads its fields with.
pub(crate) struct AttributeReader<'a> {
    content: &'a [u8],
    position: usize,
}

impl<'a> AttributeReader<'a> {
    pub(crate) const fn new(content: &'a [u8]) -> Self {
        Self {
            content,
            position: 0,
        }
    }

    /// Bytes read so far, in attribute-content coordinates.
    pub(crate) const fn position(&self) -> usize {
        self.position
    }

    pub(crate) fn u16(&mut self) -> Result<u16> {
        let start = self.position;
        let end = offset_plus(start, 2)?;
        let pair: [u8; 2] = self
            .content
            .get(start..end)
            .ok_or_else(|| attribute_eoi(start))?
            .try_into()
            .expect("slice length was checked");
        self.position = end;
        Ok(u16::from_be_bytes(pair))
    }

    pub(crate) fn u32(&mut self) -> Result<u32> {
        let start = self.position;
        let end = offset_plus(start, 4)?;
        let value: [u8; 4] = self
            .content
            .get(start..end)
            .ok_or_else(|| attribute_eoi(start))?
            .try_into()
            .expect("slice length was checked");
        self.position = end;
        Ok(u32::from_be_bytes(value))
    }

    pub(crate) fn skip(&mut self, length: usize) -> Result<()> {
        let end = offset_plus(self.position, length)?;
        if end > self.content.len() {
            return Err(attribute_eoi(self.position));
        }
        self.position = end;
        Ok(())
    }

    pub(crate) fn expect_end(&self) -> Result<()> {
        if self.position != self.content.len() {
            return Err(Error::invalid_input(
                "classfile_invalid_attribute_content",
                format!(
                    "attribute content has {} unread trailing byte(s)",
                    self.content.len() - self.position
                ),
            ));
        }
        Ok(())
    }
}

fn attribute_eoi(position: usize) -> Error {
    Error::invalid_input(
        "classfile_invalid_attribute_content",
        format!("attribute content ends before its declared structure at offset {position}"),
    )
}

pub(crate) fn ensure_unique(seen: &mut Vec<&'static str>, name: &'static str) -> Result<()> {
    if seen.contains(&name) {
        return Err(Error::invalid_input(
            "classfile_duplicate_attribute",
            format!("class or member declares more than one {name} attribute"),
        ));
    }
    seen.push(name);
    Ok(())
}

pub(crate) fn read_class_name_list(
    reader: &mut AttributeReader<'_>,
    pool: &[CpEntryFacts],
    budget: &Budget,
) -> Result<Vec<JvmBytes>> {
    let count = reader.u16()?;
    let mut names = Vec::new();
    for _ in 0..count {
        budget.poll()?;
        let index = reader.u16()?;
        names.push(cp_class_name(pool, index)?);
    }
    Ok(names)
}

/// Reads the `Module` attribute up to `uses`/`provides`.
///
/// Requires, exports and opens are only measured so that the cursor reaches the
/// sections P1 consumers read; their content is never retained.
fn read_module_facts(
    reader: &mut AttributeReader<'_>,
    pool: &[CpEntryFacts],
    budget: &Budget,
) -> Result<ModuleFacts> {
    reader.skip(6)?; // module_name_index, module_flags, module_version_index
    let requires_count = reader.u16()?;
    for _ in 0..requires_count {
        budget.poll()?;
        reader.skip(6)?; // requires_index, requires_flags, requires_version_index
    }
    skip_qualified_module_section(reader, budget)?; // exports
    skip_qualified_module_section(reader, budget)?; // opens
    let uses_count = reader.u16()?;
    let mut uses = Vec::new();
    for _ in 0..uses_count {
        budget.poll()?;
        let index = reader.u16()?;
        uses.push(cp_class_name(pool, index)?);
    }
    let provides_count = reader.u16()?;
    let mut provides = Vec::new();
    for _ in 0..provides_count {
        budget.poll()?;
        let service = cp_class_name(pool, reader.u16()?)?;
        let implementation_count = reader.u16()?;
        let mut implementations = Vec::new();
        for _ in 0..implementation_count {
            budget.poll()?;
            implementations.push(cp_class_name(pool, reader.u16()?)?);
        }
        provides.push(ProvidesFacts {
            service,
            implementations,
        });
    }
    Ok(ModuleFacts { uses, provides })
}

fn skip_qualified_module_section(reader: &mut AttributeReader<'_>, budget: &Budget) -> Result<()> {
    let count = reader.u16()?;
    for _ in 0..count {
        budget.poll()?;
        reader.skip(4)?; // section index, section flags
        let to_count = usize::from(reader.u16()?);
        reader.skip(
            to_count
                .checked_mul(2)
                .ok_or_else(|| span_overflow("module section length"))?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod reader_facts_tests {
    use super::*;
    use crate::budget::{BudgetDimension, CancellationToken, Limits};

    const V52_FIXTURE: &[u8] =
        crate::test_fixtures::fixture!("historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

    fn unlimited_limits() -> Limits {
        Limits {
            input_bytes: u64::MAX,
            archive_entries: u64::MAX,
            entry_bytes: u64::MAX,
            read_bytes: u64::MAX,
            class_bytes: u64::MAX,
            attribute_bytes: u64::MAX,
            code_bytes: u64::MAX,
            result_items: u64::MAX,
            output_bytes: u64::MAX,
            nested_depth: u64::MAX,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        }
    }

    fn budget() -> Budget {
        Budget::new(unlimited_limits())
    }

    /// Same as [`budget`], for call sites where a local binding shadows the name.
    fn fresh_budget() -> Budget {
        budget()
    }

    fn buf_u16(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_be_bytes());
    }

    fn buf_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_be_bytes());
    }

    fn attribute(out: &mut Vec<u8>, name_index: u16, content: &[u8]) {
        buf_u16(out, name_index);
        buf_u32(out, u32::try_from(content.len()).unwrap());
        out.extend_from_slice(content);
    }

    fn error_code(error: Error) -> String {
        match error {
            Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code,
            other => panic!("expected a structured error code, got {other:?}"),
        }
    }

    /// One recorded constant-pool entry: index, true byte offset and width.
    #[derive(Clone, Copy)]
    struct CpRef {
        index: u16,
        offset: u64,
        width: u64,
    }

    /// Assembles a constant pool while remembering each entry's real position, so
    /// span assertions come from the writer instead of from the reader.
    struct CpBuilder {
        bytes: Vec<u8>,
        next_index: u16,
        entries: Vec<CpRef>,
    }

    impl CpBuilder {
        fn new() -> Self {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
            buf_u16(&mut bytes, 0); // minor version
            buf_u16(&mut bytes, 52); // major version
            buf_u16(&mut bytes, 0); // constant_pool_count placeholder
            Self {
                bytes,
                next_index: 1,
                entries: Vec::new(),
            }
        }

        fn start(&mut self, tag: u8, payload: impl FnOnce(&mut Vec<u8>)) -> CpRef {
            let offset = self.bytes.len() as u64;
            self.bytes.push(tag);
            payload(&mut self.bytes);
            let reference = CpRef {
                index: self.next_index,
                offset,
                width: self.bytes.len() as u64 - offset,
            };
            self.entries.push(reference);
            self.next_index += 1;
            reference
        }

        fn wide(&mut self, tag: u8, payload: impl FnOnce(&mut Vec<u8>)) -> CpRef {
            let reference = self.start(tag, payload);
            self.next_index += 1; // Long/Double reserve the following slot
            reference
        }

        fn utf8(&mut self, value: &[u8]) -> CpRef {
            self.start(1, |bytes| {
                buf_u16(bytes, u16::try_from(value.len()).unwrap());
                bytes.extend_from_slice(value);
            })
        }

        fn integer(&mut self, value: i32) -> CpRef {
            self.start(3, |bytes| bytes.extend_from_slice(&value.to_be_bytes()))
        }

        fn float(&mut self, bits: u32) -> CpRef {
            self.start(4, |bytes| buf_u32(bytes, bits))
        }

        fn long(&mut self, value: i64) -> CpRef {
            self.wide(5, |bytes| bytes.extend_from_slice(&value.to_be_bytes()))
        }

        fn double(&mut self, bits: u64) -> CpRef {
            self.wide(6, |bytes| bytes.extend_from_slice(&bits.to_be_bytes()))
        }

        fn class(&mut self, name_index: u16) -> CpRef {
            self.start(7, |bytes| buf_u16(bytes, name_index))
        }

        fn string(&mut self, utf8_index: u16) -> CpRef {
            self.start(8, |bytes| buf_u16(bytes, utf8_index))
        }

        fn member_ref(&mut self, tag: u8, class_index: u16, name_and_type_index: u16) -> CpRef {
            self.start(tag, |bytes| {
                buf_u16(bytes, class_index);
                buf_u16(bytes, name_and_type_index);
            })
        }

        fn name_and_type(&mut self, name_index: u16, descriptor_index: u16) -> CpRef {
            self.start(12, |bytes| {
                buf_u16(bytes, name_index);
                buf_u16(bytes, descriptor_index);
            })
        }

        fn method_handle(&mut self, reference_kind: u8, reference_index: u16) -> CpRef {
            self.start(15, |bytes| {
                bytes.push(reference_kind);
                buf_u16(bytes, reference_index);
            })
        }

        fn method_type(&mut self, descriptor_index: u16) -> CpRef {
            self.start(16, |bytes| buf_u16(bytes, descriptor_index))
        }

        fn dynamic(
            &mut self,
            tag: u8,
            bootstrap_method_attr_index: u16,
            name_and_type_index: u16,
        ) -> CpRef {
            self.start(tag, |bytes| {
                buf_u16(bytes, bootstrap_method_attr_index);
                buf_u16(bytes, name_and_type_index);
            })
        }

        fn module(&mut self, name_index: u16) -> CpRef {
            self.start(19, |bytes| buf_u16(bytes, name_index))
        }

        fn package(&mut self, name_index: u16) -> CpRef {
            self.start(20, |bytes| buf_u16(bytes, name_index))
        }

        fn finish(mut self) -> (Vec<u8>, Vec<CpRef>) {
            let count = self.next_index;
            self.bytes[8..10].copy_from_slice(&count.to_be_bytes());
            (self.bytes, self.entries)
        }
    }

    fn find_shell<'a>(shells: &'a [AttributeShell], name: &[u8]) -> &'a AttributeShell {
        shells
            .iter()
            .find(|shell| shell.name.raw().0 == name)
            .unwrap_or_else(|| panic!("no attribute shell named {name:?}"))
    }

    #[test]
    fn real_fixture_class_facts_match_the_decoded_class() {
        let mut budget = budget();
        let facts = class_facts(V52_FIXTURE, &mut budget).unwrap();
        assert_eq!((facts.major_version, facts.minor_version), (52, 0));
        assert_eq!(facts.access_flags, 0x0021);
        assert_eq!(facts.this_class.raw().0, b"HistoricalControlFlow");
        assert_eq!(
            facts.super_class.as_ref().unwrap().raw().0,
            b"java/lang/Object"
        );
        assert!(facts.interfaces.is_empty());
        assert!(facts.fields.is_empty());
        assert!(facts.attributes.is_empty());
        assert_eq!(budget.usage().class_bytes, V52_FIXTURE.len() as u64);
        // Intermediate facts are not result items; the query layer owns that charge.
        assert_eq!(budget.usage().result_items, 0);
        // The three `Code` shells are enumerated, so their entries are billed the
        // same way `inspect_header` bills them: 6 header bytes plus the content.
        assert_eq!(budget.usage().attribute_bytes, 23 + 22 + 53);

        // `javap -v` and an independent byte walk both report 17 declared slots,
        // holding 16 entries at 1..=16 with no reserved slot.
        assert_eq!(facts.constant_pool.len(), 16);
        assert_eq!(facts.constant_pool.first().unwrap().index, 1);
        assert_eq!(facts.constant_pool.last().unwrap().index, 16);

        assert_eq!(
            cp_class_name(&facts.constant_pool, 1).unwrap(),
            JvmBytes(b"HistoricalControlFlow".to_vec())
        );
        assert_eq!(
            cp_utf8(&facts.constant_pool, 2).unwrap(),
            JvmBytes(b"HistoricalControlFlow".to_vec())
        );
        assert_eq!(
            cp_class_name(&facts.constant_pool, 3).unwrap(),
            JvmBytes(b"java/lang/Object".to_vec())
        );
        assert_eq!(
            cp_utf8(&facts.constant_pool, 5).unwrap(),
            JvmBytes(b"<init>".to_vec())
        );
        assert_eq!(
            cp_entry(&facts.constant_pool, 8).unwrap().kind,
            CpEntryKind::MethodRef {
                class_index: 3,
                name_and_type_index: 9,
                owner: JvmBytes(b"java/lang/Object".to_vec()),
                name: JvmBytes(b"<init>".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            }
        );
        assert_eq!(
            cp_entry(&facts.constant_pool, 9).unwrap().kind,
            CpEntryKind::NameAndType {
                name_index: 5,
                descriptor_index: 6,
                name: JvmBytes(b"<init>".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            }
        );
        assert_eq!(
            cp_class_name(&facts.constant_pool, 15).unwrap(),
            JvmBytes(b"java/lang/Throwable".to_vec())
        );

        assert_eq!(facts.methods.len(), 3);
        assert_eq!(facts.methods[2].name.raw().0, b"finallyPath");
        assert_eq!(facts.methods[2].descriptor.raw().0, b"(I)I");
        assert_eq!(facts.methods[2].access_flags, 0x0001);
        assert_eq!(facts.methods[2].attributes.len(), 1);
        assert_eq!(facts.methods[2].attributes[0].name.raw().0, b"Code");
        assert_eq!(facts.methods[2].attributes[0].span, ByteSpan::new(248, 53));
        assert_eq!(
            facts.methods[2].attributes[0].content_span,
            ByteSpan::new(254, 47)
        );
    }

    #[test]
    fn real_fixture_spans_land_on_the_true_constant_pool_offsets() {
        let facts = class_facts(V52_FIXTURE, &mut budget()).unwrap();

        // Offsets come from an independent walk of the file, not from the reader.
        let expected = [
            (1u16, 10u64, 3u64),
            (2, 13, 24),
            (4, 40, 19),
            (8, 81, 5),
            (9, 86, 5),
            (14, 126, 16),
            (15, 142, 3),
            (16, 145, 22),
        ];
        for (index, start, length) in expected {
            let entry = cp_entry(&facts.constant_pool, index).unwrap();
            assert_eq!(
                (entry.span.start, entry.span.length),
                (start, length),
                "entry #{index}"
            );
        }

        // Cross-validate the payloads by re-parsing the recorded spans.
        let method_ref = cp_entry(&facts.constant_pool, 8).unwrap();
        assert_eq!(V52_FIXTURE[81], 10);
        assert_eq!(read_u16(V52_FIXTURE, 82).unwrap(), 3);
        assert_eq!(read_u16(V52_FIXTURE, 84).unwrap(), 9);
        assert_eq!(method_ref.span.length, 5);

        assert_eq!(V52_FIXTURE[142], 7);
        assert_eq!(read_u16(V52_FIXTURE, 143).unwrap(), 16);

        let throwable = cp_entry(&facts.constant_pool, 16).unwrap();
        assert_eq!(V52_FIXTURE[145], 1);
        assert_eq!(read_u16(V52_FIXTURE, 146).unwrap(), 19);
        assert_eq!(
            &V52_FIXTURE[148..167],
            b"java/lang/Throwable",
            "Utf8 payload inside the recorded span"
        );
        assert_eq!(throwable.span.length, 3 + 19);

        // Entries tile the constant pool without gaps, and the pool ends exactly
        // where the class-level structure begins.
        assert_eq!(facts.constant_pool[0].span.start, 10);
        for pair in facts.constant_pool.windows(2) {
            assert_eq!(
                pair[0].span.start + pair[0].span.length,
                pair[1].span.start,
                "entry #{} must end where #{} begins",
                pair[0].index,
                pair[1].index
            );
        }
        let cp_end = {
            let last = facts.constant_pool.last().unwrap();
            last.span.start + last.span.length
        };
        assert_eq!(cp_end, 167);
        let header = cp_end as usize;
        assert_eq!(read_u16(V52_FIXTURE, header).unwrap(), facts.access_flags);
        assert_eq!(read_u16(V52_FIXTURE, header + 2).unwrap(), 1); // this_class
        assert_eq!(read_u16(V52_FIXTURE, header + 4).unwrap(), 3); // super_class
        assert_eq!(read_u16(V52_FIXTURE, header + 6).unwrap(), 0); // interfaces
        assert_eq!(read_u16(V52_FIXTURE, header + 8).unwrap(), 0); // fields
        assert_eq!(read_u16(V52_FIXTURE, header + 10).unwrap(), 3); // methods
        // The class attribute count closes the file, after fields and methods.
        assert_eq!(read_u16(V52_FIXTURE, V52_FIXTURE.len() - 2).unwrap(), 0); // attributes
    }

    #[test]
    fn facts_and_public_inspections_agree_on_the_same_input() {
        let facts = class_facts(V52_FIXTURE, &mut budget()).unwrap();
        let header = inspect_header(V52_FIXTURE, &mut budget(), InspectionMode::Forensic)
            .unwrap()
            .header;
        assert_eq!(facts.major_version, header.major_version);
        assert_eq!(facts.minor_version, header.minor_version);
        assert_eq!(facts.access_flags, header.access_flags);
        assert_eq!(facts.this_class, header.this_class);
        assert_eq!(facts.super_class, header.super_class);
        assert_eq!(facts.interfaces, header.interfaces);
        assert_eq!(facts.fields, header.fields);
        assert_eq!(facts.methods, header.methods);
        assert_eq!(facts.attributes, header.attributes);

        // The Code shell of `finallyPath` and the instruction report describe the
        // same bytes: re-read the Code header from the shell's content span.
        let shell = find_shell(&facts.methods[2].attributes, b"Code");
        let selector = MethodSelector {
            name: JvmBytes(b"finallyPath".to_vec()),
            descriptor: JvmBytes(b"(I)I".to_vec()),
        };
        let report = inspect_method_bytecode(V52_FIXTURE, selector, &mut budget()).unwrap();
        let content = shell.content_span.start as usize;
        assert_eq!(read_u16(V52_FIXTURE, content).unwrap(), report.max_stack);
        assert_eq!(
            read_u16(V52_FIXTURE, content + 2).unwrap(),
            report.max_locals
        );
        assert_eq!(
            read_u32(V52_FIXTURE, content + 4).unwrap(),
            report.code_span.length as u32
        );
        assert_eq!(report.code_span.start, shell.content_span.start + 8);

        // The same constant-pool index resolves to the same symbol for both APIs.
        let init = MethodSelector {
            name: JvmBytes(b"<init>".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        };
        let init_report = inspect_method_bytecode(V52_FIXTURE, init, &mut fresh_budget()).unwrap();
        let invoke = init_report
            .instructions
            .iter()
            .find(|instruction| instruction.opcode == 0xb7)
            .expect("constructor calls super");
        let index = invoke
            .constant_pool_index
            .expect("invokespecial carries an index");
        assert_eq!(index, 8);
        match cp_entry(&facts.constant_pool, index).unwrap().kind.clone() {
            CpEntryKind::MethodRef {
                owner,
                name,
                descriptor,
                ..
            } => {
                assert_eq!(owner, JvmBytes(b"java/lang/Object".to_vec()));
                assert_eq!(name, JvmBytes(b"<init>".to_vec()));
                assert_eq!(descriptor, JvmBytes(b"()V".to_vec()));
            }
            other => panic!("expected a MethodRef fact, got {other:?}"),
        }

        // Attributes this layer does not own stay unread.
        let code_only = attribute_facts(
            V52_FIXTURE,
            &facts.methods[0].attributes,
            &facts.constant_pool,
            &mut fresh_budget(),
        )
        .unwrap();
        assert_eq!(code_only, AttributeFacts::default());
    }

    #[test]
    fn budget_hits_and_cancellation_never_fabricate_facts() {
        let mut configured = unlimited_limits();
        configured.class_bytes = V52_FIXTURE.len() as u64 - 1;
        let mut budget = Budget::new(configured);
        assert!(matches!(
            class_facts(V52_FIXTURE, &mut budget).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::ClassBytes,
                ..
            }
        ));
        assert_eq!(budget.usage().class_bytes, 0);

        let facts = class_facts(V52_FIXTURE, &mut fresh_budget()).unwrap();
        let shell = find_shell(&facts.methods[0].attributes, b"Code").clone();
        let shell_bytes = 6 + shell.content_span.length;

        let mut configured = unlimited_limits();
        configured.attribute_bytes = shell_bytes - 1;
        let mut budget = Budget::new(configured);
        assert!(matches!(
            attribute_content(V52_FIXTURE, &shell, &mut budget).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::AttributeBytes,
                ..
            }
        ));
        assert_eq!(budget.usage().attribute_bytes, 0);

        // The exact shell length succeeds and is billed the same way the existing
        // attribute shell enumeration bills one entry, without a result item.
        let mut exact = unlimited_limits();
        exact.attribute_bytes = shell_bytes;
        exact.result_items = 0;
        let mut budget = Budget::new(exact);
        let content = attribute_content(V52_FIXTURE, &shell, &mut budget).unwrap();
        assert_eq!(content.len() as u64, shell.content_span.length);
        assert_eq!(budget.usage().attribute_bytes, shell_bytes);
        assert_eq!(budget.usage().result_items, 0);

        let token = CancellationToken::new();
        token.cancel();
        let mut budget = Budget::with_cancellation_token(unlimited_limits(), token);
        assert!(matches!(
            class_facts(V52_FIXTURE, &mut budget).unwrap_err(),
            Error::Cancelled { .. }
        ));
        assert_eq!(budget.usage().class_bytes, 0);

        let mut expired = unlimited_limits();
        expired.elapsed_millis = 0;
        let mut budget = Budget::new(expired);
        assert!(matches!(
            class_facts(V52_FIXTURE, &mut budget).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                ..
            }
        ));
    }

    #[test]
    fn unknown_tag_truncation_and_broken_shells_stop_with_structured_errors() {
        // An unknown tag: noak owns the class-level diagnostic, and the measured
        // walk refuses to guess a width on its own.
        let mut unknown = V52_FIXTURE.to_vec();
        unknown[10] = 2;
        assert_eq!(
            error_code(class_facts(&unknown, &mut budget()).unwrap_err()),
            "classfile_decode"
        );
        assert_eq!(
            error_code(
                measure_constant_pool(&unknown, &Budget::new(unlimited_limits())).unwrap_err()
            ),
            "classfile_unknown_constant_pool_tag"
        );

        // A truncated entry: both the class path and the walk stop structured.
        let truncated = &V52_FIXTURE[..14];
        assert_eq!(
            error_code(class_facts(truncated, &mut budget()).unwrap_err()),
            "classfile_decode"
        );
        assert_eq!(
            error_code(
                measure_constant_pool(truncated, &Budget::new(unlimited_limits())).unwrap_err()
            ),
            "classfile_decode"
        );

        let facts = class_facts(V52_FIXTURE, &mut budget()).unwrap();
        let name = facts.methods[0].attributes[0].name.clone();
        let mut budget = budget();

        // Content that leaves the class bytes is refused instead of sliced.
        let outside = AttributeShell {
            name: name.clone(),
            span: ByteSpan::new(0, 6),
            content_span: ByteSpan::new(6, 4096),
        };
        assert_eq!(
            error_code(attribute_content(V52_FIXTURE, &outside, &mut budget).unwrap_err()),
            "classfile_invalid_attribute_span"
        );

        // A shell whose spans contradict each other is refused as well.
        let inconsistent = AttributeShell {
            name,
            span: ByteSpan::new(0, 99),
            content_span: ByteSpan::new(6, 0),
        };
        assert_eq!(
            error_code(attribute_content(V52_FIXTURE, &inconsistent, &mut budget).unwrap_err()),
            "classfile_invalid_attribute_span"
        );
        assert_eq!(budget.usage().attribute_bytes, 0);

        // Broken index and tag lookups do not produce facts.
        assert_eq!(
            error_code(cp_entry(&facts.constant_pool, 0).unwrap_err()),
            "classfile_invalid_constant_pool_index"
        );
        assert_eq!(
            error_code(cp_entry(&facts.constant_pool, 17).unwrap_err()),
            "classfile_invalid_constant_pool_index"
        );
        assert_eq!(
            error_code(cp_utf8(&facts.constant_pool, 8).unwrap_err()),
            "classfile_constant_pool_tag_mismatch"
        );
        assert_eq!(
            error_code(cp_class_name(&facts.constant_pool, 2).unwrap_err()),
            "classfile_constant_pool_tag_mismatch"
        );
    }

    /// Index of the single constant-pool entry whose kind satisfies the predicate.
    ///
    /// These lookups only produce a handle; index assignment and spans are
    /// asserted against the builder's recorded positions, not against the reader.
    fn only_of(facts: &ClassFacts, matches: impl Fn(&CpEntryKind) -> bool) -> u16 {
        facts
            .constant_pool
            .iter()
            .find(|entry| matches(&entry.kind))
            .map(|entry| entry.index)
            .expect("a constant-pool entry with the requested kind")
    }

    fn i_seven_index(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::Integer { .. }))
    }

    fn float_index(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::Float { .. }))
    }

    fn long_index(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::Long { .. }))
    }

    fn double_index(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::Double { .. }))
    }

    fn string_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::String { .. }))
    }

    fn field_ref_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::FieldRef { .. }))
    }

    fn method_ref_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::MethodRef { .. }))
    }

    fn interface_method_ref_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| {
            matches!(kind, CpEntryKind::InterfaceMethodRef { .. })
        })
    }

    fn method_handle_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| {
            matches!(kind, CpEntryKind::MethodHandle { .. })
        })
    }

    fn method_type_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::MethodType { .. }))
    }

    fn dynamic_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::Dynamic { .. }))
    }

    fn invoke_dynamic_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| {
            matches!(kind, CpEntryKind::InvokeDynamic { .. })
        })
    }

    fn module_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::Module { .. }))
    }

    fn package_entry(facts: &ClassFacts) -> u16 {
        only_of(facts, |kind| matches!(kind, CpEntryKind::Package { .. }))
    }

    fn bootstrap_handle_index(facts: &ClassFacts) -> u16 {
        method_handle_entry(facts)
    }

    fn l_minus_two_slot(facts: &ClassFacts) -> u16 {
        long_index(facts) + 1
    }

    fn class_entry_for(facts: &ClassFacts, name: &[u8]) -> u16 {
        only_of(
            facts,
            |kind| matches!(kind, CpEntryKind::Class { name: value, .. } if value.0 == name),
        )
    }

    fn index_of_utf8(facts: &ClassFacts, value: &[u8]) -> u16 {
        only_of(
            facts,
            |kind| matches!(kind, CpEntryKind::Utf8 { bytes } if bytes.0 == value),
        )
    }

    fn index_of_nat(facts: &ClassFacts, name: &[u8], descriptor: &[u8]) -> u16 {
        only_of(facts, |kind| {
            matches!(
                kind,
                CpEntryKind::NameAndType { name: n, descriptor: d, .. }
                    if n.0 == name && d.0 == descriptor
            )
        })
    }

    fn nat_vf_index(facts: &ClassFacts) -> u16 {
        index_of_nat(facts, b"vf", b"I")
    }

    fn nat_run_index(facts: &ClassFacts) -> u16 {
        index_of_nat(facts, b"run", b"()V")
    }

    fn nat_indy_index(facts: &ClassFacts) -> u16 {
        index_of_nat(facts, b"indy", b"()Ljava/lang/Runnable;")
    }

    /// Constant-pool slot of the `MethodType` entry built by `bootstrap_class`.
    fn method_type_index() -> u16 {
        3
    }

    /// A minimal class whose only class attribute is `BootstrapMethods`, with the
    /// content the caller writes.
    ///
    /// Slots: 1 `Utf8 "BootstrapMethods"`, 2 `Utf8 "()V"`, 3 `MethodType #2`,
    /// 4 `MethodHandle #3`, 5 `Utf8 "condy"`, 6 `NameAndType #5:#2`,
    /// 7 `Utf8 "pkg/C"`, 8 `Class #7`, 9 `String #5`, 10 `Integer 1`,
    /// 11 `Dynamic #0.#6`.
    fn bootstrap_class(
        build: impl FnOnce(&mut Vec<u8>),
    ) -> (Vec<u8>, AttributeShell, Vec<CpEntryFacts>) {
        let mut cp = CpBuilder::new();
        let a_bootstrap = cp.utf8(b"BootstrapMethods");
        let u_void = cp.utf8(b"()V");
        cp.method_type(u_void.index);
        cp.method_handle(6, method_type_index());
        let u_condy = cp.utf8(b"condy");
        let nat = cp.name_and_type(u_condy.index, u_void.index);
        let u_class = cp.utf8(b"pkg/C");
        let c_class = cp.class(u_class.index);
        cp.string(u_condy.index);
        cp.integer(1);
        cp.dynamic(17, 0, nat.index);
        let (mut bytes, _) = cp.finish();
        buf_u16(&mut bytes, 0x0021);
        buf_u16(&mut bytes, c_class.index);
        buf_u16(&mut bytes, c_class.index);
        buf_u16(&mut bytes, 0); // interfaces
        buf_u16(&mut bytes, 0); // fields
        buf_u16(&mut bytes, 0); // methods
        buf_u16(&mut bytes, 1); // class attributes
        let mut content = Vec::new();
        build(&mut content);
        attribute(&mut bytes, a_bootstrap.index, &content);

        let mut budget = budget();
        let facts = class_facts(&bytes, &mut budget).expect("the bootstrap fixture is decodable");
        let shell = find_shell(&facts.attributes, b"BootstrapMethods").clone();
        (bytes, shell, facts.constant_pool)
    }

    /// A structural catalog, not a legal classfile: it carries every constant-pool
    /// tag plus the simple attributes this layer reads, including a `Module`
    /// attribute and bootstrap sites, which no real single class combines. The
    /// reader validates structure, never JVMS co-occurrence rules.
    fn catalog_class() -> (Vec<u8>, Vec<CpRef>) {
        let mut cp = CpBuilder::new();
        let u_catalog = cp.utf8(b"pkg/Catalog");
        let c_catalog = cp.class(u_catalog.index);
        let u_object = cp.utf8(b"java/lang/Object");
        let c_object = cp.class(u_object.index);
        let u_runnable = cp.utf8(b"java/lang/Runnable");
        let c_runnable = cp.class(u_runnable.index);
        let u_field = cp.utf8(b"field");
        let u_int = cp.utf8(b"I");
        let u_const = cp.utf8(b"CONST");
        let u_string = cp.utf8(b"Ljava/lang/String;");
        let s_value = cp.string(u_string.index);
        let i_seven = cp.integer(7);
        let _f_nan = cp.float(0x7fc0_0001);
        let _l_minus_two = cp.long(-2);
        let _d_nan = cp.double(0x7ff8_0000_0000_0001);
        let u_method = cp.utf8(b"method");
        let u_void = cp.utf8(b"()V");
        let u_indy = cp.utf8(b"indy");
        let u_indy_descriptor = cp.utf8(b"()Ljava/lang/Runnable;");
        let nat_indy = cp.name_and_type(u_indy.index, u_indy_descriptor.index);
        let _dyn_indy = cp.dynamic(18, 0, nat_indy.index);
        let u_run = cp.utf8(b"run");
        let nat_run = cp.name_and_type(u_run.index, u_void.index);
        let u_vf = cp.utf8(b"vf");
        let nat_vf = cp.name_and_type(u_vf.index, u_int.index);
        let _fr_vf = cp.member_ref(9, c_object.index, nat_vf.index);
        let mr_run = cp.member_ref(10, c_object.index, nat_run.index);
        let _imr_run = cp.member_ref(11, c_runnable.index, nat_run.index);
        let mh_bootstrap = cp.method_handle(6, mr_run.index);
        let mt_void = cp.method_type(u_void.index);
        let dyn_condy = cp.dynamic(17, 0, nat_run.index);
        let u_package = cp.utf8(b"pkg");
        let package = cp.package(u_package.index);
        let u_module = cp.utf8(b"pkg/module");
        let module = cp.module(u_module.index);
        let a_signature = cp.utf8(b"Signature");
        let u_signature = cp.utf8(b"<T:Ljava/lang/Object;>Ljava/lang/Object;");
        let a_exceptions = cp.utf8(b"Exceptions");
        let u_exception = cp.utf8(b"java/lang/Exception");
        let c_exception = cp.class(u_exception.index);
        let a_inner_classes = cp.utf8(b"InnerClasses");
        let u_inner = cp.utf8(b"pkg/Catalog$Inner");
        let c_inner = cp.class(u_inner.index);
        let u_inner_name = cp.utf8(b"Inner");
        let a_enclosing = cp.utf8(b"EnclosingMethod");
        let a_nest_host = cp.utf8(b"NestHost");
        let u_nest = cp.utf8(b"pkg/Nest");
        let c_nest = cp.class(u_nest.index);
        let a_nest_members = cp.utf8(b"NestMembers");
        let a_permitted = cp.utf8(b"PermittedSubclasses");
        let u_sub = cp.utf8(b"pkg/Sub");
        let c_sub = cp.class(u_sub.index);
        let a_constant_value = cp.utf8(b"ConstantValue");
        let a_module = cp.utf8(b"Module");
        let u_service = cp.utf8(b"pkg/Service");
        let c_service = cp.class(u_service.index);
        let u_provider = cp.utf8(b"pkg/Provider");
        let c_provider = cp.class(u_provider.index);
        let a_bootstrap = cp.utf8(b"BootstrapMethods");
        let (mut bytes, entries) = cp.finish();

        buf_u16(&mut bytes, 0x8021); // ACC_MODULE | ACC_PUBLIC | ACC_SUPER
        buf_u16(&mut bytes, c_catalog.index);
        buf_u16(&mut bytes, c_object.index);
        buf_u16(&mut bytes, 1);
        buf_u16(&mut bytes, c_runnable.index);

        buf_u16(&mut bytes, 2); // fields
        buf_u16(&mut bytes, 0x0002);
        buf_u16(&mut bytes, u_field.index);
        buf_u16(&mut bytes, u_int.index);
        buf_u16(&mut bytes, 0);
        buf_u16(&mut bytes, 0x0018);
        buf_u16(&mut bytes, u_const.index);
        buf_u16(&mut bytes, u_string.index);
        buf_u16(&mut bytes, 1);
        let mut constant_value = Vec::new();
        buf_u16(&mut constant_value, s_value.index);
        attribute(&mut bytes, a_constant_value.index, &constant_value);

        buf_u16(&mut bytes, 1); // methods
        buf_u16(&mut bytes, 0x0401);
        buf_u16(&mut bytes, u_method.index);
        buf_u16(&mut bytes, u_void.index);
        buf_u16(&mut bytes, 2);
        let mut exceptions = Vec::new();
        buf_u16(&mut exceptions, 1);
        buf_u16(&mut exceptions, c_exception.index);
        attribute(&mut bytes, a_exceptions.index, &exceptions);
        let mut signature = Vec::new();
        buf_u16(&mut signature, u_signature.index);
        attribute(&mut bytes, a_signature.index, &signature);

        buf_u16(&mut bytes, 8); // class attributes
        attribute(&mut bytes, a_signature.index, &signature);
        let mut inner_classes = Vec::new();
        buf_u16(&mut inner_classes, 2);
        buf_u16(&mut inner_classes, c_inner.index);
        buf_u16(&mut inner_classes, c_catalog.index);
        buf_u16(&mut inner_classes, u_inner_name.index);
        buf_u16(&mut inner_classes, 0x0009);
        buf_u16(&mut inner_classes, c_catalog.index);
        buf_u16(&mut inner_classes, 0);
        buf_u16(&mut inner_classes, 0);
        buf_u16(&mut inner_classes, 0x1000);
        attribute(&mut bytes, a_inner_classes.index, &inner_classes);
        let mut enclosing = Vec::new();
        buf_u16(&mut enclosing, c_object.index);
        buf_u16(&mut enclosing, nat_run.index);
        attribute(&mut bytes, a_enclosing.index, &enclosing);
        let mut nest_host = Vec::new();
        buf_u16(&mut nest_host, c_nest.index);
        attribute(&mut bytes, a_nest_host.index, &nest_host);
        let mut nest_members = Vec::new();
        buf_u16(&mut nest_members, 1);
        buf_u16(&mut nest_members, c_inner.index);
        attribute(&mut bytes, a_nest_members.index, &nest_members);
        let mut permitted = Vec::new();
        buf_u16(&mut permitted, 1);
        buf_u16(&mut permitted, c_sub.index);
        attribute(&mut bytes, a_permitted.index, &permitted);
        let mut module_content = Vec::new();
        buf_u16(&mut module_content, module.index);
        buf_u16(&mut module_content, 0x0020);
        buf_u16(&mut module_content, 0);
        buf_u16(&mut module_content, 1); // requires
        buf_u16(&mut module_content, module.index);
        buf_u16(&mut module_content, 0x0020);
        buf_u16(&mut module_content, 0);
        buf_u16(&mut module_content, 1); // exports
        buf_u16(&mut module_content, package.index);
        buf_u16(&mut module_content, 0);
        buf_u16(&mut module_content, 1);
        buf_u16(&mut module_content, module.index);
        buf_u16(&mut module_content, 0); // opens
        buf_u16(&mut module_content, 1); // uses
        buf_u16(&mut module_content, c_service.index);
        buf_u16(&mut module_content, 1); // provides
        buf_u16(&mut module_content, c_service.index);
        buf_u16(&mut module_content, 1);
        buf_u16(&mut module_content, c_provider.index);
        attribute(&mut bytes, a_module.index, &module_content);
        let mut bootstrap = Vec::new();
        buf_u16(&mut bootstrap, 1);
        buf_u16(&mut bootstrap, mh_bootstrap.index);
        buf_u16(&mut bootstrap, 4);
        buf_u16(&mut bootstrap, mt_void.index);
        buf_u16(&mut bootstrap, dyn_condy.index);
        buf_u16(&mut bootstrap, s_value.index);
        buf_u16(&mut bootstrap, i_seven.index);
        attribute(&mut bytes, a_bootstrap.index, &bootstrap);

        (bytes, entries)
    }

    #[test]
    fn synthetic_catalog_covers_every_constant_pool_tag_and_its_spans() {
        let (bytes, entries) = catalog_class();
        let mut budget = budget();
        let facts = class_facts(&bytes, &mut budget).unwrap();
        assert_eq!(budget.usage().class_bytes, bytes.len() as u64);
        assert_eq!(budget.usage().result_items, 0);

        // Two reserved slots for Long and Double, no gaps anywhere else.
        assert_eq!(entries.len(), 59);
        assert_eq!(facts.constant_pool.len(), entries.len());
        for (entry, reference) in facts.constant_pool.iter().zip(entries.iter()) {
            assert_eq!(entry.index, reference.index);
            assert_eq!(
                (entry.span.start, entry.span.length),
                (reference.offset, reference.width),
                "entry #{}",
                reference.index
            );
        }
        for pair in facts.constant_pool.windows(2) {
            assert_eq!(pair[0].span.start + pair[0].span.length, pair[1].span.start);
        }
        let lookup = |index: u16| cp_entry(&facts.constant_pool, index).unwrap();
        let utf8 = |index: u16| cp_utf8(&facts.constant_pool, index).unwrap();
        let class_name = |index: u16| cp_class_name(&facts.constant_pool, index).unwrap();
        let index_of = |name: &[u8]| {
            facts
                .constant_pool
                .iter()
                .find(|entry| match &entry.kind {
                    CpEntryKind::Utf8 { bytes } => bytes.0 == name,
                    _ => false,
                })
                .unwrap_or_else(|| panic!("no Utf8 entry for {name:?}"))
                .index
        };

        assert_eq!(facts.access_flags, 0x8021);
        assert_eq!(facts.this_class.raw().0, b"pkg/Catalog");
        assert_eq!(
            facts.super_class.as_ref().unwrap().raw().0,
            b"java/lang/Object"
        );
        assert_eq!(facts.interfaces[0].raw().0, b"java/lang/Runnable");

        let this_class = index_of(b"pkg/Catalog");
        let object = class_entry_for(&facts, b"java/lang/Object");
        let runnable = class_entry_for(&facts, b"java/lang/Runnable");
        let vf = index_of(b"vf");
        let int = index_of(b"I");
        let string = index_of(b"Ljava/lang/String;");
        let package = index_of(b"pkg");
        let module = index_of(b"pkg/module");

        assert_eq!(
            lookup(i_seven_index(&facts)).kind,
            CpEntryKind::Integer { value: 7 }
        );
        assert_eq!(
            lookup(float_index(&facts)).kind,
            CpEntryKind::Float { bits: 0x7fc0_0001 }
        );
        assert_eq!(
            lookup(long_index(&facts)).kind,
            CpEntryKind::Long { value: -2 }
        );
        assert_eq!(
            lookup(double_index(&facts)).kind,
            CpEntryKind::Double {
                bits: 0x7ff8_0000_0000_0001
            }
        );
        assert_eq!(
            lookup(string_entry(&facts)).kind,
            CpEntryKind::String {
                utf8_index: string,
                value: JvmBytes(b"Ljava/lang/String;".to_vec()),
            }
        );
        assert_eq!(
            lookup(class_entry_for(&facts, b"pkg/Catalog")).kind,
            CpEntryKind::Class {
                name_index: this_class,
                name: JvmBytes(b"pkg/Catalog".to_vec()),
            }
        );
        assert_eq!(
            lookup(field_ref_entry(&facts)).kind,
            CpEntryKind::FieldRef {
                class_index: object,
                name_and_type_index: nat_vf_index(&facts),
                owner: JvmBytes(b"java/lang/Object".to_vec()),
                name: JvmBytes(b"vf".to_vec()),
                descriptor: JvmBytes(b"I".to_vec()),
            }
        );
        assert_eq!(
            lookup(method_ref_entry(&facts)).kind,
            CpEntryKind::MethodRef {
                class_index: object,
                name_and_type_index: nat_run_index(&facts),
                owner: JvmBytes(b"java/lang/Object".to_vec()),
                name: JvmBytes(b"run".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            }
        );
        assert_eq!(
            lookup(interface_method_ref_entry(&facts)).kind,
            CpEntryKind::InterfaceMethodRef {
                class_index: runnable,
                name_and_type_index: nat_run_index(&facts),
                owner: JvmBytes(b"java/lang/Runnable".to_vec()),
                name: JvmBytes(b"run".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            }
        );
        assert_eq!(
            lookup(nat_vf_index(&facts)).kind,
            CpEntryKind::NameAndType {
                name_index: vf,
                descriptor_index: int,
                name: JvmBytes(b"vf".to_vec()),
                descriptor: JvmBytes(b"I".to_vec()),
            }
        );
        assert_eq!(
            lookup(method_handle_entry(&facts)).kind,
            CpEntryKind::MethodHandle {
                reference_kind: 6,
                reference_index: method_ref_entry(&facts),
            }
        );
        assert_eq!(
            lookup(method_type_entry(&facts)).kind,
            CpEntryKind::MethodType {
                descriptor_index: index_of_utf8(&facts, b"()V"),
                descriptor: JvmBytes(b"()V".to_vec()),
            }
        );
        assert_eq!(
            lookup(dynamic_entry(&facts)).kind,
            CpEntryKind::Dynamic {
                bootstrap_method_attr_index: 0,
                name_and_type_index: nat_run_index(&facts),
                name: JvmBytes(b"run".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            }
        );
        assert_eq!(
            lookup(invoke_dynamic_entry(&facts)).kind,
            CpEntryKind::InvokeDynamic {
                bootstrap_method_attr_index: 0,
                name_and_type_index: nat_indy_index(&facts),
                name: JvmBytes(b"indy".to_vec()),
                descriptor: JvmBytes(b"()Ljava/lang/Runnable;".to_vec()),
            }
        );
        assert_eq!(
            lookup(module_entry(&facts)).kind,
            CpEntryKind::Module {
                name_index: module,
                name: JvmBytes(b"pkg/module".to_vec()),
            }
        );
        assert_eq!(
            lookup(package_entry(&facts)).kind,
            CpEntryKind::Package {
                name_index: package,
                name: JvmBytes(b"pkg".to_vec()),
            }
        );

        // Reserved slots hold no entry, so an index into one is not resolvable.
        assert_eq!(
            error_code(cp_entry(&facts.constant_pool, l_minus_two_slot(&facts)).unwrap_err()),
            "classfile_invalid_constant_pool_index"
        );

        // Class-index expansion goes through `CONSTANT_Class`, and the UTF-8 payload
        // is reachable by its own index; the two must agree on the same bytes.
        for name in [
            b"pkg/Catalog".as_slice(),
            b"java/lang/Object".as_slice(),
            b"java/lang/Runnable".as_slice(),
            b"pkg/Catalog$Inner".as_slice(),
            b"pkg/Nest".as_slice(),
            b"pkg/Sub".as_slice(),
            b"java/lang/Exception".as_slice(),
            b"pkg/Service".as_slice(),
            b"pkg/Provider".as_slice(),
        ] {
            assert_eq!(
                class_name(class_entry_for(&facts, name)),
                JvmBytes(name.to_vec()),
                "{name:?}"
            );
            assert_eq!(
                utf8(index_of_utf8(&facts, name)),
                JvmBytes(name.to_vec()),
                "{name:?}"
            );
        }
    }

    #[test]
    fn synthetic_catalog_member_and_class_attributes_are_typed() {
        let (bytes, _) = catalog_class();
        let mut budget = budget();
        let facts = class_facts(&bytes, &mut budget).unwrap();

        // A field without attributes yields no facts, and an unrelated attribute
        // (`Code`) is never read.
        assert_eq!(
            attribute_facts(
                &bytes,
                &facts.fields[0].attributes,
                &facts.constant_pool,
                &mut budget
            )
            .unwrap(),
            AttributeFacts::default()
        );

        let field = attribute_facts(
            &bytes,
            &facts.fields[1].attributes,
            &facts.constant_pool,
            &mut budget,
        )
        .unwrap();
        assert_eq!(field.constant_value, Some(CpIndexOf(string_entry(&facts))));
        assert_eq!(field.signature, None);
        assert!(field.exceptions.is_empty());
        assert!(field.module.is_none());

        let method = attribute_facts(
            &bytes,
            &facts.methods[0].attributes,
            &facts.constant_pool,
            &mut budget,
        )
        .unwrap();
        assert_eq!(
            method.exceptions,
            vec![JvmBytes(b"java/lang/Exception".to_vec())]
        );
        assert_eq!(
            method.signature,
            Some(JvmBytes(
                b"<T:Ljava/lang/Object;>Ljava/lang/Object;".to_vec()
            ))
        );
        assert_eq!(method.constant_value, None);

        let class =
            attribute_facts(&bytes, &facts.attributes, &facts.constant_pool, &mut budget).unwrap();
        assert_eq!(
            class.signature,
            Some(JvmBytes(
                b"<T:Ljava/lang/Object;>Ljava/lang/Object;".to_vec()
            ))
        );
        assert_eq!(class.nest_host, Some(JvmBytes(b"pkg/Nest".to_vec())));
        assert_eq!(
            class.nest_members,
            vec![JvmBytes(b"pkg/Catalog$Inner".to_vec())]
        );
        assert_eq!(
            class.permitted_subclasses,
            vec![JvmBytes(b"pkg/Sub".to_vec())]
        );
        assert!(class.exceptions.is_empty());
        assert_eq!(class.inner_classes.len(), 2);
        assert_eq!(
            class.inner_classes[0],
            InnerClassFacts {
                class_index: class_entry_for(&facts, b"pkg/Catalog$Inner"),
                outer_class_index: class_entry_for(&facts, b"pkg/Catalog"),
                inner_name: Some(JvmBytes(b"Inner".to_vec())),
                access_flags: 0x0009,
            }
        );
        // `inner_name_index` 0 is the anonymous case and not a missing fact.
        assert_eq!(
            class.inner_classes[1],
            InnerClassFacts {
                class_index: class_entry_for(&facts, b"pkg/Catalog"),
                outer_class_index: 0,
                inner_name: None,
                access_flags: 0x1000,
            }
        );
        assert_eq!(
            class.enclosing_method,
            Some(EnclosingMethodFacts {
                class_index: class_entry_for(&facts, b"java/lang/Object"),
                method_index: nat_run_index(&facts),
            })
        );
        assert_eq!(class.constant_value, None);
        let module = class.module.expect("module facts");
        assert_eq!(module.uses, vec![JvmBytes(b"pkg/Service".to_vec())]);
        assert_eq!(
            module.provides,
            vec![ProvidesFacts {
                service: JvmBytes(b"pkg/Service".to_vec()),
                implementations: vec![JvmBytes(b"pkg/Provider".to_vec())],
            }]
        );
    }

    #[test]
    fn bootstrap_methods_need_a_handle_and_loadable_arguments() {
        let (bytes, _) = catalog_class();
        let mut budget = budget();
        let facts = class_facts(&bytes, &mut budget).unwrap();
        let shell = find_shell(&facts.attributes, b"BootstrapMethods").clone();
        let parsed = bootstrap_methods(&bytes, &shell, &facts.constant_pool, &mut budget).unwrap();
        assert_eq!(parsed.len(), 1);
        let handle = bootstrap_handle_index(&facts);
        assert_eq!(parsed[0].method_ref, handle);
        assert_eq!(
            parsed[0].arguments,
            vec![
                method_type_entry(&facts),
                dynamic_entry(&facts),
                string_entry(&facts),
                i_seven_index(&facts),
            ]
        );
        assert_eq!(
            cp_entry(&facts.constant_pool, handle).unwrap().kind,
            CpEntryKind::MethodHandle {
                reference_kind: 6,
                reference_index: method_ref_entry(&facts),
            }
        );

        // A bootstrap handle that is not a MethodHandle, and an argument that is
        // not a loadable constant, are structured errors.
        let bad_handle = bootstrap_class(|content| {
            buf_u16(content, 1);
            buf_u16(content, method_type_index());
            buf_u16(content, 0);
        });
        assert_eq!(
            error_code(
                bootstrap_methods(
                    &bad_handle.0,
                    &bad_handle.1,
                    &bad_handle.2,
                    &mut fresh_budget()
                )
                .unwrap_err()
            ),
            "classfile_constant_pool_tag_mismatch"
        );

        let bad_argument = bootstrap_class(|content| {
            buf_u16(content, 1);
            buf_u16(content, 4);
            buf_u16(content, 1);
            buf_u16(content, 6);
        });
        assert_eq!(
            error_code(
                bootstrap_methods(
                    &bad_argument.0,
                    &bad_argument.1,
                    &bad_argument.2,
                    &mut fresh_budget()
                )
                .unwrap_err()
            ),
            "classfile_constant_pool_tag_mismatch"
        );

        // Content that declares more arguments than it holds stops structured.
        let overrun = bootstrap_class(|content| {
            buf_u16(content, 1);
            buf_u16(content, 4);
            buf_u16(content, 2);
            buf_u16(content, 8);
        });
        assert_eq!(
            error_code(
                bootstrap_methods(&overrun.0, &overrun.1, &overrun.2, &mut fresh_budget())
                    .unwrap_err()
            ),
            "classfile_invalid_attribute_content"
        );

        // Trailing garbage behind a complete structure is refused as well.
        let trailing = bootstrap_class(|content| {
            buf_u16(content, 1);
            buf_u16(content, 4);
            buf_u16(content, 0);
            content.push(0);
        });
        assert_eq!(
            error_code(
                bootstrap_methods(&trailing.0, &trailing.1, &trailing.2, &mut fresh_budget())
                    .unwrap_err()
            ),
            "classfile_invalid_attribute_content"
        );
    }

    #[test]
    fn duplicate_attributes_and_attribute_structure_overruns_are_errors() {
        let (mut bytes, _) = catalog_class();
        let mut budget = budget();
        let facts = class_facts(&bytes, &mut budget).unwrap();
        let signature_shell = find_shell(&facts.attributes, b"Signature").clone();
        assert_eq!(
            error_code(
                attribute_facts(
                    &bytes,
                    &[signature_shell.clone(), signature_shell.clone()],
                    &facts.constant_pool,
                    &mut budget
                )
                .unwrap_err()
            ),
            "classfile_duplicate_attribute"
        );

        // An `Exceptions` attribute that declares more entries than it holds.
        let exceptions_name = index_of_utf8(&facts, b"Exceptions");
        let mut short_content = Vec::new();
        buf_u16(&mut short_content, 2);
        buf_u16(
            &mut short_content,
            class_entry_for(&facts, b"java/lang/Exception"),
        );
        let start = bytes.len();
        attribute(&mut bytes, exceptions_name, &short_content);
        let short_shell = AttributeShell {
            name: signature_shell.name.clone(),
            span: ByteSpan::new(start as u64, (6 + short_content.len()) as u64),
            content_span: ByteSpan::new((start + 6) as u64, short_content.len() as u64),
        };
        assert_eq!(
            error_code(
                attribute_facts(&bytes, &[short_shell], &facts.constant_pool, &mut budget)
                    .unwrap_err()
            ),
            "classfile_invalid_attribute_content"
        );
    }

    #[test]
    fn layout_verification_rejects_a_mismatched_pool() {
        let (bytes, _) = catalog_class();
        let layout = measure_constant_pool(&bytes, &Budget::new(unlimited_limits())).unwrap();
        let class = Class::new(&bytes).unwrap();
        assert!(verify_constant_pool_layout(&bytes, &layout, &class).is_ok());

        // The same class must reproduce the same layout, so the check is not
        // vacuous.
        let (other, _) = catalog_class();
        let other_layout = measure_constant_pool(&other, &Budget::new(unlimited_limits())).unwrap();
        assert!(verify_constant_pool_layout(&bytes, &other_layout, &class).is_ok());

        // A layout measured from a different pool, and a pool whose bytes no longer
        // match the tags, both stop instead of producing spans that merely look
        // plausible.
        let small = bootstrap_class(|content| {
            buf_u16(content, 0);
        });
        let small_layout =
            measure_constant_pool(&small.0, &Budget::new(unlimited_limits())).unwrap();
        assert!(verify_constant_pool_layout(&bytes, &small_layout, &class).is_err());

        // Retagging an entry changes the width of the following entries: the walk
        // reports a structured error instead of slicing plausible-looking bytes.
        let mut retagged = bytes.clone();
        retagged[10] = 8; // slot 1 is a Utf8 entry; `CONSTANT_String` is narrower
        assert!(matches!(
            measure_constant_pool(&retagged, &Budget::new(unlimited_limits())),
            Err(Error::InvalidInput { .. })
        ));
        assert!(matches!(
            class_facts(&retagged, &mut budget()),
            Err(Error::InvalidInput { .. })
        ));
    }

    #[test]
    fn truncated_module_attribute_is_a_structured_error() {
        let (bytes, _) = catalog_class();
        let mut budget = budget();
        let facts = class_facts(&bytes, &mut budget).unwrap();

        // The cursor stops exactly at the end of a well-formed Module attribute, so
        // a truncated Module attribute is a structured error, not an empty one.
        let shell = find_shell(&facts.attributes, b"Module").clone();
        let truncated = AttributeShell {
            name: shell.name.clone(),
            span: ByteSpan::new(shell.span.start, shell.span.length - 1),
            content_span: ByteSpan::new(shell.content_span.start, shell.content_span.length - 1),
        };
        assert_eq!(
            error_code(
                attribute_facts(&bytes, &[truncated], &facts.constant_pool, &mut budget)
                    .unwrap_err()
            ),
            "classfile_invalid_attribute_content"
        );
    }
}

/// Test-only class-file builder shared by the reader, raw-CFG and call-context unit tests.
///
/// It builds the smallest class the real reader path accepts: `Test.method()V` at a chosen
/// class-file version whose `Code` attribute holds the caller's bytes verbatim. The constant
/// pool also carries the entries an array-creation, interface-call or initialization fixture
/// names — the class `[[I` at slot 9, the interface method `run(J)V` at slot 15, and the
/// constructor calls `Test.<init>()V` at slot 18 and `java/lang/Object.<init>()V` at slot 19 — so
/// a fixture can point at a real entry instead of a placeholder. Nothing here fabricates facts:
/// the tests that use it read these bytes through `class_facts`/`method_code_facts`.
///
/// The gate is a feature as well as `test` because this builder is shared across module
/// boundaries that become crate boundaries: the raw-CFG and call-context tests move to
/// `jarde-jvm`, whose `cfg(test)` cannot see anything here. Enabling `test-support` from a
/// dev-dependency keeps this builder reachable there without putting it in the production API —
/// the feature is never enabled by a normal dependency, and nothing in the module is `pub` in
/// the package sense. Note that `--all-features` (as CI runs) does enable it, so the module must
/// keep compiling with the warning set on.
#[cfg(any(test, feature = "test-support"))]
#[allow(
    dead_code,
    reason = "used by this crate's tests, or by a dependent crate's test build"
)]
pub mod test_class {
    /// Class file (minor 0, major `major`) with one `method()V` whose body is `code`.
    pub fn single_method(major: u16, max_stack: u16, max_locals: u16, code: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, major);
        u16_be(&mut bytes, 20);
        utf8(&mut bytes, b"Test"); // 1
        class(&mut bytes, 1); // 2
        utf8(&mut bytes, b"java/lang/Object"); // 3
        class(&mut bytes, 3); // 4
        utf8(&mut bytes, b"method"); // 5
        utf8(&mut bytes, b"()V"); // 6
        utf8(&mut bytes, b"Code"); // 7
        utf8(&mut bytes, b"[[I"); // 8
        class(&mut bytes, 8); // 9
        utf8(&mut bytes, b"java/lang/Runnable"); // 10
        class(&mut bytes, 10); // 11
        utf8(&mut bytes, b"run"); // 12
        utf8(&mut bytes, b"(J)V"); // 13
        bytes.push(12); // 14: NameAndType run:(J)V
        u16_be(&mut bytes, 12);
        u16_be(&mut bytes, 13);
        bytes.push(11); // 15: InterfaceMethodRef java/lang/Runnable.run:(J)V
        u16_be(&mut bytes, 11);
        u16_be(&mut bytes, 14);
        // 16..19: the constructor calls the frame pass's initialization cases name — the class's
        // own `Test.<init>()V` and the superclass's `java/lang/Object.<init>()V`, which share one
        // name-and-type. They are appended **after** every entry above, so the fixtures that point
        // at 2, 9, 11 or 15 keep pointing at what they name.
        utf8(&mut bytes, b"<init>"); // 16
        bytes.push(12); // 17: NameAndType <init>:()V
        u16_be(&mut bytes, 16);
        u16_be(&mut bytes, 6);
        bytes.push(10); // 18: MethodRef Test.<init>:()V
        u16_be(&mut bytes, 2);
        u16_be(&mut bytes, 17);
        bytes.push(10); // 19: MethodRef java/lang/Object.<init>:()V
        u16_be(&mut bytes, 4);
        u16_be(&mut bytes, 17);
        u16_be(&mut bytes, 0x0021);
        u16_be(&mut bytes, 2);
        u16_be(&mut bytes, 4);
        u16_be(&mut bytes, 0); // interfaces
        u16_be(&mut bytes, 0); // fields
        u16_be(&mut bytes, 1); // methods
        u16_be(&mut bytes, 0x0009);
        u16_be(&mut bytes, 5);
        u16_be(&mut bytes, 6);
        u16_be(&mut bytes, 1); // attributes
        let mut content = Vec::new();
        u16_be(&mut content, max_stack);
        u16_be(&mut content, max_locals);
        content.extend_from_slice(
            &u32::try_from(code.len())
                .expect("fixture code fits u32")
                .to_be_bytes(),
        );
        content.extend_from_slice(code);
        u16_be(&mut content, 0); // exception table
        u16_be(&mut content, 0); // Code attributes
        u16_be(&mut bytes, 7);
        bytes.extend_from_slice(
            &u32::try_from(content.len())
                .expect("fixture attribute fits u32")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(&content);
        u16_be(&mut bytes, 0); // class attributes
        bytes
    }

    fn u16_be(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn utf8(bytes: &mut Vec<u8>, value: &[u8]) {
        bytes.push(1);
        u16_be(
            bytes,
            u16::try_from(value.len()).expect("fixture name fits u16"),
        );
        bytes.extend_from_slice(value);
    }

    fn class(bytes: &mut Vec<u8>, name: u16) {
        bytes.push(7);
        u16_be(bytes, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::{BudgetDimension, CancellationToken, Limits};
    use crate::release_registry::ClassfileLocation;
    use proptest::prelude::*;

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
            ..Limits::default()
        }
    }

    fn u16_be(output: &mut Vec<u8>, value: u16) {
        output.extend_from_slice(&value.to_be_bytes());
    }

    fn u32_be(output: &mut Vec<u8>, value: u32) {
        output.extend_from_slice(&value.to_be_bytes());
    }

    fn utf8(output: &mut Vec<u8>, value: &[u8]) {
        output.push(1);
        u16_be(output, u16::try_from(value.len()).unwrap());
        output.extend_from_slice(value);
    }

    fn class_entry(output: &mut Vec<u8>, name: u16) {
        output.push(7);
        u16_be(output, name);
    }

    fn attribute(output: &mut Vec<u8>, name: u16, content: &[u8]) -> (ByteSpan, ByteSpan) {
        attribute_with_length_offset(output, name, content).0
    }

    fn attribute_with_length_offset(
        output: &mut Vec<u8>,
        name: u16,
        content: &[u8],
    ) -> ((ByteSpan, ByteSpan), usize) {
        let start = output.len();
        u16_be(output, name);
        let length_offset = output.len();
        u32_be(output, u32::try_from(content.len()).unwrap());
        let content_start = output.len();
        output.extend_from_slice(content);
        (
            (
                ByteSpan::new(
                    start as u64,
                    (ATTRIBUTE_HEADER_LENGTH + content.len()) as u64,
                ),
                ByteSpan::new(content_start as u64, content.len() as u64),
            ),
            length_offset,
        )
    }

    struct Fixture {
        bytes: Vec<u8>,
        field_attribute: (ByteSpan, ByteSpan),
        method_attribute: (ByteSpan, ByteSpan),
        code_attribute: (ByteSpan, ByteSpan),
        class_attribute: (ByteSpan, ByteSpan),
        attribute_length_offsets: [usize; 5],
    }

    fn fixture() -> Fixture {
        let special_name = b"A\xc0\x80\xed\xa0\xbd\xed\xb8\x80\xed\xa0\x80\\\"\xe2\x80\xae";
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 52);
        u16_be(&mut bytes, 15);
        utf8(&mut bytes, special_name); // 1
        class_entry(&mut bytes, 1); // 2
        utf8(&mut bytes, b"java/lang/Object"); // 3
        class_entry(&mut bytes, 3); // 4
        utf8(&mut bytes, b"field"); // 5
        utf8(&mut bytes, b"I"); // 6
        utf8(&mut bytes, b"method"); // 7
        utf8(&mut bytes, b"()V"); // 8
        utf8(&mut bytes, b"ClassUnknown"); // 9
        utf8(&mut bytes, b"FieldUnknown"); // 10
        utf8(&mut bytes, b"Code"); // 11
        utf8(&mut bytes, b"pkg/Interface"); // 12
        class_entry(&mut bytes, 12); // 13
        utf8(&mut bytes, b"MethodUnknown"); // 14
        u16_be(&mut bytes, 0x0021);
        u16_be(&mut bytes, 2);
        u16_be(&mut bytes, 4);
        u16_be(&mut bytes, 1);
        u16_be(&mut bytes, 13);
        u16_be(&mut bytes, 1);
        u16_be(&mut bytes, 0x0001);
        u16_be(&mut bytes, 5);
        u16_be(&mut bytes, 6);
        u16_be(&mut bytes, 1);
        let (field_attribute, field_attribute_length_offset) =
            attribute_with_length_offset(&mut bytes, 10, &[0xde, 0xad]);
        u16_be(&mut bytes, 1);
        u16_be(&mut bytes, 0x0009);
        u16_be(&mut bytes, 7);
        u16_be(&mut bytes, 8);
        u16_be(&mut bytes, 2);
        let (method_attribute, method_attribute_length_offset) =
            attribute_with_length_offset(&mut bytes, 14, &[0xff]);
        let mut code_content = Vec::new();
        u16_be(&mut code_content, 1);
        u16_be(&mut code_content, 1);
        u32_be(&mut code_content, 1);
        code_content.push(0xb1);
        u16_be(&mut code_content, 0);
        u16_be(&mut code_content, 1);
        let (_, nested_attribute_length_offset_in_content) =
            attribute_with_length_offset(&mut code_content, 14, &[0xca, 0xfe]);
        let (code_attribute, code_attribute_length_offset) =
            attribute_with_length_offset(&mut bytes, 11, &code_content);
        let nested_attribute_length_offset = usize::try_from(code_attribute.1.start).unwrap()
            + nested_attribute_length_offset_in_content;
        u16_be(&mut bytes, 1);
        let (class_attribute, class_attribute_length_offset) =
            attribute_with_length_offset(&mut bytes, 9, &[1, 2, 3]);
        Fixture {
            bytes,
            field_attribute,
            method_attribute,
            code_attribute,
            class_attribute,
            attribute_length_offsets: [
                class_attribute_length_offset,
                field_attribute_length_offset,
                method_attribute_length_offset,
                code_attribute_length_offset,
                nested_attribute_length_offset,
            ],
        }
    }

    fn bytecode_fixture(
        major: u16,
        minor: u16,
        code: Option<&[u8]>,
        handlers: &[(u16, u16, u16, u16)],
        duplicate_method: bool,
        duplicate_code: bool,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
        u16_be(&mut bytes, minor);
        u16_be(&mut bytes, major);
        u16_be(&mut bytes, 9);
        utf8(&mut bytes, b"Test"); // 1
        class_entry(&mut bytes, 1); // 2
        utf8(&mut bytes, b"java/lang/Object"); // 3
        class_entry(&mut bytes, 3); // 4
        utf8(&mut bytes, b"method"); // 5
        utf8(&mut bytes, b"()V"); // 6
        utf8(&mut bytes, b"Code"); // 7
        utf8(&mut bytes, b"Noise"); // 8
        u16_be(&mut bytes, 0x0021);
        u16_be(&mut bytes, 2);
        u16_be(&mut bytes, 4);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, if duplicate_method { 2 } else { 1 });
        for _ in 0..if duplicate_method { 2 } else { 1 } {
            u16_be(&mut bytes, 0x0009);
            u16_be(&mut bytes, 5);
            u16_be(&mut bytes, 6);
            let attributes = if code.is_none() {
                0
            } else if duplicate_code {
                2
            } else {
                1
            };
            u16_be(&mut bytes, attributes);
            for _ in 0..attributes {
                let mut content = Vec::new();
                u16_be(&mut content, 4);
                u16_be(&mut content, 3);
                u32_be(&mut content, code.unwrap().len() as u32);
                content.extend_from_slice(code.unwrap());
                u16_be(&mut content, handlers.len() as u16);
                for &(start, end, handler, catch) in handlers {
                    u16_be(&mut content, start);
                    u16_be(&mut content, end);
                    u16_be(&mut content, handler);
                    u16_be(&mut content, catch);
                }
                u16_be(&mut content, 0);
                attribute(&mut bytes, 7, &content);
            }
        }
        u16_be(&mut bytes, 1);
        attribute(&mut bytes, 8, &[0u8; 64]);
        bytes
    }

    fn selector() -> MethodSelector {
        MethodSelector {
            name: JvmBytes(b"method".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        }
    }

    fn mutf8(units: &[u16]) -> Vec<u8> {
        let mut output = Vec::new();
        for &unit in units {
            match unit {
                0 => output.extend_from_slice(&[0xc0, 0x80]),
                1..=0x7f => output.push(unit as u8),
                0x80..=0x7ff => {
                    output.push(0xc0 | ((unit >> 6) as u8));
                    output.push(0x80 | ((unit & 0x3f) as u8));
                }
                _ => {
                    output.push(0xe0 | ((unit >> 12) as u8));
                    output.push(0x80 | (((unit >> 6) & 0x3f) as u8));
                    output.push(0x80 | ((unit & 0x3f) as u8));
                }
            }
        }
        output
    }

    fn descriptor_header_fixture(descriptor: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 52);
        u16_be(&mut bytes, 7);
        utf8(&mut bytes, b"Test"); // 1
        class_entry(&mut bytes, 1); // 2
        utf8(&mut bytes, b"java/lang/Object"); // 3
        class_entry(&mut bytes, 3); // 4
        utf8(&mut bytes, b"method"); // 5
        utf8(&mut bytes, descriptor); // 6
        u16_be(&mut bytes, 0x0021);
        u16_be(&mut bytes, 2);
        u16_be(&mut bytes, 4);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 1);
        u16_be(&mut bytes, 0x0001);
        u16_be(&mut bytes, 5);
        u16_be(&mut bytes, 6);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 0);
        bytes
    }

    fn fixture_version(major: u16, minor: u16) -> Fixture {
        let mut fixture = fixture();
        fixture.bytes[4..6].copy_from_slice(&minor.to_be_bytes());
        fixture.bytes[6..8].copy_from_slice(&major.to_be_bytes());
        fixture
    }

    fn slice<'a>(bytes: &'a [u8], span: &ByteSpan) -> &'a [u8] {
        let start = usize::try_from(span.start).unwrap();
        let end = start + usize::try_from(span.length).unwrap();
        &bytes[start..end]
    }

    fn header_attribute_shells(header: &ClassHeader) -> impl Iterator<Item = &AttributeShell> {
        header
            .attributes
            .iter()
            .chain(header.fields.iter().flat_map(|field| &field.attributes))
            .chain(header.methods.iter().flat_map(|method| &method.attributes))
    }

    fn assert_attribute_shells_within(header: &ClassHeader, input: &[u8]) {
        let input_len = u64::try_from(input.len()).unwrap();
        for shell in header_attribute_shells(header) {
            let span_end = shell.span.start.checked_add(shell.span.length).unwrap();
            let content_end = shell
                .content_span
                .start
                .checked_add(shell.content_span.length)
                .unwrap();
            assert!(span_end <= input_len);
            assert!(content_end <= input_len);
            assert_eq!(shell.content_span.start, shell.span.start + 6);
            assert_eq!(shell.span.length, shell.content_span.length + 6);
            assert_eq!(span_end, content_end);
            let length_offset = usize::try_from(shell.span.start).unwrap() + 2;
            assert_eq!(
                u64::from(read_u32(input, length_offset).unwrap()),
                shell.content_span.length
            );
        }
        let hosts = std::iter::once(&header.attributes)
            .chain(header.fields.iter().map(|field| &field.attributes))
            .chain(header.methods.iter().map(|method| &method.attributes));
        for shells in hosts {
            for pair in shells.windows(2) {
                assert!(pair[0].span.start + pair[0].span.length <= pair[1].span.start);
            }
        }
    }

    fn assert_structured_header_result(result: Result<HeaderInspection>, input: &[u8]) {
        match result {
            Ok(inspection) => assert_attribute_shells_within(&inspection.header, input),
            Err(Error::InvalidInput { .. }) => {}
            Err(other) => panic!("unexpected forensic header result: {other}"),
        }
    }

    /// Asserts a report's usage equals a fresh read of the budget, wall clock aside.
    ///
    /// `UsageSnapshot::elapsed_millis` is recomputed from the clock on every read, so two reads
    /// of an unchanged budget differ whenever a millisecond falls between them — which is what
    /// made a property run fail intermittently. Zeroing that one field is this repository's
    /// convention for comparing a snapshot read twice; every counted dimension still has to
    /// agree, so the assertion keeps its teeth.
    fn assert_usage_matches_budget(usage: &crate::budget::UsageSnapshot, budget: &Budget) {
        let mut reported = usage.clone();
        let mut live = budget.usage();
        reported.elapsed_millis = 0;
        live.elapsed_millis = 0;
        assert_eq!(
            reported, live,
            "the report's usage is a faithful read of the budget"
        );
    }

    fn assert_bytecode_report_invariants(
        report: &BytecodeInspection,
        bytes: &[u8],
        code: &[u8],
        budget: &Budget,
    ) {
        let code_length = u32::try_from(code.len()).unwrap();
        assert!(report.exception_handlers.is_empty());
        assert_eq!(report.exception_handler_count, 0);
        assert_eq!(report.verification, VerificationStatus::NotPerformed);
        assert_eq!(report.code_span.length, u64::from(code_length));

        let mut prefix_end = 0u32;
        for fact in &report.instructions {
            assert_eq!(fact.bci, prefix_end);
            assert!(fact.width > 0);
            assert_eq!(
                fact.span.start,
                report.code_span.start + u64::from(fact.bci)
            );
            assert_eq!(fact.span.length, u64::from(fact.width));
            assert_eq!(fact.operands_span.start, fact.span.start + 1);
            assert_eq!(fact.operands_span.length, u64::from(fact.width - 1));
            prefix_end = prefix_end.checked_add(fact.width).unwrap();
            assert!(prefix_end <= code_length);
            assert_eq!(fact.opcode, code[fact.bci as usize]);
            assert_eq!(
                slice(bytes, &fact.span),
                &code[fact.bci as usize..prefix_end as usize]
            );
        }

        match &report.execution {
            ExecutionReport::Complete { usage } => {
                assert_eq!(prefix_end, code_length);
                assert!(report.stopped_at.is_none());
                assert!(report.diagnostics.is_empty());
                assert_usage_matches_budget(usage, budget);
            }
            ExecutionReport::Partial { reason, usage } => {
                let TerminationReason::Error { code: reason_code } = reason else {
                    panic!("unlimited instruction failure must have an error reason");
                };
                let Some(BytecodeStop::Instructions {
                    bci,
                    class_offset,
                    code: stop_code,
                }) = report.stopped_at.as_ref()
                else {
                    panic!("partial instruction decode must report an instruction stop");
                };
                assert_eq!(*bci, prefix_end);
                assert_eq!(*class_offset, report.code_span.start + u64::from(*bci));
                assert_eq!(report.diagnostics.len(), 1);
                assert_eq!(reason_code, stop_code);
                assert_eq!(reason_code, &report.diagnostics[0].code);
                assert_eq!(report.diagnostics[0].severity, DiagnosticSeverity::Error);
                assert_usage_matches_budget(usage, budget);
            }
            ExecutionReport::Cancelled { .. } | ExecutionReport::Failed { .. } => {
                panic!("unlimited bytecode property must complete or stop on an instruction error")
            }
        }
        let usage = budget.usage();
        assert_eq!(usage.class_bytes, bytes.len() as u64);
        assert_eq!(usage.code_bytes, u64::from(prefix_end));
        assert_eq!(usage.result_items, 1 + report.instructions.len() as u64);
        let shell_start = usize::try_from(report.code_span.start).unwrap() - 14;
        let declared_length = u64::from(read_u32(bytes, shell_start + 2).unwrap());
        assert_eq!(
            usage.attribute_bytes,
            declared_length.checked_add(6).unwrap()
        );
    }

    fn instruction_stop(stop: &BytecodeStop) -> (u32, u64, &str) {
        match stop {
            BytecodeStop::Instructions {
                bci,
                class_offset,
                code,
            } => (*bci, *class_offset, code),
            BytecodeStop::ExceptionHandlers { .. } => panic!("expected instruction stop"),
        }
    }

    fn handler_stop(stop: &BytecodeStop) -> (u32, u64, &str) {
        match stop {
            BytecodeStop::ExceptionHandlers {
                ordinal,
                class_offset,
                code,
            } => (*ordinal, *class_offset, code),
            BytecodeStop::Instructions { .. } => panic!("expected exception-handler stop"),
        }
    }

    #[test]
    fn all_header_prefixes_are_panic_free_and_only_complete_input_succeeds() {
        let fixture = fixture();
        for prefix_len in 0..=fixture.bytes.len() {
            let prefix = &fixture.bytes[..prefix_len];
            let result = inspect_header(
                prefix,
                &mut Budget::new(limits(u64::MAX)),
                InspectionMode::Forensic,
            );
            assert_eq!(
                result.is_ok(),
                prefix_len == fixture.bytes.len(),
                "unexpected result for prefix {prefix_len}/{}",
                fixture.bytes.len()
            );
            assert_structured_header_result(result, prefix);
        }
    }

    #[test]
    fn oversized_attribute_length_is_rejected_at_the_owner_but_nested_code_stays_lazy() {
        let fixture = fixture();
        for (index, length_offset) in fixture.attribute_length_offsets.into_iter().enumerate() {
            let mut bytes = fixture.bytes.clone();
            bytes[length_offset..length_offset + 4].copy_from_slice(&u32::MAX.to_be_bytes());
            let header = inspect_header(
                &bytes,
                &mut Budget::new(limits(u64::MAX)),
                InspectionMode::Forensic,
            );
            if index == 4 {
                assert!(header.is_ok());
                assert!(matches!(
                    inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX)),),
                    Err(Error::InvalidInput { .. })
                ));
            } else {
                assert!(matches!(header, Err(Error::InvalidInput { .. })));
            }
        }
    }

    #[test]
    fn oversized_and_truncated_switch_declarations_stop_without_expansion() {
        let mut table_negative = vec![0xaa, 0, 0, 0];
        table_negative.extend_from_slice(&0i32.to_be_bytes());
        table_negative.extend_from_slice(&1i32.to_be_bytes());
        table_negative.extend_from_slice(&0i32.to_be_bytes());

        let mut table_huge = vec![0xaa, 0, 0, 0];
        table_huge.extend_from_slice(&0i32.to_be_bytes());
        table_huge.extend_from_slice(&i32::MIN.to_be_bytes());
        table_huge.extend_from_slice(&i32::MAX.to_be_bytes());

        let mut lookup_negative = vec![0xab, 0, 0, 0];
        lookup_negative.extend_from_slice(&0i32.to_be_bytes());
        lookup_negative.extend_from_slice(&(-1i32).to_be_bytes());

        let mut lookup_huge = vec![0xab, 0, 0, 0];
        lookup_huge.extend_from_slice(&0i32.to_be_bytes());
        lookup_huge.extend_from_slice(&i32::MAX.to_be_bytes());

        let cases = [
            (table_negative, &[0usize, 1, 4, 8, 12][..]),
            (table_huge, &[0usize, 1, 4, 8, 12][..]),
            (lookup_negative, &[0usize, 1, 4, 8][..]),
            (lookup_huge, &[0usize, 1, 4, 8][..]),
        ];
        for (code, cut_points) in cases {
            for &code_len in cut_points.iter().chain(std::iter::once(&code.len())) {
                let bytes = bytecode_fixture(52, 0, Some(&code[..code_len]), &[], false, false);
                let mut budget = Budget::new(limits(u64::MAX));
                match inspect_method_bytecode(&bytes, selector(), &mut budget) {
                    Ok(report) => {
                        assert_bytecode_report_invariants(
                            &report,
                            &bytes,
                            &code[..code_len],
                            &budget,
                        );
                        if code_len == code.len() {
                            assert!(matches!(report.execution, ExecutionReport::Partial { .. }));
                        }
                    }
                    Err(Error::InvalidInput { .. } | Error::Unsupported { .. }) => {}
                    Err(other) => panic!("unexpected unlimited-budget error: {other}"),
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn property_attribute_lengths_and_tail_truncation_are_bounded(
            attribute_index in 0usize..5,
            declared_length in any::<u32>(),
            trim_from_tail in 0usize..=64,
        ) {
            let fixture = fixture();
            let mut bytes = fixture.bytes;
            let length_offset = fixture.attribute_length_offsets[attribute_index];
            bytes[length_offset..length_offset + 4].copy_from_slice(&declared_length.to_be_bytes());
            let retained = bytes.len().saturating_sub(trim_from_tail);
            bytes.truncate(retained);

            let header_result = inspect_header(
                &bytes,
                &mut Budget::new(limits(u64::MAX)),
                InspectionMode::Forensic,
            );
            if attribute_index == 4 && trim_from_tail == 0 {
                assert!(header_result.is_ok(), "Header must lazily ignore nested Code content");
            }
            assert_structured_header_result(header_result, &bytes);

            let mut bytecode_budget = Budget::new(limits(u64::MAX));
            match inspect_method_bytecode(&bytes, selector(), &mut bytecode_budget) {
                Ok(report) => assert_bytecode_report_invariants(
                    &report,
                    &bytes,
                    &[0xb1],
                    &bytecode_budget,
                ),
                Err(Error::InvalidInput { .. }) => {}
                Err(other) => panic!("unexpected unlimited bytecode result: {other}"),
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn property_mutf8_utf16_round_trip_and_escaping_are_stable(
            units in prop::collection::vec(any::<u16>(), 0..=16),
        ) {
            let raw = mutf8(&units);
            let mstr = noak::MStr::from_mutf8(&raw).unwrap();
            let value = jvm_string_mstr(mstr).unwrap();
            prop_assert_eq!(value.raw().0.as_slice(), raw.as_slice());
            prop_assert_eq!(value.utf16(), units.as_slice());
            let repeated = jvm_string_mstr(noak::MStr::from_mutf8(value.raw().0.as_slice()).unwrap()).unwrap();
            prop_assert_eq!(value.escaped(), repeated.escaped());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 512,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        #[test]
        fn property_instruction_byte_vectors_preserve_reliable_prefix(
            code in prop::collection::vec(any::<u8>(), 0..=64),
        ) {
            let bytes = bytecode_fixture(52, 0, Some(&code), &[], false, false);
            let mut budget = Budget::new(limits(u64::MAX));
            let report = inspect_method_bytecode(&bytes, selector(), &mut budget)
                .expect("any instruction byte vector must produce a method-local report");
            assert_bytecode_report_invariants(&report, &bytes, &code, &budget);
        }
    }

    #[test]
    fn invalid_tableswitch_ranges_are_method_local_partial_reports() {
        let cases = [
            (
                vec![0xaa, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0],
                0,
                0,
            ),
            (
                vec![0x00, 0xaa, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0],
                1,
                1,
            ),
        ];
        for (code, expected_bci, expected_facts) in cases {
            let bytes = bytecode_fixture(52, 0, Some(&code), &[], false, false);
            let mut budget = Budget::new(limits(u64::MAX));
            let report = inspect_method_bytecode(&bytes, selector(), &mut budget).unwrap();
            assert_eq!(report.instructions.len(), expected_facts);
            assert_bytecode_report_invariants(&report, &bytes, &code, &budget);
            assert_eq!(
                instruction_stop(report.stopped_at.as_ref().unwrap()),
                (
                    expected_bci,
                    report.code_span.start + u64::from(expected_bci),
                    "classfile_instruction_invalid_range"
                )
            );
        }

        let valid = vec![
            0xaa, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0,
        ];
        let bytes = bytecode_fixture(52, 0, Some(&valid), &[], false, false);
        let report =
            inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX)))
                .unwrap();
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert_eq!(report.instructions[0].width, 20);
    }

    #[test]
    fn malformed_wide_forms_stop_exactly_at_the_current_bci() {
        let cases: &[&[u8]] = &[
            &[0xc4],
            &[0xc4, 0x00, 0x00, 0x00],
            &[0xc4, 0x15, 0x00],
            &[0xc4, 0x84, 0x00, 0x01, 0x00],
        ];
        for &code in cases {
            let bytes = bytecode_fixture(52, 0, Some(code), &[], false, false);
            let mut budget = Budget::new(limits(u64::MAX));
            let report = inspect_method_bytecode(&bytes, selector(), &mut budget).unwrap();
            assert_bytecode_report_invariants(&report, &bytes, code, &budget);
            assert_eq!(report.instructions.len(), 0);
            assert_eq!(instruction_stop(report.stopped_at.as_ref().unwrap()).0, 0);
        }
    }

    #[test]
    fn synthetic_historical_jsr_jsr_w_and_ret_boundaries_are_preserved() {
        // Synthetic boundary coverage only; this is not a historical compiler corpus.
        let code = [
            0xa8, 0x00, 0x00, // jsr
            0xc9, 0x00, 0x00, 0x00, 0x00, // jsr_w
            0xa9, 0x01, // ret
        ];
        for major in [49, 52] {
            let bytes = bytecode_fixture(major, 0, Some(&code), &[], false, false);
            let report =
                inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX)))
                    .unwrap();
            assert_eq!(
                report
                    .instructions
                    .iter()
                    .map(|fact| (fact.bci, fact.opcode, fact.width, fact.constant_pool_index))
                    .collect::<Vec<_>>(),
                vec![(0, 0xa8, 3, None), (3, 0xc9, 5, None), (8, 0xa9, 2, None)]
            );
            for fact in &report.instructions {
                assert_eq!(
                    slice(&bytes, &fact.span),
                    &code[fact.bci as usize..(fact.bci + fact.width) as usize]
                );
            }
        }
    }

    #[test]
    fn descriptor_mutf8_boundaries_survive_the_real_header_path() {
        let units = [
            b'(' as u16,
            0,
            0xd83d,
            0xde00,
            0xd800,
            b')' as u16,
            b'V' as u16,
        ];
        let raw = mutf8(&units);
        let bytes = descriptor_header_fixture(&raw);
        let header = inspect_header(
            &bytes,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap()
        .header;
        let descriptor = &header.methods[0].descriptor;
        assert_eq!(descriptor.raw().0, raw);
        assert_eq!(descriptor.utf16(), units);
        assert_eq!(descriptor.escaped(), "(\\u0000\\uD83D\\uDE00\\uD800)V");
    }

    #[test]
    fn inspects_fixed_wide_cp_and_exception_facts() {
        let code = [
            0x12, 0x01, // ldc
            0xb2, 0x00, 0x01, // getstatic
            0xc4, 0x15, 0x00, 0x02, // wide iload
            0xc4, 0x84, 0x00, 0x02, 0x00, 0x01, // wide iinc
            0xb1, // return
        ];
        let bytes = bytecode_fixture(
            52,
            0,
            Some(&code),
            &[(0, 5, 15, 0), (2, 9, 15, 2)],
            false,
            false,
        );
        let mut budget = Budget::new(limits(u64::MAX));
        let report = inspect_method_bytecode(&bytes, selector(), &mut budget).unwrap();
        assert_eq!((report.max_stack, report.max_locals), (4, 3));
        assert_eq!(
            report
                .instructions
                .iter()
                .map(|fact| (fact.bci, fact.opcode, fact.width, fact.constant_pool_index))
                .collect::<Vec<_>>(),
            vec![
                (0, 0x12, 2, Some(1)),
                (2, 0xb2, 3, Some(1)),
                (5, 0xc4, 4, None),
                (9, 0xc4, 6, None),
                (15, 0xb1, 1, None),
            ]
        );
        for fact in &report.instructions {
            assert_eq!(
                slice(&bytes, &fact.span),
                &code[fact.bci as usize..(fact.bci + fact.width) as usize]
            );
            assert_eq!(fact.operands_span.start, fact.span.start + 1);
            assert_eq!(fact.operands_span.length, u64::from(fact.width - 1));
        }
        assert_eq!(
            report.exception_handlers,
            vec![
                ExceptionHandlerFact {
                    ordinal: 0,
                    start_bci: 0,
                    end_bci: 5,
                    handler_bci: 15,
                    catch_type_index: None
                },
                ExceptionHandlerFact {
                    ordinal: 1,
                    start_bci: 2,
                    end_bci: 9,
                    handler_bci: 15,
                    catch_type_index: Some(2)
                },
            ]
        );
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert_eq!(report.verification, VerificationStatus::NotPerformed);
        assert_eq!(
            serde_json::from_str::<BytecodeInspection>(&serde_json::to_string(&report).unwrap())
                .unwrap(),
            report
        );
    }

    #[test]
    fn switch_widths_use_code_start_alignment_and_do_not_iterate_table_pairs() {
        let mut table = vec![0x00, 0xaa, 0, 0];
        table.extend_from_slice(&0i32.to_be_bytes());
        table.extend_from_slice(&i32::MAX.to_be_bytes());
        table.extend_from_slice(&i32::MAX.to_be_bytes());
        table.extend_from_slice(&0i32.to_be_bytes());
        table.push(0xb1);
        let bytes = bytecode_fixture(52, 0, Some(&table), &[], false, false);
        let report =
            inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX)))
                .unwrap();
        assert_eq!(
            report
                .instructions
                .iter()
                .map(|fact| (fact.bci, fact.width))
                .collect::<Vec<_>>(),
            vec![(0, 1), (1, 19), (20, 1)]
        );

        let mut lookup = vec![0x00, 0xab, 0, 0];
        lookup.extend_from_slice(&0i32.to_be_bytes());
        lookup.extend_from_slice(&1i32.to_be_bytes());
        lookup.extend_from_slice(&7i32.to_be_bytes());
        lookup.extend_from_slice(&0i32.to_be_bytes());
        lookup.push(0xb1);
        let bytes = bytecode_fixture(52, 0, Some(&lookup), &[], false, false);
        let report =
            inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX)))
                .unwrap();
        assert_eq!(
            report
                .instructions
                .iter()
                .map(|fact| (fact.bci, fact.width))
                .collect::<Vec<_>>(),
            vec![(0, 1), (1, 19), (20, 1)]
        );
    }

    #[test]
    fn instruction_failures_return_reliable_prefix_and_stop_location() {
        for code in [&[0x00, 0xcb, 0xb1][..], &[0x00, 0xb2, 0x00][..]] {
            let bytes = bytecode_fixture(52, 0, Some(code), &[], false, false);
            let report =
                inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX)))
                    .unwrap();
            assert_eq!(report.instructions.len(), 1);
            assert_eq!(report.instructions[0].bci, 0);
            let stop = report.stopped_at.as_ref().unwrap();
            assert_eq!(
                instruction_stop(stop),
                (
                    1,
                    report.code_span.start.checked_add(1).unwrap(),
                    "classfile_instruction_decode"
                )
            );
            let json = serde_json::to_value(stop).unwrap();
            assert_eq!(json["phase"], "instructions");
            assert_eq!(json["bci"], 1);
            assert!(json.get("ordinal").is_none());
            assert!(matches!(report.execution, ExecutionReport::Partial { .. }));
        }
    }

    #[test]
    fn method_lookup_code_cardinality_and_version_gate_are_stable() {
        let bytes = bytecode_fixture(52, 0, Some(&[0xb1]), &[], false, false);
        let missing = MethodSelector {
            name: JvmBytes(b"missing".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        };
        assert!(
            matches!(inspect_method_bytecode(&bytes, missing, &mut Budget::new(limits(u64::MAX))), Err(Error::InvalidInput { ref code, .. }) if code == "classfile_method_not_found")
        );
        let duplicate = bytecode_fixture(52, 0, Some(&[0xb1]), &[], true, false);
        assert!(
            matches!(inspect_method_bytecode(&duplicate, selector(), &mut Budget::new(limits(u64::MAX))), Err(Error::InvalidInput { ref code, .. }) if code == "classfile_method_ambiguous")
        );
        let no_code = bytecode_fixture(52, 0, None, &[], false, false);
        assert!(
            matches!(inspect_method_bytecode(&no_code, selector(), &mut Budget::new(limits(u64::MAX))), Err(Error::Unsupported { ref code, .. }) if code == "classfile_method_has_no_code")
        );
        let duplicate_code = bytecode_fixture(52, 0, Some(&[0xb1]), &[], false, true);
        assert!(
            matches!(inspect_method_bytecode(&duplicate_code, selector(), &mut Budget::new(limits(u64::MAX))), Err(Error::InvalidInput { ref code, .. }) if code == "classfile_duplicate_code_attribute")
        );

        for (major, minor, expected) in [
            (52, 1, "classfile_java8_runtime_rejected"),
            (53, 0, "classfile_version_structural_probe_only"),
            (56, u16::MAX, "classfile_preview_unsupported"),
            (72, 0, "classfile_future_release"),
        ] {
            let bytes = bytecode_fixture(major, minor, Some(&[0xb1]), &[], false, false);
            assert!(
                matches!(inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX))), Err(Error::Unsupported { code, .. }) if code == expected)
            );
        }
    }

    #[test]
    fn bytecode_request_charges_only_local_code_shell_and_returned_facts() {
        let bytes = bytecode_fixture(52, 0, Some(&[0xb1]), &[], false, false);
        let mut configured = limits(u64::MAX);
        configured.attribute_bytes = 19;
        configured.result_items = 2;
        let mut budget = Budget::new(configured);
        let report = inspect_method_bytecode(&bytes, selector(), &mut budget).unwrap();
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert_eq!(budget.usage().class_bytes, bytes.len() as u64);
        assert_eq!(budget.usage().attribute_bytes, 19);
        assert_eq!(budget.usage().result_items, 2); // report root + one instruction
        assert_eq!(budget.usage().code_bytes, 1);
    }

    #[test]
    fn instruction_failure_preserves_precollected_handlers() {
        let bytes = bytecode_fixture(
            52,
            0,
            Some(&[0x00, 0xcb]),
            &[(0, 1, 1, 0), (0, 2, 1, 2)],
            false,
            false,
        );
        let report =
            inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX)))
                .unwrap();
        assert_eq!(report.exception_handlers.len(), 2);
        assert_eq!(report.exception_handlers[0].ordinal, 0);
        assert_eq!(report.exception_handlers[1].ordinal, 1);
        assert_eq!(report.instructions.len(), 1);
        assert_eq!(instruction_stop(report.stopped_at.as_ref().unwrap()).0, 1);
    }

    #[test]
    fn handler_stage_budget_and_cancellation_preserve_handler_prefix() {
        let bytes = bytecode_fixture(
            52,
            0,
            Some(&[0xb1]),
            &[(0, 1, 0, 0), (0, 1, 0, 2)],
            false,
            false,
        );
        let mut configured = limits(u64::MAX);
        configured.result_items = 2; // root + first handler
        let report =
            inspect_method_bytecode(&bytes, selector(), &mut Budget::new(configured)).unwrap();
        assert_eq!(report.exception_handlers.len(), 1);
        assert!(report.instructions.is_empty());
        let expected_offset = report
            .code_span
            .start
            .checked_add(report.code_span.length)
            .and_then(|offset| offset.checked_add(2))
            .and_then(|offset| offset.checked_add(8))
            .unwrap();
        assert_eq!(
            handler_stop(report.stopped_at.as_ref().unwrap()),
            (1, expected_offset, "classfile_bytecode_budget_exceeded")
        );
        let json = serde_json::to_value(report.stopped_at.as_ref().unwrap()).unwrap();
        assert_eq!(json["phase"], "exception_handlers");
        assert_eq!(json["ordinal"], 1);
        assert!(json.get("bci").is_none());
        assert!(matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ResultItems
                },
                ..
            }
        ));
        assert_eq!(report.diagnostics.len(), 1); // termination metadata is deliberately uncharged

        let token = CancellationToken::new();
        let mut budget = Budget::with_cancellation_token(limits(u64::MAX), token.clone());
        let report = inspect_method_bytecode_with(
            &bytes,
            selector(),
            &mut budget,
            |ordinal| {
                if ordinal == 1 {
                    token.cancel();
                }
            },
            |_| {},
        )
        .unwrap();
        assert_eq!(report.exception_handlers.len(), 1);
        assert!(report.instructions.is_empty());
        assert_eq!(
            handler_stop(report.stopped_at.as_ref().unwrap()),
            (1, expected_offset, "classfile_bytecode_cancelled")
        );
        assert!(matches!(
            report.execution,
            ExecutionReport::Cancelled { .. }
        ));
    }

    #[test]
    fn bytecode_cancellation_returns_deterministic_prefix() {
        let bytes = bytecode_fixture(52, 0, Some(&[0x00, 0x00, 0xb1]), &[], false, false);
        let token = CancellationToken::new();
        let mut budget = Budget::with_cancellation_token(limits(u64::MAX), token.clone());
        let report = inspect_method_bytecode_with(
            &bytes,
            selector(),
            &mut budget,
            |_| {},
            |bci| {
                if bci == 1 {
                    token.cancel();
                }
            },
        )
        .unwrap();
        assert_eq!(report.instructions.len(), 1);
        assert_eq!(instruction_stop(report.stopped_at.as_ref().unwrap()).0, 1);
        assert!(matches!(
            report.execution,
            ExecutionReport::Cancelled { .. }
        ));
    }

    #[test]
    fn bytecode_budgets_return_prefix_reports() {
        let bytes = bytecode_fixture(52, 0, Some(&[0x00, 0x00, 0xb1]), &[], false, false);
        let mut configured = limits(u64::MAX);
        configured.code_bytes = 1;
        let report =
            inspect_method_bytecode(&bytes, selector(), &mut Budget::new(configured)).unwrap();
        assert_eq!(report.instructions.len(), 1);
        assert_eq!(instruction_stop(report.stopped_at.as_ref().unwrap()).0, 1);
        assert!(matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::CodeBytes
                },
                ..
            }
        ));
    }

    #[test]
    fn width_adapter_covers_operand_bearing_opcode_categories() {
        let cases: &[(&[u8], u32)] = &[
            (&[0x10, 0], 2),
            (&[0x11, 0, 0], 3),
            (&[0x12, 1], 2),
            (&[0x13, 0, 1], 3),
            (&[0x84, 0, 0], 3),
            (&[0xa7, 0, 0], 3),
            (&[0xb9, 0, 1, 1, 0], 5),
            (&[0xba, 0, 1, 0, 0], 5),
            (&[0xc5, 0, 1, 1], 4),
            (&[0xc8, 0, 0, 0, 0], 5),
            (&[0xc4, 0x15, 0, 1], 4),
            (&[0xc4, 0x84, 0, 1, 0, 1], 6),
        ];
        for &(code, expected) in cases {
            let bytes = bytecode_fixture(52, 0, Some(code), &[], false, false);
            let report =
                inspect_method_bytecode(&bytes, selector(), &mut Budget::new(limits(u64::MAX)))
                    .unwrap();
            assert_eq!(
                report.instructions[0].width, expected,
                "opcode {:02x}",
                code[0]
            );
        }
    }

    #[test]
    fn version_matrix_keeps_rules_dialect_and_java8_profile_orthogonal() {
        let cases = [
            (
                45,
                3,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                45,
                u16::MAX,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                46,
                0,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                47,
                0,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                48,
                0,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                49,
                0,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                50,
                0,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                51,
                1,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                52,
                0,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Accepted,
                true,
            ),
            (
                52,
                1,
                VersionRuleStatus::Valid,
                VersionDialectSupport::Supported,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
            (
                53,
                7,
                VersionRuleStatus::Valid,
                VersionDialectSupport::StructuralProbeOnly,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
            (
                55,
                1,
                VersionRuleStatus::Valid,
                VersionDialectSupport::StructuralProbeOnly,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
            (
                56,
                1,
                VersionRuleStatus::InvalidModernMinor,
                VersionDialectSupport::StructuralProbeOnly,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
            (
                56,
                u16::MAX,
                VersionRuleStatus::Valid,
                VersionDialectSupport::UnsupportedPreview,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
            (
                71,
                0,
                VersionRuleStatus::Valid,
                VersionDialectSupport::StructuralProbeOnly,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
            (
                72,
                0,
                VersionRuleStatus::Valid,
                VersionDialectSupport::FutureRelease,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
            (
                72,
                u16::MAX,
                VersionRuleStatus::Valid,
                VersionDialectSupport::FutureRelease,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
            (
                44,
                0,
                VersionRuleStatus::InvalidMajor,
                VersionDialectSupport::StructuralProbeOnly,
                Java8RuntimeCompatibility::Rejected,
                false,
            ),
        ];

        for (major, minor, rule, dialect, java8, strict_ok) in cases {
            let fixture = fixture_version(major, minor);
            let forensic = inspect_header(
                &fixture.bytes,
                &mut Budget::new(limits(u64::MAX)),
                InspectionMode::Forensic,
            )
            .unwrap();
            assert_eq!(forensic.structural_read, HeaderStructuralRead::Complete);
            assert_eq!(forensic.verification, VerificationStatus::NotPerformed);
            assert_eq!(
                forensic.version_capability.version,
                ClassfileVersion { major, minor }
            );
            assert_eq!(forensic.version_capability.version_rule, rule);
            assert_eq!(forensic.version_capability.version_dialect_support, dialect);
            assert_eq!(
                forensic.version_capability.dialect_validation_scope,
                DialectValidationScope::VersionOnly
            );
            assert_eq!(forensic.version_capability.java8_runtime, java8);
            assert_eq!(
                inspect_header(
                    &fixture.bytes,
                    &mut Budget::new(limits(u64::MAX)),
                    InspectionMode::Strict,
                )
                .is_ok(),
                strict_ok,
                "strict result for {major}.{minor}"
            );
        }
    }

    /// The preview and unregistered-release scenario, on the report the reader publishes.
    ///
    /// The three planes must stay separate in the answer: the structure was read, no verification
    /// ran, no output level was evaluated, and the dialect plane states non-support. A readable
    /// structure is never evidence of dialect or verifier success, and an unregistered release
    /// carries no registered capability with it.
    #[test]
    fn preview_and_unregistered_releases_stay_readable_and_are_never_reported_as_supported() {
        let registry = feature_registry();
        let cases = [
            (
                56,
                u16::MAX,
                VersionDialectSupport::UnsupportedPreview,
                ReleaseRegistration::Registered,
                PreviewMarker::Present,
                "classfile_preview_unsupported",
            ),
            (
                71,
                u16::MAX,
                VersionDialectSupport::UnsupportedPreview,
                ReleaseRegistration::Registered,
                PreviewMarker::Present,
                "classfile_preview_unsupported",
            ),
            (
                62,
                0,
                VersionDialectSupport::StructuralProbeOnly,
                ReleaseRegistration::Registered,
                PreviewMarker::Absent,
                "classfile_version_structural_probe_only",
            ),
            (
                72,
                0,
                VersionDialectSupport::FutureRelease,
                ReleaseRegistration::UnregisteredFutureRelease,
                PreviewMarker::Absent,
                "classfile_future_release",
            ),
            (
                72,
                u16::MAX,
                VersionDialectSupport::FutureRelease,
                ReleaseRegistration::UnregisteredFutureRelease,
                PreviewMarker::Present,
                "classfile_future_release",
            ),
            (
                99,
                3,
                VersionDialectSupport::FutureRelease,
                ReleaseRegistration::UnregisteredFutureRelease,
                PreviewMarker::Absent,
                "classfile_future_release",
            ),
        ];
        for (major, minor, dialect, registration, preview_marker, code) in cases {
            let fixture = fixture_version(major, minor);
            let forensic = inspect_header(
                &fixture.bytes,
                &mut Budget::new(limits(u64::MAX)),
                InspectionMode::Forensic,
            )
            .unwrap();
            // Parse: the structure was read, and this success belongs to that plane alone.
            assert_eq!(forensic.structural_read, HeaderStructuralRead::Complete);
            // Verification: this layer ran none, whatever the format claims.
            assert_eq!(forensic.verification, VerificationStatus::NotPerformed);
            // Output level: this layer evaluated none either.
            assert_eq!(forensic.output_level, OutputLevelStatus::NotEvaluated);
            // Dialect: explicit non-support, never a supported band.
            assert_eq!(forensic.version_capability.version_dialect_support, dialect);
            assert_eq!(forensic.version_capability.preview_marker, preview_marker);
            assert_eq!(
                forensic.version_capability.release_registration, registration,
                "{major}.{minor}"
            );
            assert!(
                forensic
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == code),
                "diagnostic {code} for {major}.{minor}"
            );
            if registration == ReleaseRegistration::UnregisteredFutureRelease {
                // The conservative path: no record, so no constraint of a registered release.
                assert!(registry.release_record(major).is_none());
                assert_eq!(registry.attributes(major).count(), 0);
                assert_eq!(registry.flags(major).count(), 0);
            }
            assert!(
                inspect_header(
                    &fixture.bytes,
                    &mut Budget::new(limits(u64::MAX)),
                    InspectionMode::Strict,
                )
                .is_err(),
                "strict refusal for {major}.{minor}"
            );
        }
    }

    /// The report's version facts are the registry record's, so the two cannot drift apart.
    #[test]
    fn header_capability_follows_the_release_registry() {
        let registry = feature_registry();
        for major in [45, 50, 52, 53, 55, 56, 60, 61, 71] {
            let record = registry.release_record(major).expect("registered release");
            let capability = version_capability(major, 0);
            assert_eq!(
                capability.release_registration,
                ReleaseRegistration::Registered,
                "{major}"
            );
            assert_eq!(capability.preview_marker, PreviewMarker::Absent, "{major}");
            assert_eq!(
                capability.version_dialect_support,
                record.dialect_support(),
                "{major}"
            );
            assert_eq!(capability.java8_runtime, record.java8_runtime(0), "{major}");
            let fixture = fixture_version(major, 0);
            let inspection = inspect_header(
                &fixture.bytes,
                &mut Budget::new(limits(u64::MAX)),
                InspectionMode::Forensic,
            )
            .unwrap();
            assert_eq!(inspection.version_capability, capability, "{major}");
        }

        assert_eq!(registry.dialect_validated_ceiling(), 52);
        // The minor rule of the format reaches above the ceiling, and below the format's minimum the
        // band stays the conservative structural probe it was.
        let unregistered_future = version_capability(72, 1);
        assert_eq!(
            unregistered_future.version_rule,
            VersionRuleStatus::InvalidModernMinor
        );
        assert_eq!(
            unregistered_future.release_registration,
            ReleaseRegistration::UnregisteredFutureRelease
        );
        let below_minimum = version_capability(44, 0);
        assert_eq!(below_minimum.version_rule, VersionRuleStatus::InvalidMajor);
        assert_eq!(
            below_minimum.release_registration,
            ReleaseRegistration::UnregisteredBelowMinimum
        );
        assert_eq!(
            below_minimum.version_dialect_support,
            VersionDialectSupport::StructuralProbeOnly
        );
        assert_eq!(below_minimum.preview_marker, PreviewMarker::Absent);
    }

    /// The boundary this slice draws: 1.1 states the release rules and answers them on demand; the
    /// passes that read an artifact's attributes and flags report them (1.2/1.3).
    #[test]
    fn header_inspection_leaves_attribute_and_flag_legality_to_the_fact_passes() {
        let fixture = fixture_version(55, 0);
        let inspection = inspect_header(
            &fixture.bytes,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Forensic,
        )
        .unwrap();
        for diagnostic in &inspection.diagnostics {
            assert!(
                matches!(
                    diagnostic.code.as_str(),
                    "classfile_version_structural_probe_only"
                        | "classfile_preview_unsupported"
                        | "classfile_future_release"
                        | "classfile_invalid_major_version"
                        | "classfile_invalid_modern_minor_version"
                        | "classfile_java8_runtime_rejected"
                ),
                "the header plan states version facts only, not {}",
                diagnostic.code
            );
        }

        let registry = feature_registry();
        let version = registry
            .attribute_diagnostic("Record", ClassfileLocation::ClassFile, 55)
            .expect("the registry states the rule");
        assert_eq!(version.code, "classfile_attribute_version_not_applicable");
        assert_eq!(
            version.message,
            "attribute \"Record\" is registered from major 60 (JVMS 4.7.30); it is not valid at major 55"
        );
        assert!(version.provenance.is_none());
        assert_eq!(
            registry.attribute_diagnostic("Record", ClassfileLocation::ClassFile, 60),
            None
        );
        let location = registry
            .attribute_diagnostic("NestMembers", ClassfileLocation::MethodInfo, 55)
            .expect("the registry states the rule");
        assert_eq!(location.code, "classfile_attribute_location_not_applicable");
        assert_eq!(
            location.message,
            "attribute \"NestMembers\" is registered only for ClassFile (JVMS 4.7.29); it is not valid in a method_info structure"
        );
    }

    #[test]
    fn strict_errors_and_forensic_diagnostics_have_stable_version_codes() {
        let cases = [
            (44, 0, "classfile_invalid_major_version", true),
            (56, 1, "classfile_invalid_modern_minor_version", true),
            (52, 1, "classfile_java8_runtime_rejected", false),
            (56, u16::MAX, "classfile_preview_unsupported", false),
            (71, 0, "classfile_version_structural_probe_only", false),
            (72, 0, "classfile_future_release", false),
            (72, u16::MAX, "classfile_future_release", false),
        ];
        for (major, minor, code, invalid_input) in cases {
            let fixture = fixture_version(major, minor);
            let forensic = inspect_header(
                &fixture.bytes,
                &mut Budget::new(limits(u64::MAX)),
                InspectionMode::Forensic,
            )
            .unwrap();
            assert!(
                forensic.diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == code && diagnostic.provenance.is_none()
                })
            );
            let error = inspect_header(
                &fixture.bytes,
                &mut Budget::new(limits(u64::MAX)),
                InspectionMode::Strict,
            )
            .unwrap_err();
            assert!(match error {
                Error::InvalidInput { code: actual, .. } if invalid_input => actual == code,
                Error::Unsupported { code: actual, .. } if !invalid_input => actual == code,
                _ => false,
            });
        }
    }

    #[test]
    fn future_major_takes_priority_over_preview_marker() {
        let fixture = fixture_version(72, u16::MAX);
        let forensic = inspect_header(
            &fixture.bytes,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Forensic,
        )
        .unwrap();
        assert_eq!(
            forensic.version_capability.version_dialect_support,
            VersionDialectSupport::FutureRelease
        );
        assert!(
            forensic
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "classfile_future_release")
        );
        assert!(
            !forensic
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "classfile_preview_unsupported")
        );

        let error = inspect_header(
            &fixture.bytes,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            Error::Unsupported { ref code, .. } if code == "classfile_future_release"
        ));
    }

    #[test]
    fn forensic_diagnostics_are_atomically_charged_as_result_items() {
        let fixture = fixture_version(55, 1);

        let mut exact_structure = limits(u64::MAX);
        exact_structure.result_items = 8;
        let mut budget = Budget::new(exact_structure);
        let error =
            inspect_header(&fixture.bytes, &mut budget, InspectionMode::Forensic).unwrap_err();
        assert_eq!(budget.usage().result_items, 8);
        assert!(matches!(
            error,
            Error::BudgetExceeded {
                dimension: BudgetDimension::ResultItems,
                consumed: 8,
                requested: 2,
                ..
            }
        ));

        let mut with_diagnostics = limits(u64::MAX);
        with_diagnostics.result_items = 10;
        let mut budget = Budget::new(with_diagnostics);
        let inspection =
            inspect_header(&fixture.bytes, &mut budget, InspectionMode::Forensic).unwrap();
        assert_eq!(inspection.diagnostics.len(), 2);
        assert_eq!(budget.usage().result_items, 10);

        let mut strict_limit = limits(u64::MAX);
        strict_limit.result_items = 8;
        let mut budget = Budget::new(strict_limit);
        let error =
            inspect_header(&fixture.bytes, &mut budget, InspectionMode::Strict).unwrap_err();
        assert!(matches!(
            error,
            Error::Unsupported { ref code, .. }
                if code == "classfile_version_structural_probe_only"
        ));
        assert_eq!(budget.usage().result_items, 8);
    }

    #[test]
    fn version_capability_and_header_inspection_round_trip_as_json() {
        let fixture = fixture_version(55, 1);
        let inspection = inspect_header(
            &fixture.bytes,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Forensic,
        )
        .unwrap();
        let capability_json = serde_json::to_string(&inspection.version_capability).unwrap();
        assert!(capability_json.contains("\"version_dialect_support\":\"structural_probe_only\""));
        assert_eq!(
            serde_json::from_str::<VersionCapability>(&capability_json).unwrap(),
            inspection.version_capability
        );
        let json = serde_json::to_string(&inspection).unwrap();
        assert!(json.contains("\"verification\":\"not_performed\""));
        assert_eq!(
            serde_json::from_str::<HeaderInspection>(&json).unwrap(),
            inspection
        );
    }

    #[test]
    fn inspects_owned_lossless_header_and_exact_attribute_shells() {
        let fixture = fixture();
        let mut budget = Budget::new(limits(u64::MAX));
        let inspection =
            inspect_header(&fixture.bytes, &mut budget, InspectionMode::Strict).unwrap();
        assert_eq!(inspection.structural_read, HeaderStructuralRead::Complete);
        assert_eq!(inspection.verification, VerificationStatus::NotPerformed);
        let header = inspection.header;
        assert_eq!((header.major_version, header.minor_version), (52, 0));
        assert_eq!(header.access_flags, 0x0021);
        assert_eq!(
            header.this_class.raw().0,
            b"A\xc0\x80\xed\xa0\xbd\xed\xb8\x80\xed\xa0\x80\\\"\xe2\x80\xae"
        );
        assert_eq!(
            header.this_class.utf16(),
            &[0x41, 0, 0xd83d, 0xde00, 0xd800, 0x5c, 0x22, 0x202e]
        );
        assert_eq!(
            header.this_class.escaped(),
            "A\\u0000\\uD83D\\uDE00\\uD800\\\\\\\"\\u202E"
        );
        assert_eq!(
            header.super_class.as_ref().unwrap().escaped(),
            "java/lang/Object"
        );
        assert_eq!(header.interfaces[0].escaped(), "pkg/Interface");
        assert_eq!(header.fields[0].name.escaped(), "field");
        assert_eq!(header.fields[0].descriptor.escaped(), "I");
        assert_eq!(header.methods[0].name.escaped(), "method");
        assert_eq!(header.methods[0].descriptor.escaped(), "()V");

        assert_eq!(header_attribute_shells(&header).count(), 4);
        let nested_shell_start = fixture.attribute_length_offsets[4] - 2;
        assert!(
            header_attribute_shells(&header)
                .all(|shell| shell.span.start != nested_shell_start as u64)
        );
        let shells = [
            (&header.fields[0].attributes[0], &fixture.field_attribute),
            (&header.methods[0].attributes[0], &fixture.method_attribute),
            (&header.methods[0].attributes[1], &fixture.code_attribute),
            (&header.attributes[0], &fixture.class_attribute),
        ];
        for (actual, expected) in shells {
            assert_eq!(
                (&actual.span, &actual.content_span),
                (&expected.0, &expected.1)
            );
            assert_eq!(
                slice(&fixture.bytes, &actual.span),
                slice(&fixture.bytes, &expected.0)
            );
            assert_eq!(
                slice(&fixture.bytes, &actual.content_span),
                slice(&fixture.bytes, &expected.1)
            );
        }
        assert_eq!(
            slice(
                &fixture.bytes,
                &header.methods[0].attributes[0].content_span
            ),
            &[0xff]
        );
        assert_eq!(
            slice(
                &fixture.bytes,
                &header.methods[0].attributes[1].content_span
            ),
            &[
                0, 1, 0, 1, 0, 0, 0, 1, 0xb1, 0, 0, 0, 1, 0, 14, 0, 0, 0, 2, 0xca, 0xfe,
            ]
        );
        assert_eq!(budget.usage().class_bytes, fixture.bytes.len() as u64);
        assert_eq!(budget.usage().attribute_bytes, 51);
        assert_eq!(budget.usage().result_items, 8);
    }

    #[test]
    fn jvm_string_json_rejects_inconsistent_derived_state() {
        let fixture = fixture();
        let header = inspect_header(
            &fixture.bytes,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap()
        .header;
        let mut value = serde_json::to_value(&header.this_class).unwrap();
        assert_eq!(value["escaped"], header.this_class.escaped());
        value["utf16"] = serde_json::json!([65]);
        assert!(serde_json::from_value::<JvmString>(value).is_err());
    }

    #[test]
    fn rejects_trailing_bytes_and_decode_failures_with_stable_codes() {
        let mut trailing = fixture().bytes;
        trailing.push(0);
        let error = inspect_header(
            &trailing,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_trailing_bytes")
        );

        let mut invalid_mutf8 = fixture().bytes;
        let nul = invalid_mutf8.iter().position(|byte| *byte == 0xc0).unwrap();
        invalid_mutf8[nul + 1] = 0x41;
        let error = inspect_header(
            &invalid_mutf8,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, ref message } if code == "classfile_decode" && message.contains("kind=invalid modified utf8") && message.contains("context=") && message.contains("position="))
        );

        let mut bad_index = fixture().bytes;
        let class_info = bad_index
            .windows(10)
            .position(|window| window == [0, 0x21, 0, 2, 0, 4, 0, 1, 0, 13])
            .unwrap();
        bad_index[class_info + 2..class_info + 4].copy_from_slice(&1u16.to_be_bytes());
        let error = inspect_header(
            &bad_index,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_decode")
        );

        let mut bad_tag = fixture().bytes;
        bad_tag[10] = 2;
        let error = inspect_header(
            &bad_tag,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_decode")
        );

        let truncated = &fixture().bytes[..12];
        let error = inspect_header(
            truncated,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_decode")
        );
    }

    #[test]
    fn enforces_class_attribute_result_and_cancellation_budgets() {
        let fixture = fixture();
        let mut class_limits = limits(u64::MAX);
        class_limits.class_bytes = fixture.bytes.len() as u64 - 1;
        assert!(matches!(
            inspect_header(
                &fixture.bytes,
                &mut Budget::new(class_limits),
                InspectionMode::Strict
            ),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ClassBytes,
                ..
            })
        ));

        let mut attribute_limits = limits(u64::MAX);
        attribute_limits.attribute_bytes = 7;
        assert!(matches!(
            inspect_header(
                &fixture.bytes,
                &mut Budget::new(attribute_limits),
                InspectionMode::Strict
            ),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::AttributeBytes,
                ..
            })
        ));

        let mut result_limits = limits(u64::MAX);
        result_limits.result_items = 1;
        assert!(matches!(
            inspect_header(
                &fixture.bytes,
                &mut Budget::new(result_limits),
                InspectionMode::Strict
            ),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ResultItems,
                ..
            })
        ));

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(limits(u64::MAX), token);
        assert!(matches!(
            inspect_header(&fixture.bytes, &mut cancelled, InspectionMode::Strict),
            Err(Error::Cancelled { .. })
        ));
        assert_eq!(cancelled.usage().class_bytes, 0);
    }

    #[test]
    fn rejects_long_in_last_constant_pool_slot_that_noak_accepts() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 52);
        u16_be(&mut bytes, 4);
        utf8(&mut bytes, b"A");
        class_entry(&mut bytes, 1);
        bytes.push(5);
        bytes.extend_from_slice(&0i64.to_be_bytes());
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 2);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 0);
        assert!(
            Class::new(&bytes).is_ok(),
            "regression premise: noak 0.7.0 accepts the malformed pool boundary"
        );
        let error = inspect_header(
            &bytes,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_invalid_constant_pool_slots")
        );
    }

    #[test]
    fn preflight_cooperatively_stops_when_cancelled_during_pool_scan() {
        let fixture = fixture();
        let token = CancellationToken::new();
        let budget = Budget::with_cancellation_token(limits(u64::MAX), token.clone());
        let mut visited = Vec::new();
        let error = validate_constant_pool_slots_with(&fixture.bytes, &budget, |slot| {
            visited.push(slot);
            if slot == 3 {
                token.cancel();
            }
        })
        .unwrap_err();
        assert!(matches!(error, Error::Cancelled { .. }));
        assert_eq!(visited, [1, 2, 3]);
    }

    #[test]
    fn invalid_magic_keeps_noak_decode_diagnostic_even_if_pool_looks_like_final_long() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xdeadbeef_u32.to_be_bytes());
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 52);
        u16_be(&mut bytes, 2);
        bytes.push(5);
        bytes.extend_from_slice(&0i64.to_be_bytes());
        let error = inspect_header(
            &bytes,
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput { ref code, ref message }
                if code == "classfile_decode"
                    && message.contains("kind=invalid file prefix")
                    && !message.contains("constant-pool tag")
        ));

        let error = inspect_header(
            &[0xca, 0xfe, 0xba],
            &mut Budget::new(limits(u64::MAX)),
            InspectionMode::Strict,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput { ref code, .. } if code == "classfile_decode"
        ));
    }

    // -----------------------------------------------------------------------
    // Reader operand facts and control-flow target validation (P2 1.2)
    // -----------------------------------------------------------------------

    /// Class file with one `method()V` and a constant pool that holds the literal
    /// kinds the `ldc` family can name:
    ///
    /// | index | entry |
    /// | --- | --- |
    /// | 8 | `Integer` 7 |
    /// | 9 | `Float` 1.5 |
    /// | 10 | `Long` 0x1122334455667788 (11 is its reserved slot) |
    /// | 12 | `Double` 2.5 (13 is its reserved slot) |
    /// | 15 | `String` "hello" |
    fn operand_fixture(code: &[u8], handlers: &[(u16, u16, u16, u16)]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 52);
        u16_be(&mut bytes, 16);
        utf8(&mut bytes, b"Test"); // 1
        class_entry(&mut bytes, 1); // 2
        utf8(&mut bytes, b"java/lang/Object"); // 3
        class_entry(&mut bytes, 3); // 4
        utf8(&mut bytes, b"method"); // 5
        utf8(&mut bytes, b"()V"); // 6
        utf8(&mut bytes, b"Code"); // 7
        bytes.push(3);
        bytes.extend_from_slice(&7i32.to_be_bytes()); // 8
        bytes.push(4);
        bytes.extend_from_slice(&1.5f32.to_bits().to_be_bytes()); // 9
        bytes.push(5);
        bytes.extend_from_slice(&0x1122_3344_5566_7788i64.to_be_bytes()); // 10 (+11)
        bytes.push(6);
        bytes.extend_from_slice(&2.5f64.to_bits().to_be_bytes()); // 12 (+13)
        utf8(&mut bytes, b"hello"); // 14
        bytes.push(8);
        u16_be(&mut bytes, 14); // 15
        u16_be(&mut bytes, 0x0021);
        u16_be(&mut bytes, 2);
        u16_be(&mut bytes, 4);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 0);
        u16_be(&mut bytes, 1);
        u16_be(&mut bytes, 0x0009);
        u16_be(&mut bytes, 5);
        u16_be(&mut bytes, 6);
        u16_be(&mut bytes, 1);
        let mut content = Vec::new();
        u16_be(&mut content, 4);
        u16_be(&mut content, 3);
        u32_be(&mut content, u32::try_from(code.len()).unwrap());
        content.extend_from_slice(code);
        u16_be(&mut content, u16::try_from(handlers.len()).unwrap());
        for &(start, end, handler, catch) in handlers {
            u16_be(&mut content, start);
            u16_be(&mut content, end);
            u16_be(&mut content, handler);
            u16_be(&mut content, catch);
        }
        u16_be(&mut content, 0);
        attribute(&mut bytes, 7, &content);
        u16_be(&mut bytes, 0);
        bytes
    }

    /// Code array of [`operand_fixture`], laying out one instruction of every operand
    /// class the 1.2 contract names and choosing every control-flow target so that it
    /// is a real instruction start:
    ///
    /// ```text
    ///  0..2    ldc #8 (Integer 7)        35..37  ldc #99 (index outside the pool)
    ///  2..5    ldc_w #9 (Float 1.5)      37..40  goto +3 -> 40
    ///  5..8    ldc2_w #10 (Long)         40..68  tableswitch default -> 100,
    ///  8..11   ldc2_w #12 (Double)                keys -1/0/1 -> 68/71/72
    /// 11..13   bipush 5                  68..71  ifne +3 -> 71
    /// 13..16   sipush 256                71      iload_0
    /// 16       pop                       72      istore_0
    /// 17..19   iload 2                   73..100 lookupswitch default -> 100,
    /// 19       istore_2                          pairs (-7 -> 0), (9 -> 68)
    /// 20..24   wide iload 2              100     return (code_length 101)
    /// 24..27   iinc 1, 3
    /// 27..33   wide iinc 1, 1
    /// 33..35   ldc #15 (String "hello")
    /// ```
    fn operand_code() -> Vec<u8> {
        let mut code = Vec::new();
        code.extend_from_slice(&[0x12, 0x08]);
        code.extend_from_slice(&[0x13, 0x00, 0x09]);
        code.extend_from_slice(&[0x14, 0x00, 0x0a]);
        code.extend_from_slice(&[0x14, 0x00, 0x0c]);
        code.extend_from_slice(&[0x10, 0x05]);
        code.extend_from_slice(&[0x11, 0x01, 0x00]);
        code.push(0x57);
        code.extend_from_slice(&[0x15, 0x02]);
        code.push(0x3d);
        code.extend_from_slice(&[0xc4, 0x15, 0x00, 0x02]);
        code.extend_from_slice(&[0x84, 0x01, 0x03]);
        code.extend_from_slice(&[0xc4, 0x84, 0x00, 0x01, 0x00, 0x01]);
        code.extend_from_slice(&[0x12, 0x0f]);
        code.extend_from_slice(&[0x12, 0x63]);
        code.extend_from_slice(&[0xa7, 0x00, 0x03]);
        assert_eq!(code.len(), 40, "tableswitch must start at BCI 40");
        code.push(0xaa);
        code.extend_from_slice(&[0, 0, 0]); // padding: BCI 40 is 4-byte aligned
        code.extend_from_slice(&(100i32 - 40).to_be_bytes());
        code.extend_from_slice(&(-1i32).to_be_bytes());
        code.extend_from_slice(&1i32.to_be_bytes());
        for target in [68i32, 71, 72] {
            code.extend_from_slice(&(target - 40).to_be_bytes());
        }
        assert_eq!(code.len(), 68, "ifne must start at BCI 68");
        code.extend_from_slice(&[0x9a, 0x00, 0x03]);
        code.push(0x1a);
        code.push(0x3b);
        assert_eq!(code.len(), 73, "lookupswitch must start at BCI 73");
        code.push(0xab);
        code.extend_from_slice(&[0, 0]); // padding: 1 + 73 needs two bytes to align
        code.extend_from_slice(&(100i32 - 73).to_be_bytes());
        code.extend_from_slice(&2i32.to_be_bytes());
        for (key, target) in [(-7i32, 0i32), (9, 68)] {
            code.extend_from_slice(&key.to_be_bytes());
            code.extend_from_slice(&(target - 73).to_be_bytes());
        }
        assert_eq!(code.len(), 100, "return must start at BCI 100");
        code.push(0xb1);
        code
    }

    /// `method_code_facts` for the single `method()V` of a fixture built by
    /// [`operand_fixture`].
    fn operand_facts(bytes: &[u8]) -> MethodCodeFacts {
        let mut budget = Budget::new(limits(u64::MAX));
        let header = class_facts(bytes, &mut budget).unwrap();
        let method = header
            .methods
            .first()
            .expect("fixture has one method")
            .clone();
        method_code_facts(bytes, &method, &mut budget).unwrap()
    }

    /// The public `inspect_method_bytecode` report of [`operand_code`] is pinned
    /// against the values recorded from the build **before** the crate-private 1.2
    /// operand facts existed (working tree at `6fc1674` plus this test module only).
    ///
    /// 1.2 adds crate-private facts and does not touch the public path, so every
    /// public field and every serialized key must stay exactly as recorded here. The
    /// only field left unasserted is `elapsed_millis`, which reports real elapsed
    /// time; the schema assertion covers its presence.
    ///
    /// An intentional public change must update this test and say why, instead of
    /// loosening it.
    #[test]
    fn public_bytecode_report_is_unchanged_by_reader_operand_facts() {
        let bytes = operand_fixture(&operand_code(), &[(0, 100, 100, 2), (40, 101, 71, 0)]);
        let mut budget = Budget::new(limits(u64::MAX));
        let report = inspect_method_bytecode(&bytes, selector(), &mut budget).unwrap();

        assert_eq!(report.selector, selector());
        assert_eq!((report.max_stack, report.max_locals), (4, 3));
        assert_eq!(report.code_span, ByteSpan::new(137, 101));
        assert_eq!(
            report
                .instructions
                .iter()
                .map(|fact| (
                    fact.bci,
                    fact.opcode,
                    fact.width,
                    fact.span.start,
                    fact.span.length,
                    fact.operands_span.start,
                    fact.operands_span.length,
                    fact.constant_pool_index,
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, 0x12, 2, 137, 2, 138, 1, Some(8)),
                (2, 0x13, 3, 139, 3, 140, 2, Some(9)),
                (5, 0x14, 3, 142, 3, 143, 2, Some(10)),
                (8, 0x14, 3, 145, 3, 146, 2, Some(12)),
                (11, 0x10, 2, 148, 2, 149, 1, None),
                (13, 0x11, 3, 150, 3, 151, 2, None),
                (16, 0x57, 1, 153, 1, 154, 0, None),
                (17, 0x15, 2, 154, 2, 155, 1, None),
                (19, 0x3d, 1, 156, 1, 157, 0, None),
                (20, 0xc4, 4, 157, 4, 158, 3, None),
                (24, 0x84, 3, 161, 3, 162, 2, None),
                (27, 0xc4, 6, 164, 6, 165, 5, None),
                (33, 0x12, 2, 170, 2, 171, 1, Some(15)),
                (35, 0x12, 2, 172, 2, 173, 1, Some(99)),
                (37, 0xa7, 3, 174, 3, 175, 2, None),
                (40, 0xaa, 28, 177, 28, 178, 27, None),
                (68, 0x9a, 3, 205, 3, 206, 2, None),
                (71, 0x1a, 1, 208, 1, 209, 0, None),
                (72, 0x3b, 1, 209, 1, 210, 0, None),
                (73, 0xab, 27, 210, 27, 211, 26, None),
                (100, 0xb1, 1, 237, 1, 238, 0, None),
            ]
        );
        assert_eq!(report.exception_handler_count, 2);
        assert_eq!(
            report.exception_handlers,
            vec![
                ExceptionHandlerFact {
                    ordinal: 0,
                    start_bci: 0,
                    end_bci: 100,
                    handler_bci: 100,
                    catch_type_index: Some(2),
                },
                ExceptionHandlerFact {
                    ordinal: 1,
                    start_bci: 40,
                    end_bci: 101,
                    handler_bci: 71,
                    catch_type_index: None,
                },
            ]
        );
        let ExecutionReport::Complete { usage } = &report.execution else {
            panic!("the pinned fixture decodes completely");
        };
        assert_eq!(
            (
                usage.input_bytes,
                usage.archive_entries,
                usage.entry_bytes,
                usage.read_bytes,
                usage.class_bytes,
                usage.attribute_bytes,
                usage.code_bytes,
                usage.result_items,
                usage.output_bytes,
                usage.nested_depth,
            ),
            (0, 0, 0, 0, 260, 135, 101, 24, 0, 0)
        );
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.verification, VerificationStatus::NotPerformed);
        assert!(report.stopped_at.is_none());

        // No new public field, and no crate-private fact type, reaches this output.
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(
            json_keys(&json),
            [
                "code_span",
                "diagnostics",
                "exception_handler_count",
                "exception_handlers",
                "execution",
                "instructions",
                "max_locals",
                "max_stack",
                "selector",
                "stopped_at",
                "verification",
            ]
        );
        for fact in json["instructions"].as_array().unwrap() {
            assert_eq!(
                json_keys(fact),
                [
                    "bci",
                    "constant_pool_index",
                    "opcode",
                    "operands_span",
                    "span",
                    "width",
                ]
            );
        }
        // The usage schema is additive: 1.3 added its six counted dimensions and the
        // `dependency_depth` high-water mark next to the P0/P1 eleven, and this list is
        // still compared exactly, so a rename or a dropped field fails here.
        assert_eq!(
            json_keys(&json["execution"]["usage"]),
            [
                "analysis_steps",
                "archive_entries",
                "attribute_bytes",
                "class_bytes",
                "class_headers",
                "code_bytes",
                "dependency_depth",
                "elapsed_millis",
                "entry_bytes",
                "input_bytes",
                "ir_edges",
                "ir_items",
                "method_bodies",
                "nested_depth",
                "normalization_clones",
                "output_bytes",
                "read_bytes",
                "result_items",
            ]
        );
        assert_eq!(json_keys(&json["selector"]), ["descriptor", "name"]);
    }

    /// Sorted serialized keys of one JSON object, so a schema change is visible.
    fn json_keys(value: &serde_json::Value) -> Vec<&str> {
        value
            .as_object()
            .expect("serialized reports are JSON objects")
            .keys()
            .map(String::as_str)
            .collect()
    }

    /// Stable code and message of the structured error `control_flow_targets` returns
    /// for one fixture; panics when the fixture validates.
    fn target_error(code: &[u8], handlers: &[(u16, u16, u16, u16)]) -> (String, String) {
        let bytes = operand_fixture(code, handlers);
        let facts = operand_facts(&bytes);
        let error = facts
            .control_flow_targets()
            .expect_err("fixture must be rejected");
        match error {
            Error::InvalidInput { code, message } => (code, message),
            other => panic!("expected a structured invalid-input error, got {other:?}"),
        }
    }

    /// Operand facts of the instruction starting at `bci`, with the lockstep check.
    fn operand_facts_at(facts: &MethodCodeFacts, bci: u32) -> InstructionOperands {
        let position = facts
            .instructions
            .binary_search_by_key(&bci, |fact| fact.bci)
            .unwrap_or_else(|_| panic!("no instruction starts at BCI {bci}"));
        assert_eq!(
            facts.operands.len(),
            facts.instructions.len(),
            "operand facts must stay in lockstep with instructions"
        );
        facts.operands[position].clone()
    }

    #[test]
    fn operand_facts_cover_immediates_locals_increments_and_cp_indices() {
        let facts = operand_facts(&operand_fixture(&operand_code(), &[]));
        assert_eq!(facts.instructions.len(), facts.operands.len());
        assert!(matches!(facts.execution, ExecutionReport::Complete { .. }));
        assert_eq!(facts.instructions.len(), 21);

        // Every operand fact repeats the constant-pool index of its instruction.
        for (fact, operands) in facts.instructions.iter().zip(facts.operands.iter()) {
            assert_eq!(operands.constant_pool_index, fact.constant_pool_index);
        }

        let expected = [
            // `ldc` family: literal kinds become immediates, the `String` entry and an
            // index outside the pool stay reference facts only. The effective opcode equals
            // the raw one for every entry except the two `wide` forms below.
            (
                0,
                InstructionOperands {
                    immediate: Some(ImmediateValue::Int(7)),
                    constant_pool_index: Some(8),
                    effective_opcode: 0x12,
                    ..InstructionOperands::default()
                },
            ),
            (
                2,
                InstructionOperands {
                    immediate: Some(ImmediateValue::Float(1.5f32.to_bits())),
                    constant_pool_index: Some(9),
                    effective_opcode: 0x13,
                    ..InstructionOperands::default()
                },
            ),
            (
                5,
                InstructionOperands {
                    immediate: Some(ImmediateValue::Long(0x1122_3344_5566_7788)),
                    constant_pool_index: Some(10),
                    effective_opcode: 0x14,
                    ..InstructionOperands::default()
                },
            ),
            (
                8,
                InstructionOperands {
                    immediate: Some(ImmediateValue::Double(2.5f64.to_bits())),
                    constant_pool_index: Some(12),
                    effective_opcode: 0x14,
                    ..InstructionOperands::default()
                },
            ),
            (
                11,
                InstructionOperands {
                    immediate: Some(ImmediateValue::Int(5)),
                    effective_opcode: 0x10,
                    ..InstructionOperands::default()
                },
            ),
            (
                13,
                InstructionOperands {
                    immediate: Some(ImmediateValue::Int(256)),
                    effective_opcode: 0x11,
                    ..InstructionOperands::default()
                },
            ),
            // `pop` has no operand at all, so its fact is all-`None` but for its opcode.
            (
                16,
                InstructionOperands {
                    effective_opcode: 0x57,
                    ..InstructionOperands::default()
                },
            ),
            (
                17,
                InstructionOperands {
                    local: Some(LocalOperand {
                        index: 2,
                        wide: false,
                    }),
                    effective_opcode: 0x15,
                    ..InstructionOperands::default()
                },
            ),
            (
                19,
                InstructionOperands {
                    local: Some(LocalOperand {
                        index: 2,
                        wide: false,
                    }),
                    effective_opcode: 0x3d,
                    ..InstructionOperands::default()
                },
            ),
            (
                20,
                InstructionOperands {
                    local: Some(LocalOperand {
                        index: 2,
                        wide: true,
                    }),
                    // The raw opcode of this instruction is the `wide` prefix; the opcode it
                    // wraps is what every classification reads.
                    effective_opcode: 0x15,
                    ..InstructionOperands::default()
                },
            ),
            (
                24,
                InstructionOperands {
                    local: Some(LocalOperand {
                        index: 1,
                        wide: false,
                    }),
                    increment: Some(3),
                    effective_opcode: 0x84,
                    ..InstructionOperands::default()
                },
            ),
            (
                27,
                InstructionOperands {
                    local: Some(LocalOperand {
                        index: 1,
                        wide: true,
                    }),
                    increment: Some(1),
                    effective_opcode: 0x84,
                    ..InstructionOperands::default()
                },
            ),
            (
                33,
                InstructionOperands {
                    constant_pool_index: Some(15),
                    effective_opcode: 0x12,
                    ..InstructionOperands::default()
                },
            ),
            (
                35,
                InstructionOperands {
                    constant_pool_index: Some(99),
                    effective_opcode: 0x12,
                    ..InstructionOperands::default()
                },
            ),
            (
                37,
                InstructionOperands {
                    branch_offset: Some(3),
                    effective_opcode: 0xa7,
                    ..InstructionOperands::default()
                },
            ),
            (
                40,
                InstructionOperands {
                    switch: Some(SwitchOperands::Table {
                        default_offset: 60,
                        low: -1,
                        high: 1,
                        offsets: vec![28, 31, 32],
                    }),
                    effective_opcode: 0xaa,
                    ..InstructionOperands::default()
                },
            ),
            (
                68,
                InstructionOperands {
                    branch_offset: Some(3),
                    effective_opcode: 0x9a,
                    ..InstructionOperands::default()
                },
            ),
            (
                71,
                InstructionOperands {
                    local: Some(LocalOperand {
                        index: 0,
                        wide: false,
                    }),
                    effective_opcode: 0x1a,
                    ..InstructionOperands::default()
                },
            ),
            (
                72,
                InstructionOperands {
                    local: Some(LocalOperand {
                        index: 0,
                        wide: false,
                    }),
                    effective_opcode: 0x3b,
                    ..InstructionOperands::default()
                },
            ),
            (
                73,
                InstructionOperands {
                    switch: Some(SwitchOperands::Lookup {
                        default_offset: 27,
                        pairs: vec![(-7, -73), (9, -5)],
                    }),
                    effective_opcode: 0xab,
                    ..InstructionOperands::default()
                },
            ),
            (
                100,
                InstructionOperands {
                    effective_opcode: 0xb1,
                    ..InstructionOperands::default()
                },
            ),
        ];
        for (bci, operands) in expected {
            assert_eq!(operand_facts_at(&facts, bci), operands, "BCI {bci}");
        }
        // The effective opcode of every other instruction of the fixture is its raw opcode:
        // the fixture's whole point is that only a `wide` form has two opcodes.
        for (fact, operands) in facts.instructions.iter().zip(facts.operands.iter()) {
            if !matches!(fact.bci, 20 | 27) {
                assert_eq!(
                    operands.effective_opcode, fact.opcode,
                    "BCI {} of a fixture whose only `wide` forms are at 20 and 27",
                    fact.bci
                );
            }
        }
        // The raw opcode of the two `wide` forms is untouched, exactly like their width and
        // their span: the public fact keeps the prefix.
        assert_eq!(
            facts
                .instructions
                .iter()
                .filter(|fact| fact.opcode == 0xc4)
                .map(|fact| fact.bci)
                .collect::<Vec<_>>(),
            vec![20, 27]
        );
    }

    #[test]
    fn control_flow_targets_convert_branches_switches_and_handlers() {
        let facts = operand_facts(&operand_fixture(
            &operand_code(),
            &[(0, 100, 100, 2), (40, 101, 71, 0)],
        ));
        let targets = facts.control_flow_targets().unwrap();
        assert_eq!(
            targets,
            vec![
                ControlFlowTarget {
                    instruction_bci: 37,
                    kind: ControlFlowTargetKind::Branch { offset: 3 },
                    target_bci: 40,
                },
                ControlFlowTarget {
                    instruction_bci: 40,
                    kind: ControlFlowTargetKind::SwitchDefault,
                    target_bci: 100,
                },
                ControlFlowTarget {
                    instruction_bci: 40,
                    kind: ControlFlowTargetKind::SwitchCase { index: 0, key: -1 },
                    target_bci: 68,
                },
                ControlFlowTarget {
                    instruction_bci: 40,
                    kind: ControlFlowTargetKind::SwitchCase { index: 1, key: 0 },
                    target_bci: 71,
                },
                ControlFlowTarget {
                    instruction_bci: 40,
                    kind: ControlFlowTargetKind::SwitchCase { index: 2, key: 1 },
                    target_bci: 72,
                },
                ControlFlowTarget {
                    instruction_bci: 68,
                    kind: ControlFlowTargetKind::Branch { offset: 3 },
                    target_bci: 71,
                },
                ControlFlowTarget {
                    instruction_bci: 73,
                    kind: ControlFlowTargetKind::SwitchDefault,
                    target_bci: 100,
                },
                ControlFlowTarget {
                    instruction_bci: 73,
                    kind: ControlFlowTargetKind::SwitchCase { index: 0, key: -7 },
                    target_bci: 0,
                },
                ControlFlowTarget {
                    instruction_bci: 73,
                    kind: ControlFlowTargetKind::SwitchCase { index: 1, key: 9 },
                    target_bci: 68,
                },
                // `end == code_length` is the legal half-open end of the code array.
                ControlFlowTarget {
                    instruction_bci: 100,
                    kind: ControlFlowTargetKind::Handler { ordinal: 0 },
                    target_bci: 100,
                },
                ControlFlowTarget {
                    instruction_bci: 71,
                    kind: ControlFlowTargetKind::Handler { ordinal: 1 },
                    target_bci: 71,
                },
            ]
        );
    }

    #[test]
    fn tableswitch_keys_hold_their_low_high_bounds() {
        // A key interval next to `i32::MAX`: the key of a case is `low + index`, and the
        // interval must not overflow the i32 key space.
        let mut code = vec![0xaa, 0, 0, 0];
        code.extend_from_slice(&28i32.to_be_bytes());
        code.extend_from_slice(&(i32::MAX - 2).to_be_bytes());
        code.extend_from_slice(&i32::MAX.to_be_bytes());
        for _ in 0..3 {
            code.extend_from_slice(&28i32.to_be_bytes());
        }
        code.push(0xb1);
        let facts = operand_facts(&operand_fixture(&code, &[]));
        assert_eq!(
            operand_facts_at(&facts, 0).switch,
            Some(SwitchOperands::Table {
                default_offset: 28,
                low: i32::MAX - 2,
                high: i32::MAX,
                offsets: vec![28, 28, 28],
            })
        );
        assert_eq!(
            facts.control_flow_targets().unwrap(),
            vec![
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::SwitchDefault,
                    target_bci: 28,
                },
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::SwitchCase {
                        index: 0,
                        key: i32::MAX - 2,
                    },
                    target_bci: 28,
                },
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::SwitchCase {
                        index: 1,
                        key: i32::MAX - 1,
                    },
                    target_bci: 28,
                },
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::SwitchCase {
                        index: 2,
                        key: i32::MAX,
                    },
                    target_bci: 28,
                },
            ]
        );
    }

    #[test]
    fn implicit_and_wide_locals_keep_their_index() {
        let code = [
            0x15, 0x07, // iload 7
            0x16, 0x08, // lload 8
            0x17, 0x09, // fload 9
            0x18, 0x0a, // dload 10
            0x19, 0x0b, // aload 11
            0x36, 0x0c, // istore 12
            0x37, 0x0d, // lstore 13
            0x38, 0x0e, // fstore 14
            0x39, 0x0f, // dstore 15
            0x3a, 0x10, // astore 16
            0x1d, // iload_3
            0x21, // lload_3
            0x25, // fload_3
            0x29, // dload_3
            0x2d, // aload_3
            0x3e, // istore_3
            0x42, // lstore_3
            0x46, // fstore_3
            0x4a, // dstore_3
            0x4e, // astore_3
            0xa9, 0x11, // ret 17
            0xc4, 0x15, 0x00, 0x12, // wide iload 18
            0xc4, 0x39, 0x00, 0x13, // wide dstore 19
            0xc4, 0xa9, 0x00, 0x14, // wide ret 20
            0xc4, 0x3a, 0x00, 0x15, // wide astore 21
            0xc4, 0x17, 0x00, 0x16, // wide fload 22
            0xb1, // return
        ];
        let facts = operand_facts(&operand_fixture(&code, &[]));
        let expected: Vec<(u32, Option<LocalOperand>)> = vec![
            (0, local(7, false)),
            (2, local(8, false)),
            (4, local(9, false)),
            (6, local(10, false)),
            (8, local(11, false)),
            (10, local(12, false)),
            (12, local(13, false)),
            (14, local(14, false)),
            (16, local(15, false)),
            (18, local(16, false)),
            (20, local(3, false)),
            (21, local(3, false)),
            (22, local(3, false)),
            (23, local(3, false)),
            (24, local(3, false)),
            (25, local(3, false)),
            (26, local(3, false)),
            (27, local(3, false)),
            (28, local(3, false)),
            (29, local(3, false)),
            (30, local(17, false)),
            (32, local(18, true)),
            (36, local(19, true)),
            (40, local(20, true)),
            (44, local(21, true)),
            (48, local(22, true)),
            (52, None),
        ];
        for (bci, operands) in expected {
            assert_eq!(operand_facts_at(&facts, bci).local, operands, "BCI {bci}");
        }
    }

    /// A local expectation, so the table above reads as one column.
    fn local(index: u16, wide: bool) -> Option<LocalOperand> {
        Some(LocalOperand { index, wide })
    }

    #[test]
    fn stopped_bodies_keep_instruction_and_operand_lockstep() {
        let bytes = operand_fixture(&operand_code(), &[]);
        let mut header_budget = Budget::new(limits(u64::MAX));
        let header = class_facts(&bytes, &mut header_budget).unwrap();
        let method = header.methods.first().unwrap().clone();
        let mut limited = limits(u64::MAX);
        limited.code_bytes = 5;
        let facts = method_code_facts(&bytes, &method, &mut Budget::new(limited)).unwrap();
        assert!(matches!(facts.execution, ExecutionReport::Partial { .. }));
        assert!(facts.stopped_at.is_some());
        assert_eq!(facts.instructions.len(), 2);
        assert_eq!(facts.operands.len(), 2);
        assert_eq!(
            facts.operands[1].immediate,
            Some(ImmediateValue::Float(1.5f32.to_bits()))
        );
    }

    #[test]
    fn branch_targets_inside_operands_are_rejected_with_the_sourcing_bci() {
        // `goto +1` at BCI 2 targets BCI 3, the first operand byte of the `goto` itself.
        let code = [0x10, 0x05, 0xa7, 0x00, 0x01, 0xb1];
        let bytes = operand_fixture(&code, &[]);
        // The body itself decodes: the rejection is target validation, not a decode stop.
        let facts = operand_facts(&bytes);
        assert!(matches!(facts.execution, ExecutionReport::Complete { .. }));
        let (error_code, message) = target_error(&code, &[]);
        assert_eq!(error_code, "classfile_instruction_invalid_target");
        assert!(
            message.contains("branch from BCI 2 with offset 1"),
            "{message}"
        );
        assert!(message.contains("target 3"), "{message}");
    }

    #[test]
    fn targets_outside_the_code_array_are_rejected() {
        // `goto +99` at BCI 2 reaches BCI 101 of a 6-byte code array.
        let (code, message) = target_error(&[0x10, 0x05, 0xa7, 0x00, 0x63, 0xb1], &[]);
        assert_eq!(code, "classfile_instruction_target_out_of_bounds");
        assert!(
            message.contains("branch from BCI 2 with offset 99"),
            "{message}"
        );
        assert!(message.contains("target 101 is outside 0..6"), "{message}");

        // A negative relative offset leaves the code array below BCI 0.
        let (code, message) = target_error(&[0xa7, 0xff, 0xfb], &[]);
        assert_eq!(code, "classfile_instruction_target_out_of_bounds");
        assert!(
            message.contains("relative offset -5 at BCI 0 targets a BCI before 0"),
            "{message}"
        );

        // A handler entry outside the code array.
        let (code, message) = target_error(&[0xb1], &[(0, 1, 9, 0)]);
        assert_eq!(code, "classfile_instruction_target_out_of_bounds");
        assert!(
            message.contains("exception handler 0 entry target 9 is outside 0..1"),
            "{message}"
        );
    }

    #[test]
    fn handler_entry_inside_operands_is_rejected() {
        // `bipush 5` at BCI 0, `return` at 2: BCI 1 is an operand byte, and the
        // protected range ends exactly at `code_length`.
        let (code, message) = target_error(&[0x10, 0x05, 0xb1], &[(0, 3, 1, 0)]);
        assert_eq!(code, "classfile_instruction_invalid_target");
        assert!(
            message.contains("exception handler 0 entry target 1 is not an instruction start"),
            "{message}"
        );
    }

    #[test]
    fn handler_ranges_must_start_before_they_end_and_stay_inside_the_code() {
        // `start > end` on a legal handler entry.
        let (code, message) = target_error(&[0x10, 0x05, 0xb1], &[(2, 1, 2, 0)]);
        assert_eq!(code, "classfile_exception_range_invalid");
        assert!(
            message.contains("exception handler 0 protected range 2..1 starts after its end"),
            "{message}"
        );

        // `end > code_length` on a legal handler entry.
        let (code, message) = target_error(&[0xb1], &[(0, 4, 0, 0)]);
        assert_eq!(code, "classfile_exception_range_invalid");
        assert!(
            message.contains("exception handler 0 protected range 0..4 ends past code_length 1"),
            "{message}"
        );

        // Half-open and legal: `end == code_length`, handler on the last instruction.
        let facts = operand_facts(&operand_fixture(&[0x10, 0x05, 0xb1], &[(0, 3, 2, 0)]));
        assert_eq!(
            facts.control_flow_targets().unwrap(),
            vec![ControlFlowTarget {
                instruction_bci: 2,
                kind: ControlFlowTargetKind::Handler { ordinal: 0 },
                target_bci: 2,
            }]
        );
    }

    #[test]
    fn switch_case_targets_are_validated_like_branches() {
        // `tableswitch` at BCI 0 with one case pointing past the 21-byte code array.
        let mut code = vec![0xaa, 0, 0, 0];
        code.extend_from_slice(&20i32.to_be_bytes());
        code.extend_from_slice(&0i32.to_be_bytes());
        code.extend_from_slice(&0i32.to_be_bytes());
        code.extend_from_slice(&100i32.to_be_bytes());
        code.push(0xb1);
        let (code, message) = target_error(&code, &[]);
        assert_eq!(code, "classfile_instruction_target_out_of_bounds");
        assert!(message.contains("switch case 0 from BCI 0"), "{message}");
        assert!(message.contains("target 100 is outside 0..21"), "{message}");

        // A case that lands inside the switch payload is not an instruction start.
        let mut code = vec![0xaa, 0, 0, 0];
        code.extend_from_slice(&20i32.to_be_bytes());
        code.extend_from_slice(&0i32.to_be_bytes());
        code.extend_from_slice(&0i32.to_be_bytes());
        code.extend_from_slice(&4i32.to_be_bytes());
        code.push(0xb1);
        let (code, message) = target_error(&code, &[]);
        assert_eq!(code, "classfile_instruction_invalid_target");
        assert!(message.contains("switch case 0 from BCI 0"), "{message}");
        assert!(
            message.contains("target 4 is not an instruction start"),
            "{message}"
        );
    }

    #[test]
    fn relative_target_arithmetic_is_checked_in_both_directions() {
        let error = relative_target_bci(u32::MAX, 1).unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_code_span_overflow")
        );
        let error = relative_target_bci(u32::MAX, i32::MAX).unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_code_span_overflow")
        );
        let error = relative_target_bci(0, -1).unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_instruction_target_out_of_bounds")
        );
        let error = relative_target_bci(1, i32::MIN).unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, .. } if code == "classfile_instruction_target_out_of_bounds")
        );
        assert_eq!(relative_target_bci(10, -10).unwrap(), 0);
        assert_eq!(relative_target_bci(10, 5).unwrap(), 15);
    }

    #[test]
    fn switch_operands_reject_a_region_that_does_not_match_the_decoded_shape() {
        // Two entries need `1 + 3 + 12 + 8` bytes; the recorded region is one truncated
        // instruction, which is what a width/byte disagreement would look like.
        let error = table_switch_operands(0, &[0xaa, 0, 0, 0], 0, 0, 1).unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, ref message }
                if code == "classfile_instruction_shape_mismatch"
                    && message.contains("tableswitch at BCI 0 has 4 recorded bytes but 2 entries need 24")),
            "{error:?}"
        );

        // A pair list whose last pair is cut off.
        let bytes = [0xab_u8, 0, 0, 0];
        let error = lookup_switch_operands(0, &bytes, 0, 1).unwrap_err();
        assert!(matches!(error, Error::InvalidInput { ref code, .. }
                if code == "classfile_instruction_shape_mismatch"));

        // `high < low` is a range violation, not a shape violation.
        let error = table_switch_operands(0, &[], 0, 5, 4).unwrap_err();
        assert!(matches!(error, Error::InvalidInput { ref code, .. }
                if code == "classfile_instruction_invalid_range"));

        // A single operand that leaves the recorded bytes.
        let error = switch_i32(&[0, 0, 0], 7, 0).unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput { ref code, ref message }
                if code == "classfile_instruction_shape_mismatch"
                    && message.contains("BCI 7")),
            "{error:?}"
        );
    }

    #[test]
    fn switch_defaults_are_validated_like_cases() {
        // `tableswitch` at BCI 0, one case on the trailing `return` at BCI 20; the
        // default points at BCI 1, inside the switch's own padding.
        let mut code = vec![0xaa, 0, 0, 0];
        code.extend_from_slice(&1i32.to_be_bytes());
        code.extend_from_slice(&0i32.to_be_bytes());
        code.extend_from_slice(&0i32.to_be_bytes());
        code.extend_from_slice(&20i32.to_be_bytes());
        code.push(0xb1);
        let (error_code, message) = target_error(&code, &[]);
        assert_eq!(error_code, "classfile_instruction_invalid_target");
        assert!(message.contains("switch default from BCI 0"), "{message}");
        assert!(
            message.contains("target 1 is not an instruction start"),
            "{message}"
        );

        // `lookupswitch` at BCI 0, one pair on the trailing `return`; the default is far
        // outside the code array.
        let mut code = vec![0xab, 0, 0, 0];
        code.extend_from_slice(&1000i32.to_be_bytes());
        code.extend_from_slice(&1i32.to_be_bytes());
        code.extend_from_slice(&0i32.to_be_bytes());
        code.extend_from_slice(&20i32.to_be_bytes());
        code.push(0xb1);
        let (error_code, message) = target_error(&code, &[]);
        assert_eq!(error_code, "classfile_instruction_target_out_of_bounds");
        assert!(message.contains("switch default from BCI 0"), "{message}");
        assert!(
            message.contains("target 1000 is outside 0..21"),
            "{message}"
        );

        // With a legal default, the same two shapes validate, so the cases above fail
        // for the default and not because the switch itself is malformed.
        let mut valid = vec![0xaa, 0, 0, 0];
        valid.extend_from_slice(&20i32.to_be_bytes());
        valid.extend_from_slice(&0i32.to_be_bytes());
        valid.extend_from_slice(&0i32.to_be_bytes());
        valid.extend_from_slice(&20i32.to_be_bytes());
        valid.push(0xb1);
        let facts = operand_facts(&operand_fixture(&valid, &[]));
        assert_eq!(
            facts.control_flow_targets().unwrap(),
            vec![
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::SwitchDefault,
                    target_bci: 20,
                },
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::SwitchCase { index: 0, key: 0 },
                    target_bci: 20,
                },
            ]
        );

        let mut valid = vec![0xab, 0, 0, 0];
        valid.extend_from_slice(&20i32.to_be_bytes());
        valid.extend_from_slice(&1i32.to_be_bytes());
        valid.extend_from_slice(&0i32.to_be_bytes());
        valid.extend_from_slice(&20i32.to_be_bytes());
        valid.push(0xb1);
        let facts = operand_facts(&operand_fixture(&valid, &[]));
        assert_eq!(
            facts.control_flow_targets().unwrap(),
            vec![
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::SwitchDefault,
                    target_bci: 20,
                },
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::SwitchCase { index: 0, key: 0 },
                    target_bci: 20,
                },
            ]
        );
    }

    #[test]
    fn jsr_operands_and_targets_use_the_same_validation() {
        let code = [
            0xa8, 0x00, 0x03, // 0: jsr +3 -> BCI 3
            0xc9, 0x00, 0x00, 0x00, 0x05, // 3: jsr_w +5 -> BCI 8
            0xb1, // 8: return
        ];
        let facts = operand_facts(&operand_fixture(&code, &[]));
        assert_eq!(operand_facts_at(&facts, 0).branch_offset, Some(3));
        assert_eq!(operand_facts_at(&facts, 3).branch_offset, Some(5));
        assert_eq!(
            facts.control_flow_targets().unwrap(),
            vec![
                ControlFlowTarget {
                    instruction_bci: 0,
                    kind: ControlFlowTargetKind::Branch { offset: 3 },
                    target_bci: 3,
                },
                ControlFlowTarget {
                    instruction_bci: 3,
                    kind: ControlFlowTargetKind::Branch { offset: 5 },
                    target_bci: 8,
                },
            ]
        );

        // Both subroutines go through the same target rules as any other branch.
        let (error_code, message) = target_error(&[0xa8, 0x00, 0x01, 0xb1], &[]);
        assert_eq!(error_code, "classfile_instruction_invalid_target");
        assert!(
            message.contains("branch from BCI 0 with offset 1"),
            "{message}"
        );
        let (error_code, message) = target_error(&[0xc9, 0x00, 0x00, 0x00, 0x63, 0xb1], &[]);
        assert_eq!(error_code, "classfile_instruction_target_out_of_bounds");
        assert!(
            message.contains("branch from BCI 0 with offset 99"),
            "{message}"
        );
        // A negative `jsr_w` offset leaves the code array below BCI 0.
        let (error_code, message) = target_error(&[0xc9, 0xff, 0xff, 0xff, 0xfb], &[]);
        assert_eq!(error_code, "classfile_instruction_target_out_of_bounds");
        assert!(
            message.contains("relative offset -5 at BCI 0 targets a BCI before 0"),
            "{message}"
        );
    }

    #[test]
    fn signed_operands_keep_their_sign() {
        let code = [
            0x10, 0xfb, // 0: bipush -5
            0x11, 0xfe, 0xd4, // 2: sipush -300
            0x84, 0x03, 0xfe, // 5: iinc 3, -2
            0xc4, 0x84, 0x00, 0x04, 0xff, 0xfb, // 8: wide iinc 4, -5
            0xb1, // 14: return
        ];
        let facts = operand_facts(&operand_fixture(&code, &[]));
        assert_eq!(
            operand_facts_at(&facts, 0).immediate,
            Some(ImmediateValue::Int(-5))
        );
        assert_eq!(
            operand_facts_at(&facts, 2).immediate,
            Some(ImmediateValue::Int(-300))
        );
        assert_eq!(
            operand_facts_at(&facts, 5),
            InstructionOperands {
                local: local(3, false),
                increment: Some(-2),
                effective_opcode: 0x84,
                ..InstructionOperands::default()
            }
        );
        assert_eq!(
            operand_facts_at(&facts, 8),
            InstructionOperands {
                local: local(4, true),
                increment: Some(-5),
                // The `wide iinc` of a local above 255: the prefix is the raw opcode, `iinc`
                // is the effective one.
                effective_opcode: 0x84,
                ..InstructionOperands::default()
            }
        );
        // The negative operands decode; nothing in this body is a control-flow target.
        assert_eq!(facts.control_flow_targets().unwrap(), Vec::new());
    }

    #[test]
    fn operand_facts_retain_the_opcode_each_wide_form_wraps() {
        // All twelve wrappings JVMS 6.5 allows, one instruction each, on real bytes: the raw
        // opcode of every one of them is the `wide` prefix, the width is the wide one, and the
        // effective opcode is the opcode the prefix wraps (`wide iinc` is the six-byte form).
        let cases: &[(u8, u32, u16)] = &[
            (0x15, 4, 2),   // wide iload
            (0x16, 4, 300), // wide lload
            (0x17, 4, 4),   // wide fload
            (0x18, 4, 5),   // wide dload
            (0x19, 4, 6),   // wide aload
            (0x36, 4, 7),   // wide istore
            (0x37, 4, 8),   // wide lstore
            (0x38, 4, 9),   // wide fstore
            (0x39, 4, 10),  // wide dstore
            (0x3a, 4, 11),  // wide astore
            (0xa9, 4, 12),  // wide ret
            (0x84, 6, 13),  // wide iinc
        ];
        let mut code = Vec::new();
        let mut expected: Vec<(u32, u8, u32, u16)> = Vec::new();
        for &(wrapped, width, index) in cases {
            expected.push((u32::try_from(code.len()).unwrap(), wrapped, width, index));
            code.push(0xc4);
            code.push(wrapped);
            code.extend_from_slice(&index.to_be_bytes());
            if wrapped == 0x84 {
                code.extend_from_slice(&0i16.to_be_bytes()); // the increment of `wide iinc 13, 0`
            }
        }
        let bytes = test_class::single_method(52, 8, 512, &code);
        let facts = operand_facts(&bytes);
        assert!(matches!(facts.execution, ExecutionReport::Complete { .. }));
        assert_eq!(facts.instructions.len(), cases.len());
        for (bci, wrapped, width, index) in expected {
            let fact = facts
                .instructions
                .iter()
                .find(|fact| fact.bci == bci)
                .unwrap_or_else(|| panic!("no instruction at BCI {bci}"));
            let operands = operand_facts_at(&facts, bci);
            assert_eq!(fact.opcode, 0xc4, "BCI {bci} keeps the raw prefix");
            assert_eq!(fact.width, width, "BCI {bci}");
            assert_eq!(
                operands.effective_opcode, wrapped,
                "BCI {bci} must classify as the opcode it wraps"
            );
            assert_eq!(
                operands.local,
                Some(LocalOperand { index, wide: true }),
                "BCI {bci} names the local the wrapped form names"
            );
        }
        // The two opcodes are separate facts: none of these twelve is malformed, so the body
        // decodes completely and every wrapped opcode is one of the twelve.
        assert_eq!(facts.stopped_at, None);
        assert!(
            cases.iter().all(|(wrapped, _, _)| *wrapped != 0xc4),
            "no wrapping is itself a prefix"
        );
    }

    #[test]
    fn operand_facts_retain_array_types_dimensions_and_interface_counts() {
        // Array creation and interface calls, on real bytes. `atype`, `dimensions` and `count`
        // are payloads of their own kind: they are never `immediate`, and the reader records
        // them exactly as encoded — `dimensions = 0` and a `count` that disagrees with the
        // descriptor are facts for the verifier (4.x/5.1) to judge, not reader errors.
        let array_types = [4u8, 5, 6, 7, 8, 9, 10, 11];
        let dimensions = [0u8, 2, 255];
        let counts = [3u8, 2, 0];
        let mut code = Vec::new();
        let mut new_arrays = Vec::new();
        for atype in array_types {
            new_arrays.push((u32::try_from(code.len()).unwrap(), atype));
            code.extend_from_slice(&[0xbc, atype]);
        }
        let mut multi_arrays = Vec::new();
        for dimension in dimensions {
            multi_arrays.push((u32::try_from(code.len()).unwrap(), dimension));
            code.push(0xc5);
            code.extend_from_slice(&9u16.to_be_bytes()); // the `[[I` class entry
            code.push(dimension);
        }
        let mut interface_calls = Vec::new();
        for count in counts {
            interface_calls.push((u32::try_from(code.len()).unwrap(), count));
            code.push(0xb9);
            code.extend_from_slice(&15u16.to_be_bytes()); // `run:(J)V` of the fixture pool
            code.push(count);
            code.push(0);
        }
        code.push(0xb1); // return

        let bytes = test_class::single_method(52, 8, 8, &code);
        let facts = operand_facts(&bytes);
        assert!(matches!(facts.execution, ExecutionReport::Complete { .. }));
        assert_eq!(facts.stopped_at, None);
        assert_eq!(
            facts.instructions.len(),
            array_types.len() + dimensions.len() + counts.len() + 1
        );

        for (bci, atype) in new_arrays {
            let operands = operand_facts_at(&facts, bci);
            assert_eq!(operands.effective_opcode, 0xbc, "BCI {bci}");
            assert_eq!(operands.atype, Some(atype), "BCI {bci}");
            assert_eq!(operands.immediate, None, "an element type is not a value");
            assert_eq!(operands.constant_pool_index, None, "BCI {bci}");
        }
        for (bci, dimension) in multi_arrays {
            let operands = operand_facts_at(&facts, bci);
            assert_eq!(operands.effective_opcode, 0xc5, "BCI {bci}");
            assert_eq!(operands.dimensions, Some(dimension), "BCI {bci}");
            assert_eq!(operands.immediate, None, "BCI {bci}");
            assert_eq!(operands.constant_pool_index, Some(9), "BCI {bci}");
        }
        for (bci, count) in interface_calls {
            let operands = operand_facts_at(&facts, bci);
            assert_eq!(operands.effective_opcode, 0xb9, "BCI {bci}");
            assert_eq!(operands.interface_count, Some(count), "BCI {bci}");
            assert_eq!(operands.immediate, None, "BCI {bci}");
            assert_eq!(operands.constant_pool_index, Some(15), "BCI {bci}");
        }

        // The three facts belong to three opcodes: none of them leaks onto any other
        // instruction of the same body.
        for (position, fact) in facts.instructions.iter().enumerate() {
            let operands = &facts.operands[position];
            match fact.opcode {
                0xbc => {
                    assert!(operands.dimensions.is_none() && operands.interface_count.is_none())
                }
                0xc5 => assert!(operands.atype.is_none() && operands.interface_count.is_none()),
                0xb9 => assert!(operands.atype.is_none() && operands.dimensions.is_none()),
                _ => assert!(
                    operands.atype.is_none()
                        && operands.dimensions.is_none()
                        && operands.interface_count.is_none(),
                    "BCI {} carries no array or invocation payload",
                    fact.bci
                ),
            }
        }
    }

    #[test]
    fn an_element_type_outside_the_encoded_range_stops_the_decode() {
        // `atype` is part of the instruction's encoding, so a code outside JVMS' `4..=11` is
        // not an operand fact at all: the instruction does not decode, the reader stops at its
        // BCI, and the reliable prefix keeps only what came before it. The adapter does not
        // guess a fact for an instruction noak refused, and it does not re-decode the bytes.
        for atype in [0u8, 3, 12, 255] {
            let code = [0x00, 0xbc, atype, 0xb1]; // nop; newarray <atype>; return
            let bytes = test_class::single_method(52, 8, 8, &code);
            let facts = operand_facts(&bytes);
            assert_eq!(facts.instructions.len(), 1, "atype {atype}");
            assert_eq!(facts.operands.len(), 1, "atype {atype}");
            assert_eq!(facts.operands[0].effective_opcode, 0x00, "atype {atype}");
            assert_eq!(facts.operands[0].atype, None, "atype {atype}");
            match &facts.stopped_at {
                Some(BytecodeStop::Instructions { bci, code, .. }) => {
                    assert_eq!(*bci, 1, "the stop is the undecodable instruction");
                    assert_eq!(code, "classfile_instruction_decode", "atype {atype}");
                }
                other => panic!("expected a decode stop at BCI 1, got {other:?}"),
            }
        }
    }

    #[test]
    fn targets_at_code_length_are_out_of_bounds_not_unstarted() {
        // `goto +3` in a 3-byte code array targets exactly `code_length`: one past the
        // last byte. That is a bounds error, not an "inside an operand" error.
        let (error_code, message) = target_error(&[0xa7, 0x00, 0x03], &[]);
        assert_eq!(error_code, "classfile_instruction_target_out_of_bounds");
        assert!(message.contains("target 3 is outside 0..3"), "{message}");

        // BCI `code_length - 1` is still a legal target: `goto +3` reaches the `return`.
        let facts = operand_facts(&operand_fixture(&[0xa7, 0x00, 0x03, 0xb1], &[]));
        assert_eq!(facts.control_flow_targets().unwrap()[0].target_bci, 3);

        // A handler entry exactly at `code_length` is not a target either, and the
        // range itself is legal (`end == code_length`).
        let (error_code, message) = target_error(&[0xb1], &[(0, 1, 1, 0)]);
        assert_eq!(error_code, "classfile_instruction_target_out_of_bounds");
        assert!(
            message.contains("exception handler 0 entry target 1 is outside 0..1"),
            "{message}"
        );
    }

    #[test]
    fn protected_range_endpoints_must_be_instruction_starts() {
        // `bipush 5` at BCI 0 (its operand is BCI 1), `return` at BCI 2.
        let code = [0x10, 0x05, 0xb1];

        // A range start inside an operand.
        let (error_code, message) = target_error(&code, &[(1, 3, 2, 0)]);
        assert_eq!(error_code, "classfile_instruction_invalid_target");
        assert!(
            message.contains(
                "exception handler 0 protected range start 1 is not an instruction start"
            ),
            "{message}"
        );

        // A range end inside an operand.
        let (error_code, message) = target_error(&code, &[(0, 1, 2, 0)]);
        assert_eq!(error_code, "classfile_instruction_invalid_target");
        assert!(
            message.contains(
                "protected range end 1 is neither an instruction start nor code_length 3"
            ),
            "{message}"
        );

        // An aligned range keeps validating, including the legal half-open end.
        let facts = operand_facts(&operand_fixture(&code, &[(0, 3, 2, 0)]));
        assert_eq!(
            facts.control_flow_targets().unwrap(),
            vec![ControlFlowTarget {
                instruction_bci: 2,
                kind: ControlFlowTargetKind::Handler { ordinal: 0 },
                target_bci: 2,
            }]
        );
    }

    #[test]
    fn protected_ranges_must_not_be_empty() {
        let code = [0x10, 0x05, 0xb1];

        // An empty range on an instruction start: JVMS 4.7.3 wants `start < end`.
        let (error_code, message) = target_error(&code, &[(2, 2, 2, 0)]);
        assert_eq!(error_code, "classfile_exception_range_invalid");
        assert!(
            message.contains("exception handler 0 protected range at BCI 2 is empty"),
            "{message}"
        );

        // An empty range inside an operand stays a range error: the relation is checked
        // before the endpoints, so the two failure kinds stay distinguishable.
        let (error_code, message) = target_error(&code, &[(1, 1, 2, 0)]);
        assert_eq!(error_code, "classfile_exception_range_invalid");
        assert!(message.contains("is empty"), "{message}");

        // An inverted range keeps its own message.
        let (error_code, message) = target_error(&code, &[(2, 1, 2, 0)]);
        assert_eq!(error_code, "classfile_exception_range_invalid");
        assert!(
            message.contains("protected range 2..1 starts after its end"),
            "{message}"
        );
    }

    /// Every class fixture in the repository still validates: the tightened handler
    /// rules must not reject real code, and real historical subroutines must go through
    /// the same target checks.
    ///
    /// The counts are the measured population — 37 classes, 138 bodies, 44 exception table
    /// records, 86 branch/switch targets, 8 `jsr`/`jsr_w` instructions (the ECJ 4.6.1 45–48
    /// `finally` codegen; neither the P3 samples nor the P4 modern samples contain a
    /// subroutine) — so a fixture that silently stops being visited, or a body that stops
    /// decoding, fails here instead of quietly shrinking the sweep. The P4 modern samples raise
    /// the first two counts by their own files and bodies and leave the last three where they
    /// were, which is also how the sweep states that a javac 23.0.1 class at major 60 or 61 is
    /// read by the same structural path as every earlier fixture. The one count the P3 samples do
    /// not move is the last: `p3-local-rewrite`, `p3-scope`, `p3-handlers` and `p3-corpus` are
    /// javac 23.0.1 output, which has no subroutines at all, so every `jsr` in the sweep is the ECJ
    /// corpus's.
    #[test]
    fn repository_class_fixtures_validate_without_false_target_rejections() {
        let fixtures = class_fixture_paths();
        let mut bodies = 0u32;
        let mut handler_records = 0u32;
        let mut branch_targets = 0u32;
        let mut subroutines = 0u32;
        for path in &fixtures {
            let bytes = std::fs::read(path).unwrap();
            let mut budget = Budget::new(limits(u64::MAX));
            let header = class_facts(&bytes, &mut budget)
                .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
            for method in &header.methods {
                let facts = match method_code_facts(&bytes, method, &mut budget) {
                    Ok(facts) => facts,
                    Err(Error::Unsupported { code, .. })
                        if code == "classfile_method_has_no_code" =>
                    {
                        continue;
                    }
                    Err(error) => panic!("{}: {error:?}", path.display()),
                };
                assert!(
                    matches!(facts.execution, ExecutionReport::Complete { .. }),
                    "{}: a fixture body must decode completely",
                    path.display()
                );
                let targets = facts
                    .control_flow_targets()
                    .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
                bodies += 1;
                handler_records += facts.exception_handlers.len() as u32;
                branch_targets += targets
                    .iter()
                    .filter(|target| !matches!(target.kind, ControlFlowTargetKind::Handler { .. }))
                    .count() as u32;
                for fact in &facts.instructions {
                    if matches!(fact.opcode, 0xa8 | 0xc9) {
                        subroutines += 1;
                        assert!(
                            targets.iter().any(|target| {
                                target.instruction_bci == fact.bci
                                    && matches!(target.kind, ControlFlowTargetKind::Branch { .. })
                            }),
                            "{}: the subroutine at BCI {} lost its branch target",
                            path.display(),
                            fact.bci
                        );
                    }
                }
            }
        }
        assert_eq!(
            (
                fixtures.len(),
                bodies,
                handler_records,
                branch_targets,
                subroutines
            ),
            (37, 138, 44, 86, 8),
            "fixture population changed: re-measure these counts"
        );
    }

    /// Every `.class` under `tests/fixtures`, in a deterministic order.
    fn class_fixture_paths() -> Vec<std::path::PathBuf> {
        fn walk(directory: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
            let mut entries = std::fs::read_dir(directory)
                .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for entry in entries {
                if entry.is_dir() {
                    walk(&entry, found);
                } else if entry
                    .extension()
                    .is_some_and(|extension| extension == "class")
                {
                    found.push(entry);
                }
            }
        }
        let root = crate::test_fixtures::fixtures_root();
        let mut found = Vec::new();
        walk(&root, &mut found);
        found
    }
}
