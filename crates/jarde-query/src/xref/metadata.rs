//! Metadata consumer: hierarchy, descriptors, generic signatures, annotations,
//! inner/nest relations, `Exceptions`, `ConstantValue` and module uses/provides.
//!
//! Scope and soundness, in one place because the code below is dense:
//!
//! * Only the categories this request names are read, and only for the relations
//!   they can answer. The reader fact layer enumerates attribute shells for the
//!   whole class (and bills that pass), but attribute *content* is read only for a
//!   requested category, so an unrequested category's content adds no item and is not
//!   parsed. That is not a promise that a request cannot fail: a structure the scan does
//!   read has to match its JVMS grammar, and attribute declarations that contradict the
//!   bytes they cover stop the scan whatever else was requested, because the reader facts
//!   own the class-level structure of every unit they are called for. What an unrequested
//!   category cannot do is contribute a fact of its own. A category whose facts are always
//!   symbol references is not scanned for a `literal_value` request at all (and
//!   `ConstantValue` is a value, so it is not scanned for a symbol request), which is a
//!   provable no-op for the result.
//! * Content this consumer parses itself (member descriptors, generic signatures,
//!   annotations, `Record` components, `ConstantValue`) is parsed exactly; content
//!   that does not match its JVMS grammar is a structured error rather than a
//!   partial fact set, so an incomplete set can never be reported as complete.
//!   Unknown attributes are never read and never influence a read category.
//! * A nested `attribute_info` is consumed through its own declared length
//!   ([`Reader::attribute_body`]): the structure this consumer understands is parsed,
//!   and only the bytes the declaration still covers are skipped. No parser states a
//!   magic content length, because an attribute declares its own size (JVMS 4.7) and a
//!   fixed expectation silently rejects legal input.
//! * A unit is scanned as a class only through the shared archive candidate rule: an
//!   archive entry needs a case-sensitive `.class` raw name (the bare `.class` name
//!   included) before it is materialized at all, and a candidate whose bytes do not carry
//!   the class magic is a located parse failure the scan must report, not a silent
//!   no-fact result. Both the rule and the failure are owned by [`super::class_content`].
//! * `this_class` produces no item: a class is its own definition identity, not a
//!   user of itself (architecture §9.5). Hierarchy comes from `super_class` and
//!   `interfaces` only.
//! * Generic signatures and annotation element values are parsed with their own
//!   grammar; no arbitrary string is substring-matched for a type name. The signature
//!   grammar is the one JVMS 4.7.9.1 states and not the descriptor grammar: a method
//!   signature is `[TypeParameters] ( {JavaTypeSignature} ) Result {ThrowsSignature}`, so
//!   its result may be any java type — a type variable or a parameterized class included —
//!   and the class types of its `throws` list are references of this category. A
//!   descriptor, by contrast, is read with the descriptor productions, and each grammar
//!   rejects what it cannot parse exactly.
//! * Annotation completeness follows the standard positions (decision 25): besides the
//!   class, its fields and its methods, a type annotation inside a method's `Code`
//!   attribute and an annotation or type annotation inside a `Record` component are read.
//!   Both are nested positions, both are read for the annotation category alone (never
//!   through a `Signature` switch), and both stop at the attribute content: no
//!   instruction is decoded, no debug table is read, and nothing here builds a CFG, SSA
//!   or Java AST.
//! * A fact of a nested position is published only at that position's own range: its facts
//!   are held while the structure is read and placed when the structure ends, so a
//!   structure that does not parse publishes nothing of itself — never a fact carrying the
//!   flat coordinates it happened to be read through, which would name a location this
//!   class does not have for it. Nothing unpublished is ever billed.
//! * Nothing here charges `ResultItems`: `super` bills every item it publishes from here.
//!
//! # Order
//!
//! The order is fixed, because the page cursor replays a prefix of it: hierarchy and
//! member descriptors, then the class/field/method `Signature` attributes, then the
//! `Record` walk (component by component: the component's descriptor, and then its nested
//! attributes in the order the component declares them, so a component `Signature` and a
//! component annotation each produce their items where they are declared), then the flat
//! class/field/method annotations, then the type annotations inside method bodies (methods
//! in declaration order, nested attributes in declaration order), then `Exceptions`,
//! `ConstantValue`, inner/nest relations and the module facts.
//!
//! Evidence. `XrefEvidence.attribute` is the raw attribute name that recorded the
//! fact, or `None` for facts the class header records (super class, interfaces,
//! member descriptors). `constant_pool_index` and `span` address the constant-pool
//! entry the fact was read through when this scan read that index itself (annotation
//! elements, `ConstantValue`, `Record` components, `InnerClasses`,
//! `EnclosingMethod`); otherwise they address the one pool entry that carries the
//! same bytes, and when no entry is unique the span falls back to the attribute
//! content that recorded the fact, or to the whole class for header facts.
//! `Provenance` is `Location::ClassOffset { definition, offset }` with
//! `offset = span.start` for the flat class, field and method positions. A fact read at
//! a nested position keeps `Location::Attribute { definition, path, span }` instead:
//! `path` is `Record.components[{i}].{attribute}` or `methods[{i}].Code.{attribute}`, and
//! `span` is the range of the annotation structure itself (`target_type` included for a
//! type annotation), so the bytes can be sliced out of the class file. `evidence.span` is
//! that same range, and `evidence.bci` is set only when a type annotation's `target_info`
//! names exactly one BCI.

use super::{ScanContext, ScanUnit, class_content, to_u64};
use crate::query::{
    ConsumerKind, LiteralValue, QueryRelation, QueryResolution, XrefCertainty, XrefDerivation,
    XrefEvidence, XrefItem, XrefOperation, XrefTarget,
};
use jarde_reader::classfile::{
    AttributeFacts, AttributeShell, ClassFacts, CpEntryFacts, CpEntryKind, DescriptorKind,
    attribute_content, attribute_facts, attribute_slice, class_facts, code_nested_attributes,
    cp_class_name, cp_entry, cp_utf8, descriptor_types,
};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    ArchiveNameBytes, ByteSpan, ClassBytesId, JvmBytes, Location, PhysicalDefinitionId, Provenance,
    SymbolRef,
};
use jarde_reader::signature::{
    parse_class_signature, parse_field_signature, parse_method_signature,
};

const MODULE_INFO_CLASS: &[u8] = b"module-info";

const SIGNATURE_CODE: &str = "query_signature_malformed";
const ANNOTATION_CODE: &str = "query_annotation_malformed";
const RECORD_CODE: &str = "query_record_malformed";
const CONSTANT_VALUE_CODE: &str = "query_constant_value_malformed";
const ENCLOSING_METHOD_CODE: &str = "query_enclosing_method_malformed";

pub(super) fn scan(
    ctx: &mut ScanContext<'_>,
    unit: &ScanUnit,
    out: &mut Vec<XrefItem>,
) -> Result<()> {
    // The raw constant-pool probe asks for pool entries, and every fact below is a
    // structure fact, so this consumer never answers that relation: the X0 producer
    // owns it.
    if ctx.request().relation == QueryRelation::ConstantPoolContains {
        return Ok(());
    }
    let Some(wanted) = Wanted::for_request(ctx) else {
        return Ok(());
    };
    // The shared rule, the read and the damaged-candidate failure are `super`'s: an
    // archive entry is a class only by name, a standalone CLASS root is one by
    // construction, and this stream reads bytes only once it needs them.
    let Some(content) = class_content(ctx, unit)? else {
        return Ok(());
    };
    let length = to_u64(content.bytes.len())?;
    let definition = unit.definition(ClassBytesId {
        digest: content.digest.clone(),
        length,
    });
    let facts = {
        let budget = ctx.budget();
        class_facts(&content.bytes, budget)?
    };
    let units = UnitFacts {
        bytes: &content.bytes,
        facts,
        definition,
        length,
    };
    let mut scan = Scan {
        ctx,
        units: &units,
        out,
        wanted,
        nested: None,
        pending: Vec::new(),
    };
    if wanted.ty {
        scan_hierarchy(&mut scan)?;
        scan_descriptors(&mut scan)?;
    }
    if wanted.signature {
        scan_signatures(&mut scan)?;
    }
    if wanted.signature || wanted.annotation {
        scan_record(&mut scan)?;
    }
    if wanted.annotation {
        scan_annotations(&mut scan)?;
        scan_code_annotations(&mut scan)?;
    }
    if wanted.exception {
        scan_exceptions(&mut scan)?;
    }
    if wanted.constant {
        scan_constants(&mut scan)?;
    }
    if wanted.inner_nest {
        scan_inner_nest(&mut scan)?;
    }
    if wanted.module {
        scan_module(&mut scan)?;
    }
    Ok(())
}

/// Categories this request actually needs scanned.
///
/// The relation decides which candidate shapes can match: symbol references can only
/// answer `mentions_symbol`, values only `literal_value`. Skipping the others is
/// therefore provably result-preserving, and it keeps the budget honest: a
/// `literal_value` request over a `Type`-only schema reads no class at all.
#[derive(Clone, Copy)]
struct Wanted {
    ty: bool,
    signature: bool,
    annotation: bool,
    exception: bool,
    constant: bool,
    inner_nest: bool,
    module: bool,
}

impl Wanted {
    fn for_request(ctx: &ScanContext<'_>) -> Option<Self> {
        let symbol = ctx.request().relation == QueryRelation::MentionsSymbol;
        let literal = ctx.request().relation == QueryRelation::LiteralValue;
        let wanted = Self {
            ty: symbol && ctx.wants(ConsumerKind::Type),
            signature: symbol && ctx.wants(ConsumerKind::Signature),
            exception: symbol && ctx.wants(ConsumerKind::Exception),
            inner_nest: symbol && ctx.wants(ConsumerKind::InnerNest),
            module: symbol && ctx.wants(ConsumerKind::Module),
            // Annotations record both annotation types (symbols) and their element
            // values (literals).
            annotation: (symbol || literal) && ctx.wants(ConsumerKind::Annotation),
            // `ConstantValue` is a value reference and never a class reference.
            constant: literal && ctx.wants(ConsumerKind::Constant),
        };
        let any = wanted.ty
            || wanted.signature
            || wanted.annotation
            || wanted.exception
            || wanted.constant
            || wanted.inner_nest
            || wanted.module;
        any.then_some(wanted)
    }
}

/// Reader facts and physical identity shared by one unit's metadata categories.
struct UnitFacts<'a> {
    bytes: &'a [u8],
    facts: ClassFacts,
    definition: PhysicalDefinitionId,
    length: u64,
}

impl UnitFacts<'_> {
    fn pool(&self) -> &[CpEntryFacts] {
        &self.facts.constant_pool
    }

    /// Class-file range of the whole class, used when a fact the reader layer
    /// resolved cannot be tied to a narrower range.
    fn whole_class(&self) -> ByteSpan {
        ByteSpan::new(0, self.length)
    }
}

/// Which consumer category, operation and attribute one fact belongs to.
#[derive(Clone, Copy)]
struct Site<'a> {
    consumer: ConsumerKind,
    operation: XrefOperation,
    /// Raw attribute name that recorded the fact; `None` for header facts.
    attribute: Option<&'a [u8]>,
}

/// One metadata fact before the request target filter is applied.
struct Candidate<'a> {
    site: Site<'a>,
    target: XrefTarget,
    constant_pool_index: Option<u16>,
    span: Option<ByteSpan>,
}

/// One unit's metadata scan: the shared reader facts, the scan context that owns the
/// request and the budget, the item sink, and the nested position being parsed.
struct Scan<'a, 'c, 's> {
    ctx: &'a mut ScanContext<'s>,
    units: &'a UnitFacts<'c>,
    out: &'a mut Vec<XrefItem>,
    /// Categories this request needs, so a position read for two categories knows which
    /// products to emit from the one pass over its bytes.
    wanted: Wanted,
    /// The nested attribute position the parse is currently inside, if any.
    nested: Option<Nested>,
    /// Facts read inside the open nested structure, waiting for its range.
    ///
    /// A fact can only be published at a position, and a nested structure's own range is
    /// only known once it has been read to its end, so its facts wait here
    /// ([`Scan::place`]). A structure that fails leaves them here instead of publishing
    /// them with a position they do not have ([`Scan::discard_pending`]).
    pending: Vec<XrefItem>,
}

/// The nested attribute position facts are being read at.
///
/// A fact read at a flat class, field or method position keeps
/// `Location::ClassOffset`; a fact read inside a nested attribute belongs to a structure
/// that lives at a path inside another attribute, and states that path and its own range
/// through [`Scan::place`]. This is parse state rather than part of the position because
/// the range of a structure is only known once it has been read to its end.
struct Nested {
    /// `Location::Attribute.path` prefix, without the attribute name.
    prefix: String,
    /// Raw name of the nested attribute the structure was read from.
    attribute: Vec<u8>,
}

impl Scan<'_, '_, '_> {
    fn pool(&self) -> &[CpEntryFacts] {
        self.units.pool()
    }

    /// Constant-pool index and span of a fact this scan resolves through `index`.
    fn indexed(&self, index: u16) -> Result<(Option<u16>, Option<ByteSpan>)> {
        Ok((
            Some(index),
            Some(cp_entry(self.pool(), index)?.span.clone()),
        ))
    }

    /// Constant-pool index and span of a class name the reader facts resolved
    /// without exposing the index that declared it.
    fn resolved_class(
        &self,
        name: &[u8],
        fallback: Option<ByteSpan>,
    ) -> (Option<u16>, Option<ByteSpan>) {
        match unique_entry(self.pool(), |entry| match &entry.kind {
            CpEntryKind::Class {
                name: entry_name, ..
            } => entry_name.0.as_slice() == name,
            _ => false,
        }) {
            Some(entry) => (Some(entry.index), Some(entry.span.clone())),
            None => (None, fallback),
        }
    }

    /// Constant-pool index and span of a `CONSTANT_Utf8` value (a descriptor or a
    /// generic signature) the reader facts resolved without exposing its index.
    fn resolved_utf8(
        &self,
        bytes: &[u8],
        fallback: Option<ByteSpan>,
    ) -> (Option<u16>, Option<ByteSpan>) {
        match unique_entry(self.pool(), |entry| match &entry.kind {
            CpEntryKind::Utf8 { bytes: entry_bytes } => entry_bytes.0.as_slice() == bytes,
            _ => false,
        }) {
            Some(entry) => (Some(entry.index), Some(entry.span.clone())),
            None => (None, fallback),
        }
    }

    fn class(&mut self, site: Site<'_>, name: &[u8], index: Option<u16>, span: Option<ByteSpan>) {
        self.emit(Candidate {
            site,
            target: XrefTarget::Symbol {
                value: SymbolRef::Class {
                    owner: JvmBytes(name.to_vec()),
                },
            },
            constant_pool_index: index,
            span,
        });
    }

    fn method(
        &mut self,
        site: Site<'_>,
        owner: &[u8],
        name: &[u8],
        descriptor: &[u8],
        index: Option<u16>,
        span: Option<ByteSpan>,
    ) {
        self.emit(Candidate {
            site,
            target: XrefTarget::Symbol {
                value: SymbolRef::Method {
                    owner: JvmBytes(owner.to_vec()),
                    name: JvmBytes(name.to_vec()),
                    descriptor: JvmBytes(descriptor.to_vec()),
                },
            },
            constant_pool_index: index,
            span,
        });
    }

    fn literal(
        &mut self,
        site: Site<'_>,
        value: LiteralValue,
        index: Option<u16>,
        span: Option<ByteSpan>,
    ) {
        self.emit(Candidate {
            site,
            target: XrefTarget::Literal { value },
            constant_pool_index: index,
            span,
        });
    }

    /// Publishes a candidate when the active filter answers it, and never otherwise.
    ///
    /// The filter only drops candidates that do not answer the scan, so it cannot turn a
    /// recorded reference into a missing one. Which candidates answer it is the scan
    /// context's decision ([`super::ScanContext::candidate_matches`]), and the published
    /// target is the one that decision returns: under a query's exact target that is the
    /// target the caller asked for, and under a member shape it is the symbol this fact
    /// really names, owner included. Every candidate is `Exact`: the fact was read from
    /// bytes that parsed exactly.
    ///
    /// A fact read while a nested position is open is held in [`Scan::pending`] instead of
    /// being published: its position is the structure it was read from, and that range is
    /// only known once the structure has been read to its end ([`Scan::place`]).
    fn emit(&mut self, candidate: Candidate<'_>) {
        let (symbol, literal) = match &candidate.target {
            XrefTarget::Symbol { value } => (Some(value), None),
            XrefTarget::Literal { value } => (None, Some(value)),
        };
        let Some(target) = self.ctx.published_target(symbol, literal) else {
            return;
        };
        let relation = self.ctx.request().relation;
        let offset = candidate.span.as_ref().map_or(0, |span| span.start);
        let definition = self.units.definition.clone();
        let source = Provenance {
            location: Location::ClassOffset { definition, offset },
        };
        let item = XrefItem {
            relation,
            source,
            target,
            consumer: Some(candidate.site.consumer),
            operation: candidate.site.operation,
            derivation: XrefDerivation::StructuralConsumer,
            certainty: XrefCertainty::Exact,
            resolution: QueryResolution::NotRequested,
            evidence: XrefEvidence {
                constant_pool_index: candidate.constant_pool_index,
                bci: None,
                opcode: None,
                attribute: candidate
                    .site
                    .attribute
                    .map(|name| ArchiveNameBytes(name.to_vec())),
                span: candidate.span,
                via: Vec::new(),
            },
        };
        if self.nested.is_some() {
            self.pending.push(item);
        } else {
            self.out.push(item);
        }
    }

    /// Places the facts one nested structure produced at that structure's own position.
    ///
    /// A nested structure's class-file range is only known once it has been read to its
    /// end, while its facts are found as it is read. The parse therefore states the
    /// position it is inside ([`Scan::nested`]), holds the facts of one structure in
    /// [`Scan::pending`], and places them here once the structure ends: every fact of it
    /// keeps that structure's span in both `evidence.span` and `Location::Attribute.span`,
    /// the attribute path that reached it, the raw nested attribute name, and the BCI its
    /// `target_info` names. Only then are they published to `out`, in the order they were
    /// read, which is the order of the structures themselves.
    fn place(&mut self, span: ByteSpan, bci: Option<u32>) -> Result<()> {
        let Some(nested) = &self.nested else {
            return Err(Error::invalid_input(
                ANNOTATION_CODE,
                "no nested attribute position is open for these facts",
            ));
        };
        let definition = self.units.definition.clone();
        let attribute = ArchiveNameBytes(nested.attribute.clone());
        let path = format!("{}.{}", nested.prefix, escaped_name(&nested.attribute));
        for item in &mut self.pending {
            item.source.location = Location::Attribute {
                owner: definition.clone(),
                path: path.clone(),
                span: span.clone(),
            };
            item.evidence.attribute = Some(attribute.clone());
            item.evidence.span = Some(span.clone());
            item.evidence.bci = bci;
        }
        self.out.append(&mut self.pending);
        Ok(())
    }

    /// Drops the facts of a nested structure that could not be read to its end.
    ///
    /// A fact of a nested structure is only a fact at that structure's position, so when
    /// the structure fails there is no position to publish it at: the facts are dropped
    /// instead of being published with the flat coordinates they were read through, which
    /// would name a location this class does not have for them. Nothing dropped here is
    /// ever published, so no `ResultItems` is charged for it either.
    fn discard_pending(&mut self) {
        self.pending.clear();
    }
}

/// Renders one raw attribute name for an attribute path.
///
/// A name is raw bytes, so a byte outside printable ASCII is spelled `\xNN` instead of
/// being replaced; every standard attribute name is unaffected.
fn escaped_name(raw: &[u8]) -> String {
    raw.escape_ascii().to_string()
}

/// The one entry the predicate accepts, or `None` when zero or several do.
///
/// The reader fact layer resolves names, descriptors and signatures without the
/// index the declaring structure used, so an entry can only be identified by its
/// bytes. Two entries with the same bytes make that identification ambiguous, and
/// then no entry (never a guessed one) is reported as evidence.
fn unique_entry(
    pool: &[CpEntryFacts],
    accept: impl Fn(&CpEntryFacts) -> bool,
) -> Option<&CpEntryFacts> {
    let mut found: Option<&CpEntryFacts> = None;
    for entry in pool {
        if !accept(entry) {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(entry);
    }
    found
}

fn type_site(operation: XrefOperation) -> Site<'static> {
    Site {
        consumer: ConsumerKind::Type,
        operation,
        attribute: None,
    }
}

// ---------------------------------------------------------------------------
// Hierarchy and descriptors
// ---------------------------------------------------------------------------

/// `super_class` and `interfaces`.
///
/// `this_class` is the unit's definition identity, so the super class and the
/// interface list are the whole hierarchy this consumer records.
fn scan_hierarchy(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    let fallback = Some(scan.units.whole_class());
    if let Some(super_class) = &scan.units.facts.super_class {
        let name = super_class.raw();
        let (index, span) = scan.resolved_class(&name.0, fallback.clone());
        scan.class(type_site(XrefOperation::SuperClass), &name.0, index, span);
    }
    let interfaces: Vec<Vec<u8>> = scan
        .units
        .facts
        .interfaces
        .iter()
        .map(|interface| interface.raw().0.clone())
        .collect();
    for name in &interfaces {
        let (index, span) = scan.resolved_class(name, fallback.clone());
        scan.class(type_site(XrefOperation::Interface), name, index, span);
    }
    Ok(())
}

/// Field and method descriptors: one item per object type of each descriptor.
fn scan_descriptors(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    let fields: Vec<Vec<u8>> = scan
        .units
        .facts
        .fields
        .iter()
        .map(|field| field.descriptor.raw().0.clone())
        .collect();
    for descriptor in &fields {
        let site = type_site(XrefOperation::FieldDescriptor);
        emit_member_descriptor(scan, site, descriptor, DescriptorKind::Field)?;
    }
    let methods: Vec<Vec<u8>> = scan
        .units
        .facts
        .methods
        .iter()
        .map(|method| method.descriptor.raw().0.clone())
        .collect();
    for descriptor in &methods {
        let site = type_site(XrefOperation::MethodDescriptor);
        emit_member_descriptor(scan, site, descriptor, DescriptorKind::Method)?;
    }
    Ok(())
}

/// Emits the object types of a member descriptor.
///
/// The reader facts resolve a member descriptor without the constant-pool index
/// that declared it, so the index and span come from the one `CONSTANT_Utf8` entry
/// that carries this descriptor string, or from the class-wide range when the
/// descriptor string is not unique in the pool.
fn emit_member_descriptor(
    scan: &mut Scan<'_, '_, '_>,
    site: Site<'_>,
    descriptor: &[u8],
    kind: DescriptorKind,
) -> Result<()> {
    let fallback = Some(scan.units.whole_class());
    let (index, span) = scan.resolved_utf8(descriptor, fallback);
    for name in descriptor_types(descriptor, kind)? {
        scan.class(site, &name.0, index, span.clone());
    }
    Ok(())
}

/// Emits the object types of a descriptor this scan read through `index`.
fn emit_indexed_descriptor(
    scan: &mut Scan<'_, '_, '_>,
    site: Site<'_>,
    descriptor: &[u8],
    kind: DescriptorKind,
    index: u16,
) -> Result<()> {
    let (index, span) = scan.indexed(index)?;
    for name in descriptor_types(descriptor, kind)? {
        scan.class(site, &name.0, index, span.clone());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Signature
// ---------------------------------------------------------------------------

/// Class, field and method `Signature` attributes.
///
/// The `Record` attribute is walked separately ([`scan_record`]), because a component's
/// nested annotations are a position of their own and are read for the annotation category
/// even when no signature was asked for.
fn scan_signatures(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    let class_shells = scan.units.facts.attributes.clone();
    scan_signature_attribute(scan, &class_shells, SignatureKind::Class)?;
    let fields: Vec<Vec<AttributeShell>> = scan
        .units
        .facts
        .fields
        .iter()
        .map(|field| field.attributes.clone())
        .collect();
    for shells in &fields {
        scan_signature_attribute(scan, shells, SignatureKind::Field)?;
    }
    let methods: Vec<Vec<AttributeShell>> = scan
        .units
        .facts
        .methods
        .iter()
        .map(|method| method.attributes.clone())
        .collect();
    for shells in &methods {
        scan_signature_attribute(scan, shells, SignatureKind::Method)?;
    }
    Ok(())
}

/// Emits the `Signature` attribute of one declaration, if it has one.
fn scan_signature_attribute(
    scan: &mut Scan<'_, '_, '_>,
    shells: &[AttributeShell],
    kind: SignatureKind,
) -> Result<()> {
    let Some((facts, span)) = read_named_attribute(scan, shells, b"Signature")? else {
        return Ok(());
    };
    let Some(signature) = facts.signature else {
        return Ok(());
    };
    let site = Site {
        consumer: ConsumerKind::Signature,
        operation: XrefOperation::GenericSignature,
        attribute: Some(b"Signature"),
    };
    emit_signature(scan, site, &signature.0, kind, None, Some(span))
}

/// Emits one item per class type a generic signature names.
fn emit_signature(
    scan: &mut Scan<'_, '_, '_>,
    site: Site<'_>,
    signature: &[u8],
    kind: SignatureKind,
    index: Option<u16>,
    fallback: Option<ByteSpan>,
) -> Result<()> {
    let (index, span) = match index {
        Some(index) => scan.indexed(index)?,
        None => scan.resolved_utf8(signature, fallback),
    };
    let names = {
        let budget = scan.ctx.budget();
        let parsed = match kind {
            SignatureKind::Class => parse_class_signature(signature, budget)
                .and_then(|signature| signature.class_references(budget)),
            SignatureKind::Field => parse_field_signature(signature, budget)
                .and_then(|signature| signature.class_references(budget)),
            SignatureKind::Method => parse_method_signature(signature, budget)
                .and_then(|signature| signature.class_references(budget)),
        };
        parsed.map_err(|error| match error {
            Error::InvalidInput { message, .. } => Error::invalid_input(SIGNATURE_CODE, message),
            other => other,
        })?
    };
    for name in names {
        scan.class(site, &name, index, span.clone());
    }
    Ok(())
}

/// Which signature production to accept (JVMS 4.7.9.1).
#[derive(Clone, Copy, Eq, PartialEq)]
enum SignatureKind {
    Class,
    Field,
    Method,
}

/// `Record` attribute: `record_component_info` entries (JVMS 4.7.30).
///
/// A record component is a declared member whose type is recorded here rather than in a
/// field, and its own attributes carry facts of two categories:
///
/// * the component descriptor and a component `Signature` attribute name types, and are
///   read when the signature category is requested;
/// * a component `RuntimeVisible/InvisibleAnnotations` or
///   `RuntimeVisible/InvisibleTypeAnnotations` attribute is an annotation position in its
///   own right (JVMS 4.7.30, decision 25), read when the annotation category is requested
///   and never through the signature switch.
///
/// The component `name_index` is a member name and is not a type reference. A nested
/// attribute of neither kind is skipped as its declared byte range: those bytes are
/// already part of the `Record` content this read billed, so they are not billed twice. An
/// annotation attribute's declaration must cover exactly its content, like a flat
/// annotation attribute; a `Signature` is parsed through its declared length, so a
/// declaration that contradicts the structure contributes no fact.
fn scan_record(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    let Some(shell) = scan
        .units
        .facts
        .attributes
        .iter()
        .find(|shell| shell.name.raw().0.as_slice() == b"Record")
        .cloned()
    else {
        return Ok(());
    };
    let content = {
        let budget = scan.ctx.budget();
        attribute_content(scan.units.bytes, &shell, budget)?
    };
    let mut reader = Reader::located(content, shell.content_span.start, RECORD_CODE);
    let components = reader.u16()?;
    for component in 0..components {
        let _component_name = reader.u16()?;
        let descriptor_index = reader.u16()?;
        if scan.wanted.signature {
            let descriptor = cp_utf8(scan.pool(), descriptor_index)?;
            let site = Site {
                consumer: ConsumerKind::Signature,
                operation: XrefOperation::RecordComponent,
                attribute: Some(b"Record"),
            };
            emit_indexed_descriptor(
                scan,
                site,
                &descriptor.0,
                DescriptorKind::Field,
                descriptor_index,
            )?;
        }
        let attributes = reader.u16()?;
        let prefix = format!("Record.components[{component}]");
        for _ in 0..attributes {
            let name_index = reader.u16()?;
            let declared = u64::from(reader.u32()?);
            let declared_bytes = usize::try_from(declared)
                .map_err(|_| reader.malformed("attribute length does not fit a byte range"))?;
            let name = cp_utf8(scan.pool(), name_index)?;
            let name_bytes = name.0.as_slice();
            if name_bytes == b"Signature" {
                if !scan.wanted.signature {
                    reader.skip(declared_bytes)?;
                    continue;
                }
                // A component `Signature` is one constant-pool index (JVMS 4.7.9), and
                // whatever else the declaration covers is skipped. The item is emitted
                // only after the declared length and the structure agree, so an attribute
                // whose declaration contradicts its content contributes no fact.
                let signature = reader.attribute_body(declared, |reader| {
                    let signature_index = reader.u16()?;
                    let signature = cp_utf8(scan.pool(), signature_index)?;
                    Ok((signature_index, signature))
                })?;
                let site = Site {
                    consumer: ConsumerKind::Signature,
                    operation: XrefOperation::RecordComponent,
                    attribute: Some(b"Signature"),
                };
                emit_signature(
                    scan,
                    site,
                    &signature.1.0,
                    SignatureKind::Field,
                    Some(signature.0),
                    None,
                )?;
                continue;
            }
            let annotation = annotation_attribute(name_bytes, Scope::RecordComponent);
            if !scan.wanted.annotation || annotation.is_none() {
                reader.skip(declared_bytes)?;
                continue;
            }
            let (operation, kind) = annotation.expect("checked above");
            let start = reader.here()?;
            let bytes = reader.take(declared_bytes)?;
            let site = Site {
                consumer: ConsumerKind::Annotation,
                operation,
                attribute: Some(name_bytes),
            };
            let mut nested = Reader::located(bytes, start, ANNOTATION_CODE);
            scan.nested = Some(Nested {
                prefix: prefix.clone(),
                attribute: name.0.clone(),
            });
            let outcome = parse_annotations(&mut nested, scan, site, kind);
            scan.nested = None;
            if let Err(error) = outcome {
                // The structure's facts are dropped with it, so a later position cannot
                // inherit facts that were never placed.
                scan.discard_pending();
                return Err(error);
            }
            nested.expect_end()?;
        }
    }
    reader.expect_end()
}

// ---------------------------------------------------------------------------
// Annotations
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Eq, PartialEq)]
enum Scope {
    Class,
    Field,
    Method,
    /// A `record_component_info`, whose own nested attributes may hold annotations.
    RecordComponent,
    /// A `Code` attribute, which may hold type annotations only.
    Code,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AnnotationKind {
    /// `RuntimeVisible/InvisibleAnnotations`.
    Annotations,
    /// `RuntimeVisible/InvisibleTypeAnnotations`.
    TypeAnnotations,
    /// `RuntimeVisible/InvisibleParameterAnnotations` (methods only).
    ParameterAnnotations,
    /// `AnnotationDefault` (methods only).
    Default,
}

/// The annotation attribute of this name in this scope, if it is one this consumer
/// reads.
///
/// A scope decides which kinds are possible, and the standard positions differ: a
/// `RuntimeVisible/InvisibleAnnotations` attribute is read on the class, a field, a method
/// and a record component, while a `Code` attribute holds type annotations only
/// (JVMS 4.7.3, 4.7.20, 4.7.30). Parameter annotations and `AnnotationDefault` are method
/// attributes, and a record component or a body declares neither.
fn annotation_attribute(name: &[u8], scope: Scope) -> Option<(XrefOperation, AnnotationKind)> {
    match name {
        b"RuntimeVisibleAnnotations" | b"RuntimeInvisibleAnnotations" if scope != Scope::Code => {
            Some((XrefOperation::Annotation, AnnotationKind::Annotations))
        }
        b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations" => Some((
            XrefOperation::TypeAnnotation,
            AnnotationKind::TypeAnnotations,
        )),
        b"RuntimeVisibleParameterAnnotations" | b"RuntimeInvisibleParameterAnnotations"
            if scope == Scope::Method =>
        {
            Some((
                XrefOperation::Annotation,
                AnnotationKind::ParameterAnnotations,
            ))
        }
        b"AnnotationDefault" if scope == Scope::Method => {
            Some((XrefOperation::AnnotationDefault, AnnotationKind::Default))
        }
        _ => None,
    }
}

/// Annotation attributes on the class, its fields and its methods.
///
/// Annotation type descriptors, class literals, enum types and nested annotation
/// types are emitted as type references; string, primitive, class literal and enum
/// constant values are additionally emitted as literal values (an enum constant is
/// reported by its recorded name, and its enum type by the type reference above).
fn scan_annotations(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    for (scope, shell) in annotation_shells(scan.units) {
        let Some((operation, kind)) = annotation_attribute(shell.name.raw().0.as_slice(), scope)
        else {
            continue;
        };
        let attribute = shell.name.raw().0.clone();
        let content = {
            let budget = scan.ctx.budget();
            attribute_content(scan.units.bytes, &shell, budget)?
        };
        let site = Site {
            consumer: ConsumerKind::Annotation,
            operation,
            attribute: Some(&attribute),
        };
        let mut reader = Reader::new(content, ANNOTATION_CODE);
        parse_annotations(&mut reader, scan, site, kind)?;
        reader.expect_end()?;
    }
    Ok(())
}

/// `RuntimeVisible/InvisibleTypeAnnotations` inside a method's `Code` attribute.
///
/// A type annotation on an instruction, a local variable, a catch clause or a resource is
/// a standard annotation position *inside* a body (JVMS 4.7.20), so a request that asks for
/// annotation completeness reads it even though nothing here decodes an instruction. The
/// body's structure is walked by the reader facts ([`code_nested_attributes`]), which bills
/// the `Code` entry once and decodes nothing; the nested type-annotation content is sliced
/// out of that charge, and only the requested attribute is parsed.
///
/// Methods are walked in declaration order and the nested attributes in declaration order,
/// which is the order the items of one unit come out in.
fn scan_code_annotations(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    let methods: Vec<Vec<AttributeShell>> = scan
        .units
        .facts
        .methods
        .iter()
        .map(|method| method.attributes.clone())
        .collect();
    for (index, shells) in methods.iter().enumerate() {
        let nested = {
            let budget = scan.ctx.budget();
            let bytes = scan.units.bytes;
            let pool = scan.units.pool();
            code_nested_attributes(bytes, shells, pool, budget)?
        };
        let prefix = format!("methods[{index}].Code");
        for fact in &nested {
            let name = fact.name.0.as_slice();
            let Some((operation, AnnotationKind::TypeAnnotations)) =
                annotation_attribute(name, Scope::Code)
            else {
                // Another nested attribute, or a kind a body cannot hold: it is skipped
                // as its declared bytes, and its content is not this consumer's to read.
                continue;
            };
            let content = {
                let bytes = scan.units.bytes;
                attribute_slice(bytes, &fact.span, &fact.content_span)?
            };
            let site = Site {
                consumer: ConsumerKind::Annotation,
                operation,
                attribute: Some(name),
            };
            let mut reader = Reader::located(content, fact.content_span.start, ANNOTATION_CODE);
            scan.nested = Some(Nested {
                prefix: prefix.clone(),
                attribute: fact.name.0.clone(),
            });
            let outcome =
                parse_annotations(&mut reader, scan, site, AnnotationKind::TypeAnnotations);
            scan.nested = None;
            if let Err(error) = outcome {
                // The structure's facts are dropped with it, so a later position cannot
                // inherit facts that were never placed.
                scan.discard_pending();
                return Err(error);
            }
            reader.expect_end()?;
        }
    }
    Ok(())
}

/// Parses the annotation list of one attribute content.
///
/// One parser serves the flat class/field/method positions and the nested `Record` and
/// `Code` positions: the grammar is the same, and what differs is only the position a fact
/// is placed at, which [`Scan::place`] applies when a nested position is open.
fn parse_annotations(
    reader: &mut Reader<'_>,
    scan: &mut Scan<'_, '_, '_>,
    site: Site<'_>,
    kind: AnnotationKind,
) -> Result<()> {
    match kind {
        AnnotationKind::Annotations => {
            for _ in 0..reader.u16()? {
                let start = reader.here()?;
                parse_annotation(reader, scan, site, start, None)?;
            }
        }
        AnnotationKind::TypeAnnotations => {
            for _ in 0..reader.u16()? {
                let start = reader.here()?;
                let bci = type_annotation_prefix(reader)?;
                parse_annotation(reader, scan, site, start, bci)?;
            }
        }
        AnnotationKind::ParameterAnnotations => {
            for _ in 0..reader.u8()? {
                for _ in 0..reader.u16()? {
                    let start = reader.here()?;
                    parse_annotation(reader, scan, site, start, None)?;
                }
            }
        }
        AnnotationKind::Default => element_value(reader, scan, site)?,
    }
    Ok(())
}

/// Parses one annotation structure and places its facts at that structure's own range.
///
/// A structure's class-file range is only known once it has been read to its end, which is
/// why its facts are placed afterwards ([`Scan::place`]); a flat position keeps its class
/// offset and states no BCI, and the target kinds that name a bytecode offset are only
/// legal inside a `Code` attribute, where the nested position records it.
///
/// A structure that fails does not leave a half-read fact behind: its facts are dropped and
/// the error is propagated, so the scan stops with a report that holds no fact of a
/// structure nobody finished reading. That rule is the same for flat and nested positions —
/// a flat fact of an earlier, complete structure stays published, while a nested structure
/// is one position and therefore publishes nothing of itself.
fn parse_annotation(
    reader: &mut Reader<'_>,
    scan: &mut Scan<'_, '_, '_>,
    site: Site<'_>,
    start: u64,
    bci: Option<u32>,
) -> Result<()> {
    let outcome = read_annotation(reader, scan, site, start, bci);
    if outcome.is_err() {
        scan.discard_pending();
    }
    outcome
}

/// Reads one annotation structure and places its facts, without the failure rule.
fn read_annotation(
    reader: &mut Reader<'_>,
    scan: &mut Scan<'_, '_, '_>,
    site: Site<'_>,
    start: u64,
    bci: Option<u32>,
) -> Result<()> {
    annotation(reader, scan, site)?;
    if scan.nested.is_some() {
        let span = reader.span_since(start)?;
        scan.place(span, bci)?;
    }
    Ok(())
}

/// Annotation attributes with the scope they appear in, copied out of the reader
/// facts so the scan state stays mutably borrowable while one attribute is read.
fn annotation_shells(units: &UnitFacts<'_>) -> Vec<(Scope, AttributeShell)> {
    let mut shells = Vec::new();
    let mut collect = |scope: Scope, shells_of: &[AttributeShell]| {
        for shell in shells_of {
            if annotation_attribute(shell.name.raw().0.as_slice(), scope).is_some() {
                shells.push((scope, shell.clone()));
            }
        }
    };
    collect(Scope::Class, &units.facts.attributes);
    for field in &units.facts.fields {
        collect(Scope::Field, &field.attributes);
    }
    for method in &units.facts.methods {
        collect(Scope::Method, &method.attributes);
    }
    shells
}

/// One annotation: its type descriptor and its element values.
fn annotation(reader: &mut Reader<'_>, scan: &mut Scan<'_, '_, '_>, site: Site<'_>) -> Result<()> {
    let type_index = reader.u16()?;
    let descriptor = cp_utf8(scan.pool(), type_index)?;
    emit_indexed_descriptor(scan, site, &descriptor.0, DescriptorKind::Field, type_index)?;
    for _ in 0..reader.u16()? {
        // The element name is an annotation member, not a type reference.
        let _element_name = reader.u16()?;
        element_value(reader, scan, site)?;
    }
    Ok(())
}

/// One type annotation up to its `annotation` structure: `target_info` and `type_path`.
///
/// Returns the BCI the `target_info` names, when it names exactly one.
fn type_annotation_prefix(reader: &mut Reader<'_>) -> Result<Option<u32>> {
    let target_type = reader.u8()?;
    let bci = read_target_info(reader, target_type)?;
    for _ in 0..reader.u8()? {
        reader.skip(2)?;
    }
    Ok(bci)
}

/// Reads the `target_info` of one type annotation (JVMS 4.7.20) and returns the BCI it
/// names, when the target kind names exactly one.
///
/// * `new`, `instanceof`, constructor-reference and method-reference targets
///   (`0x43`..=`0x46`) and type-argument targets (`0x47`..=`0x4B`) hold a bytecode offset.
/// * A `localvar_target` (`0x40`/`0x41`) names one start PC when its table declares exactly
///   one entry; a table with several entries names a range each, so no single BCI is
///   claimed for it.
/// * A `catch_target` (`0x42`) holds an **exception-table index**, not an offset, so it
///   states no BCI: `evidence` has no field for a table index, and reporting an index as a
///   bytecode offset would be a false coordinate.
/// * Type-parameter, supertype, bound, formal-parameter and throws targets record a
///   declaration position rather than a bytecode position.
fn read_target_info(reader: &mut Reader<'_>, target_type: u8) -> Result<Option<u32>> {
    match target_type {
        0x00 | 0x01 | 0x16 => {
            reader.skip(1)?;
            Ok(None)
        }
        0x10 | 0x11 | 0x12 | 0x17 => {
            reader.skip(2)?;
            Ok(None)
        }
        0x13..=0x15 => Ok(None),
        // `localvar_target`: a `table_length` counted table of three u16 fields.
        0x40 | 0x41 => {
            let count = usize::from(reader.u16()?);
            if count != 1 {
                reader.skip(count * 6)?;
                return Ok(None);
            }
            let start_pc = reader.u16()?;
            reader.skip(4)?; // length, index
            Ok(Some(u32::from(start_pc)))
        }
        // `catch_target`: an exception-table index.
        0x42 => {
            reader.skip(2)?;
            Ok(None)
        }
        // `offset` targets: a bytecode offset.
        0x43..=0x46 => Ok(Some(u32::from(reader.u16()?))),
        // `type_argument_target`: `offset` (u16) and `type_argument_index` (u8).
        0x47..=0x4B => {
            let offset = reader.u16()?;
            reader.skip(1)?;
            Ok(Some(u32::from(offset)))
        }
        _ => Err(reader.malformed("type annotation target kind is unknown")),
    }
}

/// One `element_value` (JVMS 4.7.16.1).
fn element_value(
    reader: &mut Reader<'_>,
    scan: &mut Scan<'_, '_, '_>,
    site: Site<'_>,
) -> Result<()> {
    match reader.u8()? {
        b'B' | b'C' | b'S' | b'Z' | b'I' => {
            let index = reader.u16()?;
            let value = match &cp_entry(scan.pool(), index)?.kind {
                CpEntryKind::Integer { value } => *value,
                _ => {
                    return Err(
                        reader.malformed("an int annotation value must reference CONSTANT_Integer")
                    );
                }
            };
            let (index, span) = scan.indexed(index)?;
            scan.literal(site, LiteralValue::Integer { value }, index, span);
        }
        b'J' => {
            let index = reader.u16()?;
            let value = match &cp_entry(scan.pool(), index)?.kind {
                CpEntryKind::Long { value } => *value,
                _ => {
                    return Err(
                        reader.malformed("a long annotation value must reference CONSTANT_Long")
                    );
                }
            };
            let (index, span) = scan.indexed(index)?;
            scan.literal(site, LiteralValue::Long { value }, index, span);
        }
        b'F' => {
            let index = reader.u16()?;
            let value = match &cp_entry(scan.pool(), index)?.kind {
                CpEntryKind::Float { bits } => *bits,
                _ => {
                    return Err(
                        reader.malformed("a float annotation value must reference CONSTANT_Float")
                    );
                }
            };
            let (index, span) = scan.indexed(index)?;
            scan.literal(site, LiteralValue::Float { value }, index, span);
        }
        b'D' => {
            let index = reader.u16()?;
            let value = match &cp_entry(scan.pool(), index)?.kind {
                CpEntryKind::Double { bits } => *bits,
                _ => {
                    return Err(reader
                        .malformed("a double annotation value must reference CONSTANT_Double"));
                }
            };
            let (index, span) = scan.indexed(index)?;
            scan.literal(site, LiteralValue::Double { value }, index, span);
        }
        b's' => {
            // A string value is a `CONSTANT_Utf8` value; a `CONSTANT_String` entry is
            // not required and is not what this reads.
            let index = reader.u16()?;
            let value = cp_utf8(scan.pool(), index)?;
            let (index, span) = scan.indexed(index)?;
            scan.literal(site, LiteralValue::String { value }, index, span);
        }
        b'e' => {
            let type_index = reader.u16()?;
            let const_index = reader.u16()?;
            let descriptor = cp_utf8(scan.pool(), type_index)?;
            emit_indexed_descriptor(scan, site, &descriptor.0, DescriptorKind::Field, type_index)?;
            let constant = cp_utf8(scan.pool(), const_index)?;
            let (index, span) = scan.indexed(const_index)?;
            scan.literal(site, LiteralValue::String { value: constant }, index, span);
        }
        b'c' => {
            let index = reader.u16()?;
            let descriptor = cp_utf8(scan.pool(), index)?;
            emit_indexed_descriptor(scan, site, &descriptor.0, DescriptorKind::Return, index)?;
            let (index, span) = scan.indexed(index)?;
            scan.literal(site, LiteralValue::Class { value: descriptor }, index, span);
        }
        b'@' => annotation(reader, scan, site)?,
        b'[' => {
            for _ in 0..reader.u16()? {
                element_value(reader, scan, site)?;
            }
        }
        _ => return Err(reader.malformed("annotation element value tag is unknown")),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Exceptions, constant values, inner/nest and modules
// ---------------------------------------------------------------------------

/// Method `Exceptions` attributes. Code exception-table catch types belong to the
/// code consumer, so this category never reads a `Code` attribute.
fn scan_exceptions(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    let methods: Vec<Vec<AttributeShell>> = scan
        .units
        .facts
        .methods
        .iter()
        .map(|method| method.attributes.clone())
        .collect();
    for shells in &methods {
        let Some((facts, span)) = read_named_attribute(scan, shells, b"Exceptions")? else {
            continue;
        };
        let site = Site {
            consumer: ConsumerKind::Exception,
            operation: XrefOperation::ExceptionsAttribute,
            attribute: Some(b"Exceptions"),
        };
        for name in &facts.exceptions {
            let (index, located) = scan.resolved_class(&name.0, Some(span.clone()));
            scan.class(site, &name.0, index, located);
        }
    }
    Ok(())
}

/// Field `ConstantValue` attributes: the value reference only.
///
/// The value is what the field declaration records, not a claim about where an
/// inlined constant came from, so the target is the value and never the field.
fn scan_constants(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    let fields: Vec<Vec<AttributeShell>> = scan
        .units
        .facts
        .fields
        .iter()
        .map(|field| field.attributes.clone())
        .collect();
    for shells in &fields {
        let Some((facts, _)) = read_named_attribute(scan, shells, b"ConstantValue")? else {
            continue;
        };
        let Some(value_index) = facts.constant_value else {
            continue;
        };
        let index = value_index.0;
        let entry = cp_entry(scan.pool(), index)?;
        let value = match &entry.kind {
            CpEntryKind::Integer { value } => LiteralValue::Integer { value: *value },
            CpEntryKind::Long { value } => LiteralValue::Long { value: *value },
            CpEntryKind::Float { bits } => LiteralValue::Float { value: *bits },
            CpEntryKind::Double { bits } => LiteralValue::Double { value: *bits },
            CpEntryKind::String { value, .. } => LiteralValue::String {
                value: value.clone(),
            },
            _ => {
                return Err(Error::invalid_input(
                    CONSTANT_VALUE_CODE,
                    "ConstantValue must reference a String or primitive constant",
                ));
            }
        };
        let site = Site {
            consumer: ConsumerKind::Constant,
            operation: XrefOperation::ConstantValue,
            attribute: Some(b"ConstantValue"),
        };
        let (index, span) = scan.indexed(index)?;
        scan.literal(site, value, index, span);
    }
    Ok(())
}

/// `InnerClasses`, `EnclosingMethod`, `NestHost`, `NestMembers` and
/// `PermittedSubclasses`.
///
/// The relation kind is kept: the inner class and its declaring class both report
/// `InnerClass`, the enclosing class and method report `EnclosingMethod`, and the
/// nest and permitted lists keep their own operations. Nothing is inferred from a
/// `$` in a name.
fn scan_inner_nest(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    let class_shells = scan.units.facts.attributes.clone();
    let inner = Site {
        consumer: ConsumerKind::InnerNest,
        operation: XrefOperation::InnerClass,
        attribute: Some(b"InnerClasses"),
    };
    if let Some((facts, _)) = read_named_attribute(scan, &class_shells, b"InnerClasses")? {
        for entry in &facts.inner_classes {
            let name = cp_class_name(scan.pool(), entry.class_index)?;
            let (index, span) = scan.indexed(entry.class_index)?;
            scan.class(inner, &name.0, index, span);
            if entry.outer_class_index != 0 {
                let outer = cp_class_name(scan.pool(), entry.outer_class_index)?;
                let (index, span) = scan.indexed(entry.outer_class_index)?;
                scan.class(inner, &outer.0, index, span);
            }
        }
    }
    let enclosing = Site {
        consumer: ConsumerKind::InnerNest,
        operation: XrefOperation::EnclosingMethod,
        attribute: Some(b"EnclosingMethod"),
    };
    if let Some((facts, _)) = read_named_attribute(scan, &class_shells, b"EnclosingMethod")?
        && let Some(method) = &facts.enclosing_method
    {
        let owner = cp_class_name(scan.pool(), method.class_index)?;
        let (index, span) = scan.indexed(method.class_index)?;
        scan.class(enclosing, &owner.0, index, span);
        if method.method_index != 0 {
            // The enclosing member is a method (or constructor) of the enclosing
            // class, so the symbol carries all three parts.
            let (name, descriptor) = match &cp_entry(scan.pool(), method.method_index)?.kind {
                CpEntryKind::NameAndType {
                    name, descriptor, ..
                } => (name.clone(), descriptor.clone()),
                _ => {
                    return Err(Error::invalid_input(
                        ENCLOSING_METHOD_CODE,
                        "EnclosingMethod must reference a NameAndType",
                    ));
                }
            };
            let (index, span) = scan.indexed(method.method_index)?;
            scan.method(enclosing, &owner.0, &name.0, &descriptor.0, index, span);
        }
    }
    let host_site = Site {
        consumer: ConsumerKind::InnerNest,
        operation: XrefOperation::NestHost,
        attribute: Some(b"NestHost"),
    };
    if let Some((facts, span)) = read_named_attribute(scan, &class_shells, b"NestHost")?
        && let Some(host) = &facts.nest_host
    {
        let (index, located) = scan.resolved_class(&host.0, Some(span.clone()));
        scan.class(host_site, &host.0, index, located);
    }
    if let Some((facts, span)) = read_named_attribute(scan, &class_shells, b"NestMembers")? {
        let site = Site {
            consumer: ConsumerKind::InnerNest,
            operation: XrefOperation::NestMembers,
            attribute: Some(b"NestMembers"),
        };
        for name in &facts.nest_members {
            let (index, located) = scan.resolved_class(&name.0, Some(span.clone()));
            scan.class(site, &name.0, index, located);
        }
    }
    if let Some((facts, span)) = read_named_attribute(scan, &class_shells, b"PermittedSubclasses")?
    {
        let site = Site {
            consumer: ConsumerKind::InnerNest,
            operation: XrefOperation::PermittedSubclasses,
            attribute: Some(b"PermittedSubclasses"),
        };
        for name in &facts.permitted_subclasses {
            let (index, located) = scan.resolved_class(&name.0, Some(span.clone()));
            scan.class(site, &name.0, index, located);
        }
    }
    Ok(())
}

/// `Module` `uses` and `provides`, and only in a `module-info` class.
///
/// Every node this consumer reports is a class: `uses` names a service interface and
/// `provides` names a service interface plus its implementations. `exports`,
/// `requires` and `opens` are not resolved here, so no package or module
/// relationship is claimed.
fn scan_module(scan: &mut Scan<'_, '_, '_>) -> Result<()> {
    if scan.units.facts.this_class.raw().0.as_slice() != MODULE_INFO_CLASS {
        return Ok(());
    }
    let class_shells = scan.units.facts.attributes.clone();
    let Some((facts, span)) = read_named_attribute(scan, &class_shells, b"Module")? else {
        return Ok(());
    };
    let Some(module) = facts.module else {
        return Ok(());
    };
    let uses = Site {
        consumer: ConsumerKind::Module,
        operation: XrefOperation::ModuleUses,
        attribute: Some(b"Module"),
    };
    for service in &module.uses {
        let (index, located) = scan.resolved_class(&service.0, Some(span.clone()));
        scan.class(uses, &service.0, index, located);
    }
    let provides = Site {
        consumer: ConsumerKind::Module,
        operation: XrefOperation::ModuleProvides,
        attribute: Some(b"Module"),
    };
    for declaration in &module.provides {
        let (index, located) = scan.resolved_class(&declaration.service.0, Some(span.clone()));
        scan.class(provides, &declaration.service.0, index, located);
        for implementation in &declaration.implementations {
            let (index, located) = scan.resolved_class(&implementation.0, Some(span.clone()));
            scan.class(provides, &implementation.0, index, located);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Reader facts
// ---------------------------------------------------------------------------

/// Reads the shells of exactly one attribute name through the reader fact layer.
///
/// Filtering the shell list first is what keeps this scan honest about scope: only
/// the requested attribute is read (and billed), so an attribute of a category this
/// request did not name can neither produce a fact nor fail the scan. A duplicate of
/// the same attribute stays visible to the reader layer's own uniqueness check.
fn read_named_attribute(
    scan: &mut Scan<'_, '_, '_>,
    shells: &[AttributeShell],
    name: &[u8],
) -> Result<Option<(AttributeFacts, ByteSpan)>> {
    let selected: Vec<AttributeShell> = shells
        .iter()
        .filter(|shell| shell.name.raw().0.as_slice() == name)
        .cloned()
        .collect();
    let Some(first) = selected.first() else {
        return Ok(None);
    };
    let span = first.content_span.clone();
    let facts = {
        let budget = scan.ctx.budget();
        let bytes = scan.units.bytes;
        let pool = scan.units.pool();
        attribute_facts(bytes, &selected, pool, budget)?
    };
    Ok(Some((facts, span)))
}

/// Bounded reader over one class-file region.
///
/// Every read is bounds-checked and callers must reach `expect_end`, so a structure
/// that does not end exactly at the region end is an error instead of a silently
/// truncated fact set. A reader built with [`Reader::located`] also knows where its region
/// starts in the class file, so a structure inside it can be reported as an absolute span
/// ([`Reader::here`], [`Reader::span_since`]) that a caller can slice out of the class
/// bytes.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    /// Class-file offset of the region start.
    base: u64,
    code: &'static str,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], code: &'static str) -> Self {
        Self {
            bytes,
            at: 0,
            base: 0,
            code,
        }
    }

    /// A reader over `bytes`, which begin at `base` in the class file.
    fn located(bytes: &'a [u8], base: u64, code: &'static str) -> Self {
        Self {
            bytes,
            at: 0,
            base,
            code,
        }
    }

    /// Class-file offset of the current position.
    fn here(&self) -> Result<u64> {
        u64::try_from(self.at)
            .ok()
            .and_then(|at| self.base.checked_add(at))
            .ok_or_else(|| self.malformed("region offset overflow"))
    }

    /// Class-file range of the bytes from `start` to the current position.
    fn span_since(&self, start: u64) -> Result<ByteSpan> {
        let end = self.here()?;
        let length = end
            .checked_sub(start)
            .ok_or_else(|| self.malformed("structure ends before it starts"))?;
        Ok(ByteSpan::new(start, length))
    }

    fn is_empty(&self) -> bool {
        self.at >= self.bytes.len()
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(count)
            .ok_or_else(|| self.malformed("region length overflow"))?;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| self.malformed("region ends before the structure does"))?;
        self.at = end;
        Ok(slice)
    }

    fn skip(&mut self, count: usize) -> Result<()> {
        self.take(count).map(|_| ())
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn expect_end(&self) -> Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.malformed("structure does not end at the region end"))
        }
    }

    /// Reads one nested `attribute_info` body declared as `length` bytes (JVMS 4.7).
    ///
    /// `parse` reads the structure this consumer understands from the current
    /// position; whatever the declaration still covers is then skipped, so the reader
    /// stays aligned with the surrounding attribute list instead of trusting the
    /// structure to describe the whole declaration. A structure that needs more bytes
    /// than the attribute declares is a structured error: the declaration and the
    /// content disagree about the same bytes, so nothing from that attribute is a fact.
    ///
    /// This is the only place a declared length becomes a skip, and it is why no parser
    /// in this module spells a magic content length.
    fn attribute_body<T>(
        &mut self,
        length: u64,
        parse: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let length = usize::try_from(length)
            .map_err(|_| self.malformed("attribute length does not fit a byte range"))?;
        let start = self.at;
        let parsed = parse(self)?;
        let consumed = self.at - start;
        let rest = length
            .checked_sub(consumed)
            .ok_or_else(|| self.malformed("attribute body is longer than its declared length"))?;
        self.skip(rest)?;
        Ok(parsed)
    }

    fn malformed(&self, message: &str) -> Error {
        Error::invalid_input(
            self.code,
            format!("{message} (at byte {} of the region)", self.at),
        )
    }
}
