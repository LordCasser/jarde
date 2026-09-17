//! Owned, bounded header inspection backed exclusively by noak 0.7.0.

use crate::budget::{Budget, CountedBudgetDimension};
use crate::error::{Error, Result};
use crate::model::{
    ByteSpan, Diagnostic, DiagnosticSeverity, ExecutionReport, JvmBytes, JvmString,
    TerminationReason,
};
use noak::reader::attributes::{Code, RawInstruction};
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    NotPerformed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeaderStructuralRead {
    Complete,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VersionCapability {
    pub version: ClassfileVersion,
    pub version_rule: VersionRuleStatus,
    /// P0 support inferred from major/minor only; no CP, flags, attribute, or opcode dialect validation.
    pub version_dialect_support: VersionDialectSupport,
    pub dialect_validation_scope: DialectValidationScope,
    pub java8_runtime: Java8RuntimeCompatibility,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HeaderInspection {
    pub header: ClassHeader,
    pub structural_read: HeaderStructuralRead,
    pub version_capability: VersionCapability,
    pub verification: VerificationStatus,
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

/// Reads declaration-level structure and applies the implemented version-only P0 gate.
/// Attribute content, instructions, and JVM verification are never performed here.
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

fn classify_version(major: u16, minor: u16) -> VersionCapability {
    let version_rule = if major < 45 {
        VersionRuleStatus::InvalidMajor
    } else if major >= 56 && minor != 0 && minor != u16::MAX {
        VersionRuleStatus::InvalidModernMinor
    } else {
        VersionRuleStatus::Valid
    };
    let preview = major >= 56 && minor == u16::MAX;
    let version_dialect_support = if major > 71 {
        VersionDialectSupport::FutureRelease
    } else if preview {
        VersionDialectSupport::UnsupportedPreview
    } else if (53..=71).contains(&major) {
        VersionDialectSupport::StructuralProbeOnly
    } else if (45..=52).contains(&major) {
        VersionDialectSupport::Supported
    } else {
        VersionDialectSupport::StructuralProbeOnly
    };
    let java8_runtime = if (45..=51).contains(&major) || (major == 52 && minor == 0) {
        Java8RuntimeCompatibility::Accepted
    } else {
        Java8RuntimeCompatibility::Rejected
    };
    VersionCapability {
        version: ClassfileVersion { major, minor },
        version_rule,
        version_dialect_support,
        dialect_validation_scope: DialectValidationScope::VersionOnly,
        java8_runtime,
    }
}

fn version_diagnostics(capability: &VersionCapability) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    match capability.version_rule {
        VersionRuleStatus::Valid => {}
        VersionRuleStatus::InvalidMajor => diagnostics.push(version_diagnostic(
            "classfile_invalid_major_version",
            DiagnosticSeverity::Error,
            format!(
                "classfile major version {} is below the minimum valid major 45",
                capability.version.major
            ),
        )),
        VersionRuleStatus::InvalidModernMinor => diagnostics.push(version_diagnostic(
            "classfile_invalid_modern_minor_version",
            DiagnosticSeverity::Error,
            format!(
                "classfile version {}.{} requires minor 0 or 65535 for major >= 56",
                capability.version.major, capability.version.minor
            ),
        )),
    }
    match capability.version_dialect_support {
        VersionDialectSupport::Supported => {}
        VersionDialectSupport::StructuralProbeOnly => diagnostics.push(version_diagnostic(
            "classfile_version_structural_probe_only",
            DiagnosticSeverity::Warning,
            format!(
                "classfile version {}.{} is only structurally inspected; dialect validation scope is version_only",
                capability.version.major, capability.version.minor
            ),
        )),
        VersionDialectSupport::UnsupportedPreview => diagnostics.push(version_diagnostic(
            "classfile_preview_unsupported",
            DiagnosticSeverity::Warning,
            format!(
                "preview classfile version {}.{} is not supported by the P0 dialect",
                capability.version.major, capability.version.minor
            ),
        )),
        VersionDialectSupport::FutureRelease => diagnostics.push(version_diagnostic(
            "classfile_future_release",
            DiagnosticSeverity::Warning,
            format!(
                "classfile version {}.{} is newer than the P0 structural range ending at major 71",
                capability.version.major, capability.version.minor
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
                "strict header inspection requires P0 version-dialect support",
            ));
        }
        VersionDialectSupport::UnsupportedPreview => {
            return Err(Error::unsupported(
                "classfile_preview_unsupported",
                "strict header inspection does not support preview classfiles",
            ));
        }
        VersionDialectSupport::FutureRelease => {
            return Err(Error::unsupported(
                "classfile_future_release",
                "strict header inspection does not support future classfile releases",
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
            let padding = (4 - ((bci + 1) & 3)) & 3;
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
            let padding = (4 - ((bci + 1) & 3)) & 3;
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

fn charge_item(budget: &mut Budget) -> Result<()> {
    budget.charge(CountedBudgetDimension::ResultItems, 1)
}

fn to_u64(value: usize) -> Result<u64> {
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

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
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
pub(crate) struct CpIndexOf(pub u16);

/// Payload of one constant-pool entry.
///
/// Field names follow the JVMS constant-pool layouts. `owner`, `name`,
/// `descriptor` and every other resolved value are the raw Modified UTF-8 bytes
/// held by the class file: they are never text-decoded and never normalised, so
/// a `.` is not rewritten into `/` and an array owner differs from its element
/// owner.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CpEntryKind {
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
pub(crate) struct CpEntryFacts {
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
pub(crate) struct ClassFacts {
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
pub(crate) struct InnerClassFacts {
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
pub(crate) struct EnclosingMethodFacts {
    /// Index of the immediately enclosing class; resolve it with `cp_class_name`.
    pub class_index: u16,
    /// Index of the enclosing `NameAndType`, or 0 when the class is not
    /// immediately enclosed by a method or constructor.
    pub method_index: u16,
}

/// One `Module` `provides` entry.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProvidesFacts {
    /// Service interface internal name, expanded from `CONSTANT_Class`.
    pub service: JvmBytes,
    /// Provider internal names, in declaration order.
    pub implementations: Vec<JvmBytes>,
}

/// `Module` uses/provides facts. Requires, exports and opens are skipped as byte
/// ranges because no P1 consumer reads them.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModuleFacts {
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
pub(crate) struct AttributeFacts {
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
pub(crate) struct BootstrapMethodFacts {
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
pub(crate) fn class_facts(bytes: &[u8], budget: &mut Budget) -> Result<ClassFacts> {
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
pub(crate) fn attribute_content<'a>(
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
pub(crate) fn attribute_slice<'a>(
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
fn attribute_shell_length(content_span: &ByteSpan) -> Result<u64> {
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
pub(crate) fn attribute_facts(
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
pub(crate) fn bootstrap_methods(
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
pub(crate) enum DescriptorKind {
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
pub(crate) struct EntryDescriptor {
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
pub(crate) fn entry_descriptor(kind: &CpEntryKind) -> Option<EntryDescriptor> {
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
pub(crate) fn descriptor_types(descriptor: &[u8], kind: DescriptorKind) -> Result<Vec<JvmBytes>> {
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
pub(crate) fn push_unique(types: &mut Vec<JvmBytes>, name: Vec<u8>) {
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

/// One nested `attribute_info` inside a `Code` attribute.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NestedAttributeFact {
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
pub(crate) fn code_nested_attributes(
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
fn entry_offset(content_span: &ByteSpan, position: usize) -> Result<u64> {
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
pub(crate) struct MethodCodeFacts {
    pub max_stack: u16,
    pub max_locals: u16,
    /// Class-file range of the instruction array: `code_length` bytes starting at the
    /// first opcode, so a BCI becomes a class offset by adding `code_span.start`.
    pub code_span: ByteSpan,
    pub instructions: Vec<InstructionFact>,
    /// The whole exception table. Handlers are decoded before instructions, exactly
    /// like `inspect_method_bytecode`, so these are complete even when the instruction
    /// stream is the phase that stopped.
    pub exception_handlers: Vec<ExceptionHandlerFact>,
    pub execution: ExecutionReport,
    pub stopped_at: Option<BytecodeStop>,
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
pub(crate) fn method_code_facts(
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
    let mut cursor_bci = 0u32;
    for item in code.raw_instructions() {
        if let Err(error) = budget.poll() {
            return stopped_code_facts(
                &code,
                code_span,
                instructions,
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

    Ok(MethodCodeFacts {
        max_stack: code.max_stack(),
        max_locals: code.max_locals(),
        code_span,
        instructions,
        exception_handlers: handlers,
        execution: ExecutionReport::Complete {
            usage: budget.usage(),
        },
        stopped_at: None,
    })
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
fn stopped_code_facts(
    code: &Code<'_>,
    code_span: ByteSpan,
    instructions: Vec<InstructionFact>,
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
        exception_handlers: handlers,
        execution,
        stopped_at: Some(stopped_at),
    })
}

/// Resolves one 1-based constant-pool index inside a fact table.
#[allow(dead_code)]
pub(crate) fn cp_entry(pool: &[CpEntryFacts], index: u16) -> Result<&CpEntryFacts> {
    if index == 0 {
        return Err(cp_index_error(index));
    }
    pool.binary_search_by_key(&index, |entry| entry.index)
        .map(|position| &pool[position])
        .map_err(|_| cp_index_error(index))
}

/// Resolves a `CONSTANT_Utf8` index to its raw Modified UTF-8 bytes.
#[allow(dead_code)]
pub(crate) fn cp_utf8(pool: &[CpEntryFacts], index: u16) -> Result<JvmBytes> {
    match &cp_entry(pool, index)?.kind {
        CpEntryKind::Utf8 { bytes } => Ok(bytes.clone()),
        _ => Err(cp_fact_tag_mismatch("CONSTANT_Utf8", index)),
    }
}

/// Resolves a `CONSTANT_Class` index to its internal-name bytes.
#[allow(dead_code)]
pub(crate) fn cp_class_name(pool: &[CpEntryFacts], index: u16) -> Result<JvmBytes> {
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
struct AttributeReader<'a> {
    content: &'a [u8],
    position: usize,
}

impl<'a> AttributeReader<'a> {
    const fn new(content: &'a [u8]) -> Self {
        Self {
            content,
            position: 0,
        }
    }

    fn u16(&mut self) -> Result<u16> {
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

    fn u32(&mut self) -> Result<u32> {
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

    fn skip(&mut self, length: usize) -> Result<()> {
        let end = offset_plus(self.position, length)?;
        if end > self.content.len() {
            return Err(attribute_eoi(self.position));
        }
        self.position = end;
        Ok(())
    }

    fn expect_end(&self) -> Result<()> {
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

fn ensure_unique(seen: &mut Vec<&'static str>, name: &'static str) -> Result<()> {
    if seen.contains(&name) {
        return Err(Error::invalid_input(
            "classfile_duplicate_attribute",
            format!("class or member declares more than one {name} attribute"),
        ));
    }
    seen.push(name);
    Ok(())
}

fn read_class_name_list(
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
        include_bytes!("../tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::{BudgetDimension, CancellationToken, Limits};
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
                assert_eq!(*usage, budget.usage());
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
                assert_eq!(*usage, budget.usage());
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
}
