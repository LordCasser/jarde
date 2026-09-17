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

fn collect_attributes<'a>(
    bytes: &'a [u8],
    pool: &noak::reader::cpool::ConstantPool<'a>,
    attributes: impl IntoIterator<Item = std::result::Result<Attribute<'a>, noak::error::DecodeError>>,
    budget: &mut Budget,
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
        charge_item(budget)?;
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
