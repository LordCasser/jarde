//! Resource consumer: `Manifest` main-section attributes and `META-INF/services`
//! registrations.
//!
//! Facts are structural references, never load or execution claims: a
//! `Main-Class` or agent attribute says the archive names that class, and a
//! `Class-Path` token is a *relative* reference that this scan never resolves.
//!
//! Provenance and coordinates: every item is located with
//! `Location::Resource { entry, span }`, where `span` addresses the decompressed
//! entry content. Class-name attributes use the span of their raw value, service
//! providers the span of their name, and the service registration key the whole
//! resource entry, because that name comes from the entry path rather than from the
//! content. `XrefEvidence.attribute` carries the raw manifest attribute name, or the
//! raw service registration key for `META-INF/services` items.
//!
//! Byte handling: entry names are matched ASCII case-insensitively (as ZIP lookups
//! are) but the raw bytes are preserved. Symbols are built from the raw value bytes
//! with `.` mapped to `/`, which is the internal-name spelling of the same binary
//! name; nothing else is normalized, no text is decoded and matching stays exact.

use super::{ScanContext, ScanUnit, UnitContent, to_u64};
use crate::query::{
    ConsumerKind, LiteralValue, QueryRelation, QueryResolution, QueryTarget, XrefCertainty,
    XrefDerivation, XrefEvidence, XrefItem, XrefOperation, XrefTarget,
};
use jarde_reader::budget::Budget;
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    ArchiveNameBytes, ByteSpan, Diagnostic, DiagnosticSeverity, JvmBytes, SymbolRef,
};

const MANIFEST_PATH: &[u8] = b"META-INF/MANIFEST.MF";
const SERVICES_PREFIX: &[u8] = b"META-INF/services/";
const AGENT_ATTRIBUTES: [&[u8]; 3] = [b"Premain-Class", b"Agent-Class", b"Launcher-Agent-Class"];

pub(super) fn scan(
    ctx: &mut ScanContext<'_>,
    unit: &ScanUnit,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    if !ctx.wants(ConsumerKind::Resource) {
        return Ok(());
    }
    // The raw constant-pool probe is strictly about constant-pool entries, so no
    // resource fact can answer it.
    if ctx.request().relation == QueryRelation::ConstantPoolContains {
        return Ok(());
    }
    let Some(entry) = unit.entry() else {
        // A standalone CLASS carries no Manifest and no service registration.
        return Ok(());
    };
    let name = entry.id.raw_name.0.as_slice();
    let service = if ascii_eq(name, MANIFEST_PATH) {
        None
    } else {
        match service_key(name) {
            Some(key) => Some(key),
            None => return Ok(()),
        }
    };
    let content = ctx.read_unit(unit)?;
    match service {
        None => scan_manifest(ctx, unit, &content, out),
        Some(key) => scan_service(ctx, unit, key, &content, out),
    }
}

fn scan_manifest(
    ctx: &mut ScanContext<'_>,
    unit: &ScanUnit,
    content: &UnitContent,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    let (attributes, malformed) = {
        let budget = ctx.budget();
        parse_main_section(&content.bytes, budget)?
    };
    for span in malformed {
        if let Some(provenance) = unit.resource_provenance(span) {
            ctx.push_diagnostic(Diagnostic {
                code: "query_manifest_line_malformed".into(),
                severity: DiagnosticSeverity::Warning,
                message: "manifest main section contains a line that is not a name/value pair"
                    .into(),
                provenance: Some(provenance),
            });
        }
    }
    for attribute in attributes {
        let Some((operation, reference)) = reference_of(&attribute.name) else {
            continue;
        };
        match reference {
            ManifestReference::ClassSymbol => {
                if attribute.value.is_empty() {
                    continue;
                }
                emit(
                    ctx,
                    unit,
                    Candidate {
                        target: symbol(&attribute.value),
                        attribute: &attribute.name,
                        span: attribute.value_span,
                        operation,
                    },
                    out,
                );
            }
            ManifestReference::Literal => {
                emit(
                    ctx,
                    unit,
                    Candidate {
                        target: literal(&attribute.value),
                        attribute: &attribute.name,
                        span: attribute.value_span,
                        operation,
                    },
                    out,
                );
            }
            ManifestReference::ClassPath => {
                for (span, token) in references(&attribute.value, &attribute.value_span)? {
                    emit(
                        ctx,
                        unit,
                        Candidate {
                            target: literal(token),
                            attribute: &attribute.name,
                            span,
                            operation,
                        },
                        out,
                    );
                }
            }
        }
    }
    Ok(())
}

fn scan_service(
    ctx: &mut ScanContext<'_>,
    unit: &ScanUnit,
    key: &[u8],
    content: &UnitContent,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    // The registration key is the interface the resource registers: the name comes
    // from the entry path, so the whole content is the located range.
    emit(
        ctx,
        unit,
        Candidate {
            target: symbol(key),
            attribute: key,
            span: ByteSpan::new(0, to_u64(content.bytes.len())?),
            operation: XrefOperation::ServiceProvider,
        },
        out,
    );
    let providers = {
        let budget = ctx.budget();
        service_providers(&content.bytes, budget)?
    };
    for (span, provider) in providers {
        emit(
            ctx,
            unit,
            Candidate {
                target: symbol(&provider),
                attribute: key,
                span,
                operation: XrefOperation::ServiceProvider,
            },
            out,
        );
    }
    Ok(())
}

/// Kind of reference a main-section attribute carries.
enum ManifestReference {
    ClassSymbol,
    Literal,
    ClassPath,
}

fn reference_of(name: &[u8]) -> Option<(XrefOperation, ManifestReference)> {
    if ascii_eq(name, b"Main-Class") {
        Some((
            XrefOperation::ManifestMainClass,
            ManifestReference::ClassSymbol,
        ))
    } else if AGENT_ATTRIBUTES.iter().any(|agent| ascii_eq(name, agent)) {
        Some((XrefOperation::ManifestAgent, ManifestReference::ClassSymbol))
    } else if ascii_eq(name, b"Class-Path") {
        Some((
            XrefOperation::ManifestClassPath,
            ManifestReference::ClassPath,
        ))
    } else if ascii_eq(name, b"Automatic-Module-Name") {
        Some((
            XrefOperation::ManifestAutomaticModuleName,
            ManifestReference::Literal,
        ))
    } else if ascii_eq(name, b"Multi-Release") {
        Some((
            XrefOperation::ManifestMultiRelease,
            ManifestReference::Literal,
        ))
    } else {
        None
    }
}

/// One candidate resource fact, before the request filter is applied.
struct Candidate<'a> {
    target: XrefTarget,
    attribute: &'a [u8],
    span: ByteSpan,
    operation: XrefOperation,
}

/// Publishes a candidate when the request target matches it.
///
/// The filter can only drop non-matching candidates, so it never turns an existing
/// reference into a missing one: every candidate is either published as `Exact` or
/// is not a reference to the requested target at all.
fn emit(ctx: &ScanContext<'_>, unit: &ScanUnit, candidate: Candidate<'_>, out: &mut Vec<XrefItem>) {
    if !target_matches(&ctx.request().target, &candidate.target) {
        return;
    }
    let Some(source) = unit.resource_provenance(candidate.span.clone()) else {
        return;
    };
    out.push(XrefItem {
        relation: ctx.request().relation,
        source,
        target: candidate.target,
        consumer: Some(ConsumerKind::Resource),
        operation: candidate.operation,
        derivation: XrefDerivation::StructuralConsumer,
        certainty: XrefCertainty::Exact,
        resolution: QueryResolution::NotRequested,
        evidence: XrefEvidence {
            constant_pool_index: None,
            bci: None,
            opcode: None,
            attribute: Some(ArchiveNameBytes(candidate.attribute.to_vec())),
            span: Some(candidate.span),
            via: Vec::new(),
        },
    });
}

fn target_matches(request: &QueryTarget, candidate: &XrefTarget) -> bool {
    match (request, candidate) {
        (QueryTarget::Symbol { value: expected }, XrefTarget::Symbol { value: found }) => {
            expected == found
        }
        (QueryTarget::Literal { value: expected }, XrefTarget::Literal { value: found }) => {
            expected == found
        }
        _ => false,
    }
}

fn symbol(binary_name: &[u8]) -> XrefTarget {
    XrefTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(internal_name(binary_name)),
        },
    }
}

fn literal(value: &[u8]) -> XrefTarget {
    XrefTarget::Literal {
        value: LiteralValue::String {
            value: JvmBytes(value.to_vec()),
        },
    }
}

/// Internal-name spelling of a binary class name; nothing else is normalized.
fn internal_name(binary_name: &[u8]) -> Vec<u8> {
    binary_name
        .iter()
        .map(|byte| if *byte == b'.' { b'/' } else { *byte })
        .collect()
}

/// Parsed main-section attribute.
struct ManifestAttribute {
    name: Vec<u8>,
    value: Vec<u8>,
    /// Byte range of the value inside the manifest content. A continued value keeps
    /// one range, so it also covers the line break bytes that separate its lines.
    value_span: ByteSpan,
}

/// Parses the main section of a manifest.
///
/// Line breaks may be CRLF, LF or CR. A line starting with a single space continues
/// the previous value: the leading space is dropped and no separator is inserted, as
/// in `java.util.jar.Manifest`. `name: value` skips exactly one space after the
/// colon and trims nothing else. The first empty line ends the main section, so
/// named sections are never application attributes. Names are matched
/// ASCII case-insensitively by the caller while the raw bytes are preserved.
fn parse_main_section(
    content: &[u8],
    budget: &mut Budget,
) -> Result<(Vec<ManifestAttribute>, Vec<ByteSpan>)> {
    let mut attributes: Vec<ManifestAttribute> = Vec::new();
    let mut malformed = Vec::new();
    let mut offset = 0_usize;
    while let Some((line, span)) = next_line(content, &mut offset)? {
        budget.poll()?;
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix(b" ") {
            match attributes.last_mut() {
                Some(attribute) => {
                    attribute.value.extend_from_slice(rest);
                    attribute.value_span.length = attribute
                        .value_span
                        .length
                        .checked_add(to_u64(rest.len())?)
                        .ok_or_else(span_overflow)?;
                }
                None => malformed.push(span),
            }
            continue;
        }
        match line.iter().position(|byte| *byte == b':') {
            Some(colon) if colon > 0 => {
                let mut value_start = colon + 1;
                if value_start < line.len() && line[value_start] == b' ' {
                    value_start += 1;
                }
                attributes.push(ManifestAttribute {
                    name: line[..colon].to_vec(),
                    value: line[value_start..].to_vec(),
                    value_span: sub_span(&span, value_start, line.len())?,
                });
            }
            _ => malformed.push(span),
        }
    }
    Ok((attributes, malformed))
}

/// Provider class names of one service resource, in file order.
///
/// `java.util.ServiceLoader` syntax: a `#` starts a comment that ends the line,
/// blank lines are ignored, surrounding whitespace is trimmed, and a name that ends
/// with `.` continues on the next line.
fn service_providers(content: &[u8], budget: &mut Budget) -> Result<Vec<(ByteSpan, Vec<u8>)>> {
    let mut providers = Vec::new();
    let mut pending: Option<(ByteSpan, Vec<u8>)> = None;
    let mut offset = 0_usize;
    while let Some((line, span)) = next_line(content, &mut offset)? {
        budget.poll()?;
        let body = match line.iter().position(|byte| *byte == b'#') {
            Some(comment) => &line[..comment],
            None => line,
        };
        let (start, end) = trimmed_bounds(body);
        if start == end {
            continue;
        }
        let (name_span, mut name) = match pending.take() {
            Some((previous, name)) => (cover(&previous, &sub_span(&span, start, end)?), name),
            None => (sub_span(&span, start, end)?, Vec::new()),
        };
        name.extend_from_slice(&body[start..end]);
        if name.ends_with(b".") {
            pending = Some((name_span, name));
            continue;
        }
        providers.push((name_span, name));
    }
    Ok(providers)
}

/// Registration key of a `META-INF/services` entry, matched case-insensitively
/// while the raw service name bytes are preserved.
fn service_key(raw_name: &[u8]) -> Option<&[u8]> {
    if raw_name.len() <= SERVICES_PREFIX.len()
        || !ascii_eq(&raw_name[..SERVICES_PREFIX.len()], SERVICES_PREFIX)
    {
        return None;
    }
    let key = &raw_name[SERVICES_PREFIX.len()..];
    if key.is_empty() || key.ends_with(b"/") {
        return None;
    }
    Some(key)
}

/// Splits a `Class-Path` value into its space/tab separated relative references.
fn references<'a>(value: &'a [u8], value_span: &ByteSpan) -> Result<Vec<(ByteSpan, &'a [u8])>> {
    let mut references = Vec::new();
    let mut start: Option<usize> = None;
    for (index, byte) in value.iter().enumerate() {
        if *byte == b' ' || *byte == b'\t' {
            if let Some(begin) = start.take() {
                references.push((sub_span(value_span, begin, index)?, &value[begin..index]));
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(begin) = start {
        references.push((sub_span(value_span, begin, value.len())?, &value[begin..]));
    }
    Ok(references)
}

/// Next line of `content`, advancing `offset` past its line break.
///
/// CRLF, LF and CR all terminate a line; the returned span excludes the break, so a
/// span addresses exactly the line bytes.
fn next_line<'a>(content: &'a [u8], offset: &mut usize) -> Result<Option<(&'a [u8], ByteSpan)>> {
    if *offset >= content.len() {
        return Ok(None);
    }
    let start = *offset;
    let mut end = start;
    while end < content.len() && content[end] != b'\n' && content[end] != b'\r' {
        end += 1;
    }
    let line = &content[start..end];
    let mut next = end;
    if next < content.len() {
        next += if content[next] == b'\r' && content.get(next + 1) == Some(&b'\n') {
            2
        } else {
            1
        };
    }
    *offset = next;
    Ok(Some((
        line,
        ByteSpan::new(to_u64(start)?, to_u64(end - start)?),
    )))
}

fn ascii_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

fn is_space(byte: u8) -> bool {
    byte <= b' '
}

fn trimmed_bounds(line: &[u8]) -> (usize, usize) {
    let end = line
        .iter()
        .rposition(|byte| !is_space(*byte))
        .map_or(0, |index| index + 1);
    let start = line.iter().position(|byte| !is_space(*byte)).unwrap_or(end);
    (start, end)
}

fn cover(first: &ByteSpan, second: &ByteSpan) -> ByteSpan {
    let start = first.start.min(second.start);
    let end = first
        .start
        .saturating_add(first.length)
        .max(second.start.saturating_add(second.length));
    ByteSpan::new(start, end.saturating_sub(start))
}

fn sub_span(base: &ByteSpan, start: usize, end: usize) -> Result<ByteSpan> {
    let offset = to_u64(start)?;
    let length = to_u64(end.saturating_sub(start))?;
    Ok(ByteSpan::new(
        base.start.checked_add(offset).ok_or_else(span_overflow)?,
        length,
    ))
}

fn span_overflow() -> Error {
    Error::invalid_input("query_span_overflow", "resource span exceeds u64")
}
