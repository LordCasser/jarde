//! P1 metadata consumer acceptance (task 2.2): hierarchy, descriptors, generic
//! signatures, annotations, inner/nest relations, `Exceptions`, `ConstantValue` and
//! module uses/provides, exercised through the public `Engine::query` entry point.
//!
//! Fixtures are hand-built class files, so a fixture contains only the structure an
//! assertion needs and every byte charge can be derived from what the writer wrote
//! instead of from a hand-kept constant.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const ACCESS_PUBLIC: u16 = 0x0001;
const ACCESS_PRIVATE: u16 = 0x0002;
const ACCESS_SUPER: u16 = 0x0020;
const ACCESS_MODULE: u16 = 0x8000;

fn be16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn be32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

// ---------------------------------------------------------------------------
// Class-file writer
// ---------------------------------------------------------------------------

/// Constant-pool writer.
///
/// `Utf8` and `Class` entries are interned by value the way a compiler does, and
/// every entry keeps its byte range, so tests can assert the pool entry an item
/// points at instead of trusting a number copied from the implementation.
struct Cp {
    bytes: Vec<u8>,
    next: u16,
    strings: Vec<(Vec<u8>, u16)>,
    classes: Vec<(Vec<u8>, u16)>,
    literals: Vec<(Vec<u8>, u16)>,
    spans: Vec<(u16, u64, u64)>,
}

impl Cp {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            next: 1,
            strings: Vec::new(),
            classes: Vec::new(),
            literals: Vec::new(),
            spans: Vec::new(),
        }
    }

    fn entry(&mut self, tag: u8, payload: impl FnOnce(&mut Vec<u8>)) -> u16 {
        let index = self.next;
        let offset = self.bytes.len() as u64;
        self.bytes.push(tag);
        payload(&mut self.bytes);
        self.spans
            .push((index, offset, self.bytes.len() as u64 - offset));
        self.next += 1;
        index
    }

    /// A two-slot entry whose following slot is unusable.
    fn wide(&mut self, tag: u8, payload: impl FnOnce(&mut Vec<u8>)) -> u16 {
        let index = self.entry(tag, payload);
        self.next += 1;
        index
    }

    fn utf8(&mut self, value: &[u8]) -> u16 {
        if let Some((_, index)) = self.strings.iter().find(|(bytes, _)| bytes == value) {
            return *index;
        }
        let index = self.entry(1, |bytes| {
            be16(bytes, u16::try_from(value.len()).unwrap());
            bytes.extend_from_slice(value);
        });
        self.strings.push((value.to_vec(), index));
        index
    }

    fn class(&mut self, name: &[u8]) -> u16 {
        if let Some((_, index)) = self.classes.iter().find(|(bytes, _)| bytes == name) {
            return *index;
        }
        let name_index = self.utf8(name);
        let index = self.entry(7, |bytes| be16(bytes, name_index));
        self.classes.push((name.to_vec(), index));
        index
    }

    fn integer(&mut self, value: i32) -> u16 {
        self.entry(3, |bytes| bytes.extend_from_slice(&value.to_be_bytes()))
    }

    fn long(&mut self, value: i64) -> u16 {
        self.wide(5, |bytes| bytes.extend_from_slice(&value.to_be_bytes()))
    }

    fn double(&mut self, bits: u64) -> u16 {
        self.wide(6, |bytes| bytes.extend_from_slice(&bits.to_be_bytes()))
    }

    fn string(&mut self, value: &[u8]) -> u16 {
        let utf8_index = self.utf8(value);
        let index = self.entry(8, |bytes| be16(bytes, utf8_index));
        self.literals.push((value.to_vec(), index));
        index
    }

    fn name_and_type(&mut self, name: &[u8], descriptor: &[u8]) -> u16 {
        let name_index = self.utf8(name);
        let descriptor_index = self.utf8(descriptor);
        self.entry(12, |bytes| {
            be16(bytes, name_index);
            be16(bytes, descriptor_index);
        })
    }

    fn module(&mut self, name: &[u8]) -> u16 {
        let name_index = self.utf8(name);
        self.entry(19, |bytes| be16(bytes, name_index))
    }

    /// One extra `CONSTANT_Utf8` with these bytes even when an equal entry exists.
    ///
    /// A pool is interned only by convention: another writer may store one entry per
    /// use site, and then no constant-pool entry is the entry a fact came from.
    fn utf8_duplicate(&mut self, value: &[u8]) -> u16 {
        let index = self.entry(1, |bytes| {
            be16(bytes, u16::try_from(value.len()).unwrap());
            bytes.extend_from_slice(value);
        });
        self.strings.push((value.to_vec(), index));
        index
    }

    /// One extra `CONSTANT_Class` naming the same internal name as an existing entry.
    fn class_duplicate(&mut self, name: &[u8]) -> u16 {
        let name_index = self.utf8(name);
        let index = self.entry(7, |bytes| be16(bytes, name_index));
        self.classes.push((name.to_vec(), index));
        index
    }

    /// Raw bytes of one `CONSTANT_Utf8` entry.
    fn utf8_bytes(&self, index: u16) -> Option<&[u8]> {
        self.strings
            .iter()
            .find(|(_, entry)| *entry == index)
            .map(|(bytes, _)| bytes.as_slice())
    }
}

/// One pending attribute: raw name bytes plus content bytes.
struct Pending {
    name: &'static [u8],
    content: Vec<u8>,
}

fn attribute(name: &'static [u8], content: Vec<u8>) -> Pending {
    Pending { name, content }
}

struct Member {
    access_flags: u16,
    name: u16,
    descriptor: u16,
    attributes: Vec<(u16, Vec<u8>)>,
}

/// Class-file writer with the pool and structure it needs.
struct Class {
    cp: Cp,
    major: u16,
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<u16>,
    fields: Vec<Member>,
    methods: Vec<Member>,
    attributes: Vec<(u16, Vec<u8>)>,
}

impl Class {
    fn new(this: &[u8], super_name: Option<&[u8]>) -> Self {
        let mut class = Self {
            cp: Cp::new(),
            major: 52,
            access_flags: ACCESS_PUBLIC | ACCESS_SUPER,
            this_class: 0,
            super_class: 0,
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
        };
        class.this_class = class.cp.class(this);
        class.super_class = match super_name {
            Some(name) => class.cp.class(name),
            None => 0,
        };
        class
    }

    fn major(&mut self, major: u16) {
        self.major = major;
    }

    fn access_flags(&mut self, access_flags: u16) {
        self.access_flags = access_flags;
    }

    fn utf8(&mut self, value: &[u8]) -> u16 {
        self.cp.utf8(value)
    }

    fn class_index(&mut self, name: &[u8]) -> u16 {
        self.cp.class(name)
    }

    fn integer(&mut self, value: i32) -> u16 {
        self.cp.integer(value)
    }

    fn long(&mut self, value: i64) -> u16 {
        self.cp.long(value)
    }

    fn double(&mut self, bits: u64) -> u16 {
        self.cp.double(bits)
    }

    fn string(&mut self, value: &[u8]) -> u16 {
        self.cp.string(value)
    }

    fn module(&mut self, name: &[u8]) -> u16 {
        self.cp.module(name)
    }

    /// One extra `CONSTANT_Utf8` entry with these bytes, even when an equal one exists.
    fn duplicate_utf8(&mut self, value: &[u8]) -> u16 {
        self.cp.utf8_duplicate(value)
    }

    /// One extra `CONSTANT_Class` entry naming the same internal name.
    fn duplicate_class(&mut self, name: &[u8]) -> u16 {
        self.cp.class_duplicate(name)
    }

    fn interface(&mut self, name: &[u8]) {
        let index = self.cp.class(name);
        self.interfaces.push(index);
    }

    fn field(&mut self, access_flags: u16, name: &[u8], descriptor: &[u8], attributes: &[Pending]) {
        let name = self.cp.utf8(name);
        let descriptor = self.cp.utf8(descriptor);
        let attributes = self.intern(attributes);
        self.fields.push(Member {
            access_flags,
            name,
            descriptor,
            attributes,
        });
    }

    fn method(
        &mut self,
        access_flags: u16,
        name: &[u8],
        descriptor: &[u8],
        attributes: &[Pending],
    ) {
        let name = self.cp.utf8(name);
        let descriptor = self.cp.utf8(descriptor);
        let attributes = self.intern(attributes);
        self.methods.push(Member {
            access_flags,
            name,
            descriptor,
            attributes,
        });
    }

    fn class_attributes(&mut self, attributes: &[Pending]) {
        let attributes = self.intern(attributes);
        self.attributes.extend(attributes);
    }

    fn intern(&mut self, attributes: &[Pending]) -> Vec<(u16, Vec<u8>)> {
        attributes
            .iter()
            .map(|pending| (self.cp.utf8(pending.name), pending.content.clone()))
            .collect()
    }

    fn finish(self) -> Built {
        let Self {
            cp,
            major,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        } = self;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
        be16(&mut bytes, 0);
        be16(&mut bytes, major);
        be16(&mut bytes, cp.next);
        bytes.extend_from_slice(&cp.bytes);
        be16(&mut bytes, access_flags);
        be16(&mut bytes, this_class);
        be16(&mut bytes, super_class);
        be16(&mut bytes, u16::try_from(interfaces.len()).unwrap());
        for interface in &interfaces {
            be16(&mut bytes, *interface);
        }
        // Attribute ranges are recorded while the bytes are written, so a test asserts
        // the offsets the writer produced instead of recomputing them.
        let mut written = Vec::new();
        be16(&mut bytes, u16::try_from(fields.len()).unwrap());
        for field in &fields {
            write_member(&mut bytes, field, &cp, &mut written);
        }
        be16(&mut bytes, u16::try_from(methods.len()).unwrap());
        for method in &methods {
            write_member(&mut bytes, method, &cp, &mut written);
        }
        be16(&mut bytes, u16::try_from(attributes.len()).unwrap());
        for (name, content) in &attributes {
            write_attribute(&mut bytes, *name, content, &cp, &mut written);
        }
        Built {
            bytes,
            spans: cp.spans,
            written,
            classes: cp.classes,
            strings: cp.strings,
            literals: cp.literals,
        }
    }
}

fn write_member(
    out: &mut Vec<u8>,
    member: &Member,
    pool: &Cp,
    written: &mut Vec<WrittenAttribute>,
) {
    be16(out, member.access_flags);
    be16(out, member.name);
    be16(out, member.descriptor);
    be16(out, u16::try_from(member.attributes.len()).unwrap());
    for (name, content) in &member.attributes {
        write_attribute(out, *name, content, pool, written);
    }
}

fn write_attribute(
    out: &mut Vec<u8>,
    name_index: u16,
    content: &[u8],
    pool: &Cp,
    written: &mut Vec<WrittenAttribute>,
) {
    let entry_start = out.len() as u64;
    be16(out, name_index);
    be32(out, u32::try_from(content.len()).unwrap());
    let content_start = out.len() as u64;
    out.extend_from_slice(content);
    written.push(WrittenAttribute {
        name: pool
            .utf8_bytes(name_index)
            .expect("an attribute name must be a Utf8 entry")
            .to_vec(),
        entry_span: ByteSpan::new(entry_start, out.len() as u64 - entry_start),
        content_span: ByteSpan::new(content_start, content.len() as u64),
    });
}

/// One attribute entry the writer wrote, located inside the fixture it built.
struct WrittenAttribute {
    /// Raw attribute name bytes.
    name: Vec<u8>,
    /// Byte range of the whole `attribute_info`: name index, length and content.
    entry_span: ByteSpan,
    /// Byte range of the attribute content a reader slices.
    content_span: ByteSpan,
}

/// One finished fixture plus the pool facts a test needs to pin evidence.
struct Built {
    bytes: Vec<u8>,
    spans: Vec<(u16, u64, u64)>,
    written: Vec<WrittenAttribute>,
    classes: Vec<(Vec<u8>, u16)>,
    strings: Vec<(Vec<u8>, u16)>,
    literals: Vec<(Vec<u8>, u16)>,
}

impl Built {
    /// Class-file byte range of one constant-pool entry, as the writer laid it out.
    fn entry_span(&self, index: u16) -> ByteSpan {
        let (_, offset, width) = self
            .spans
            .iter()
            .find(|(entry, _, _)| *entry == index)
            .expect("fixture constant-pool entry must exist");
        ByteSpan::new(10 + offset, *width)
    }

    fn class_index(&self, name: &[u8]) -> u16 {
        self.classes
            .iter()
            .find(|(bytes, _)| bytes == name)
            .map(|(_, index)| *index)
            .expect("fixture must have a CONSTANT_Class for this name")
    }

    /// Whether the fixture has a `CONSTANT_Class` for this name.
    fn has_class(&self, name: &[u8]) -> bool {
        self.classes.iter().any(|(bytes, _)| bytes == name)
    }

    fn utf8_index(&self, value: &[u8]) -> u16 {
        self.strings
            .iter()
            .find(|(bytes, _)| bytes == value)
            .map(|(_, index)| *index)
            .expect("fixture must have a CONSTANT_Utf8 for this value")
    }

    /// The `CONSTANT_String` entry of a value, when the fixture has one.
    fn string_constant(&self, value: &[u8]) -> Option<u16> {
        self.literals
            .iter()
            .find(|(bytes, _)| bytes == value)
            .map(|(_, index)| *index)
    }

    /// Every attribute entry (shell plus content) the writer wrote.
    fn attribute_bytes(&self) -> u64 {
        self.written
            .iter()
            .map(|attribute| attribute.entry_span.length)
            .sum()
    }

    /// Attribute entry bytes of the attributes with one of these names.
    fn attribute_bytes_of(&self, names: &[&[u8]]) -> u64 {
        self.written
            .iter()
            .filter(|attribute| names.contains(&attribute.name.as_slice()))
            .map(|attribute| attribute.entry_span.length)
            .sum()
    }

    /// Content range of the first attribute with this name.
    fn attribute_content_span(&self, name: &[u8]) -> ByteSpan {
        self.written
            .iter()
            .find(|attribute| attribute.name == name)
            .map(|attribute| attribute.content_span.clone())
            .expect("fixture must have an attribute with this name")
    }

    /// Content bytes of the first attribute with this name.
    fn attribute_content(&self, name: &[u8]) -> Vec<u8> {
        let span = self.attribute_content_span(name);
        let start = usize::try_from(span.start).unwrap();
        let end = start + usize::try_from(span.length).unwrap();
        self.bytes[start..end].to_vec()
    }

    /// Raw bytes of one constant-pool `Utf8` entry of the fixture.
    fn utf8_bytes(&self, index: u16) -> Vec<u8> {
        self.strings
            .iter()
            .find(|(_, entry)| *entry == index)
            .map(|(bytes, _)| bytes.clone())
            .expect("fixture must have a CONSTANT_Utf8 for this index")
    }
}

/// Declared content length of every nested attribute inside a `Record` content.
///
/// The fixture writer lays the declarations out, and this reads them back so a test can
/// state the length a parser must honour instead of assuming one.
fn record_component_attributes(built: &Built, content: &[u8]) -> Vec<(Vec<u8>, u32)> {
    let u16_at = |at: &mut usize| {
        let value = u16::from_be_bytes([content[*at], content[*at + 1]]);
        *at += 2;
        value
    };
    let u32_at = |at: &mut usize| {
        let value = u32::from_be_bytes([
            content[*at],
            content[*at + 1],
            content[*at + 2],
            content[*at + 3],
        ]);
        *at += 4;
        value
    };
    let mut at = 0_usize;
    let components = u16_at(&mut at);
    let mut declarations = Vec::new();
    for _ in 0..components {
        let _name_index = u16_at(&mut at);
        let _descriptor_index = u16_at(&mut at);
        let attributes = u16_at(&mut at);
        for _ in 0..attributes {
            let name_index = u16_at(&mut at);
            let length = u32_at(&mut at);
            declarations.push((built.utf8_bytes(name_index), length));
            at += usize::try_from(length).unwrap();
        }
    }
    assert_eq!(at, content.len(), "the declarations must cover the content");
    declarations
}

// ---------------------------------------------------------------------------
// Attribute content encoders
// ---------------------------------------------------------------------------

/// One annotation element of a fixture: member name plus value.
type Element = (&'static [u8], Value);

/// One annotation of a fixture: type descriptor plus its element values.
type Annotation<'a> = (&'a [u8], Vec<Element>);

/// One `InnerClasses` entry: inner class, outer class (or `None`) and simple name (or
/// `None` for an anonymous entry).
type InnerEntry<'a> = (&'a [u8], Option<&'a [u8]>, Option<&'a [u8]>);

/// One `element_value` with its JVMS 4.7.16.1 tag.
enum Value {
    Bool(bool),
    Int(i32),
    Long(i64),
    Double(u64),
    Str(Vec<u8>),
    Class(Vec<u8>),
    Enum {
        descriptor: Vec<u8>,
        constant: Vec<u8>,
    },
    Anno {
        descriptor: Vec<u8>,
        elements: Vec<Element>,
    },
    Array(Vec<Value>),
}

impl Value {
    fn encode(&self, class: &mut Class) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            Value::Bool(value) => {
                out.push(b'Z');
                let index = class.integer(i32::from(*value));
                be16(&mut out, index);
            }
            Value::Int(value) => {
                out.push(b'I');
                let index = class.integer(*value);
                be16(&mut out, index);
            }
            Value::Long(value) => {
                out.push(b'J');
                let index = class.long(*value);
                be16(&mut out, index);
            }
            Value::Double(bits) => {
                out.push(b'D');
                let index = class.double(*bits);
                be16(&mut out, index);
            }
            Value::Str(value) => {
                out.push(b's');
                let index = class.utf8(value);
                be16(&mut out, index);
            }
            Value::Class(descriptor) => {
                out.push(b'c');
                let index = class.utf8(descriptor);
                be16(&mut out, index);
            }
            Value::Enum {
                descriptor,
                constant,
            } => {
                out.push(b'e');
                let type_index = class.utf8(descriptor);
                let constant_index = class.utf8(constant);
                be16(&mut out, type_index);
                be16(&mut out, constant_index);
            }
            Value::Anno {
                descriptor,
                elements,
            } => {
                out.push(b'@');
                out.extend_from_slice(&annotation(class, descriptor, elements));
            }
            Value::Array(values) => {
                out.push(b'[');
                be16(&mut out, u16::try_from(values.len()).unwrap());
                for value in values {
                    out.extend_from_slice(&value.encode(class));
                }
            }
        }
        out
    }
}

/// One `annotation` structure (JVMS 4.7.16) without a tag.
fn annotation(
    class: &mut Class,
    descriptor: &[u8],
    elements: &[(&'static [u8], Value)],
) -> Vec<u8> {
    let mut out = Vec::new();
    let type_index = class.utf8(descriptor);
    be16(&mut out, type_index);
    be16(&mut out, u16::try_from(elements.len()).unwrap());
    for (name, value) in elements {
        let name_index = class.utf8(name);
        be16(&mut out, name_index);
        out.extend_from_slice(&value.encode(class));
    }
    out
}

/// `RuntimeVisible/InvisibleAnnotations` content.
fn annotations(class: &mut Class, items: &[Annotation<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    be16(&mut out, u16::try_from(items.len()).unwrap());
    for (descriptor, elements) in items {
        out.extend_from_slice(&annotation(class, descriptor, elements));
    }
    out
}

/// One type annotation whose `target_info` is already encoded.
struct TypeAnno<'a> {
    target_type: u8,
    target_info: Vec<u8>,
    descriptor: &'a [u8],
    elements: Vec<Element>,
}

/// `RuntimeVisible/InvisibleTypeAnnotations` content with an empty `type_path`.
fn type_annotations(class: &mut Class, items: &[TypeAnno<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    be16(&mut out, u16::try_from(items.len()).unwrap());
    for item in items {
        out.push(item.target_type);
        out.extend_from_slice(&item.target_info);
        out.push(0);
        out.extend_from_slice(&annotation(class, item.descriptor, &item.elements));
    }
    out
}

/// `RuntimeVisible/InvisibleParameterAnnotations` content.
fn parameter_annotations(class: &mut Class, parameters: &[Vec<Annotation<'_>>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(u8::try_from(parameters.len()).unwrap());
    for items in parameters {
        be16(&mut out, u16::try_from(items.len()).unwrap());
        for (descriptor, elements) in items {
            out.extend_from_slice(&annotation(class, descriptor, elements));
        }
    }
    out
}

/// `Signature` content: the index of the signature string.
fn signature(class: &mut Class, signature: &[u8]) -> Vec<u8> {
    let index = class.utf8(signature);
    index.to_be_bytes().to_vec()
}

/// A `CONSTANT_Class` index list content (`Exceptions`, `NestMembers`,
/// `PermittedSubclasses`).
fn class_list(class: &mut Class, names: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    be16(&mut out, u16::try_from(names.len()).unwrap());
    for name in names {
        let index = class.class_index(name);
        be16(&mut out, index);
    }
    out
}

/// `InnerClasses` content. `inner_name` is `None` for an anonymous entry.
fn inner_classes(class: &mut Class, entries: &[InnerEntry<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    be16(&mut out, u16::try_from(entries.len()).unwrap());
    for (inner, outer, inner_name) in entries {
        let class_index = class.class_index(inner);
        let outer_index = match outer {
            Some(name) => class.class_index(name),
            None => 0,
        };
        let name_index = match inner_name {
            Some(name) => class.utf8(name),
            None => 0,
        };
        be16(&mut out, class_index);
        be16(&mut out, outer_index);
        be16(&mut out, name_index);
        be16(&mut out, 0);
    }
    out
}

/// `EnclosingMethod` content.
fn enclosing_method(class: &mut Class, owner: &[u8], name: &[u8], descriptor: &[u8]) -> Vec<u8> {
    let class_index = class.class_index(owner);
    let method_index = class.cp.name_and_type(name, descriptor);
    let mut out = Vec::new();
    be16(&mut out, class_index);
    be16(&mut out, method_index);
    out
}

/// `NestHost` content.
fn nest_host(class: &mut Class, host: &[u8]) -> Vec<u8> {
    let index = class.class_index(host);
    index.to_be_bytes().to_vec()
}

/// A minimal `Code` attribute content: no instructions and no handlers.
///
/// Every method of a fixture carries one, because a class file that a compiler could
/// have written has a `Code` attribute per concrete method and the code consumer reads
/// exactly that attribute.
fn empty_code() -> Vec<u8> {
    let mut out = Vec::new();
    be16(&mut out, 0); // max_stack
    be16(&mut out, 0); // max_locals
    be32(&mut out, 0); // code_length
    be16(&mut out, 0); // exception_table_length
    be16(&mut out, 0); // attributes_count
    out
}

/// `Module` content with no `requires`, `exports` or `opens`.
fn module_attribute(
    class: &mut Class,
    name: &[u8],
    uses: &[&[u8]],
    provides: &[(&[u8], &[&[u8]])],
) -> Vec<u8> {
    let module_name = class.module(name);
    let mut out = Vec::new();
    be16(&mut out, module_name);
    be16(&mut out, 0); // module_flags
    be16(&mut out, 0); // module_version_index
    be16(&mut out, 0); // requires_count
    be16(&mut out, 0); // exports_count
    be16(&mut out, 0); // opens_count
    be16(&mut out, u16::try_from(uses.len()).unwrap());
    for service in uses {
        let index = class.class_index(service);
        be16(&mut out, index);
    }
    be16(&mut out, u16::try_from(provides.len()).unwrap());
    for (service, implementations) in provides {
        let index = class.class_index(service);
        be16(&mut out, index);
        be16(&mut out, u16::try_from(implementations.len()).unwrap());
        for implementation in *implementations {
            let index = class.class_index(implementation);
            be16(&mut out, index);
        }
    }
    out
}

/// `Record` content: (component name, descriptor, component attributes).
fn record(class: &mut Class, components: &[(&[u8], &[u8], Vec<Pending>)]) -> Vec<u8> {
    let mut out = Vec::new();
    be16(&mut out, u16::try_from(components.len()).unwrap());
    for (name, descriptor, attributes) in components {
        let name_index = class.utf8(name);
        let descriptor_index = class.utf8(descriptor);
        be16(&mut out, name_index);
        be16(&mut out, descriptor_index);
        be16(&mut out, u16::try_from(attributes.len()).unwrap());
        for pending in attributes {
            let attribute_name = class.utf8(pending.name);
            be16(&mut out, attribute_name);
            be32(&mut out, u32::try_from(pending.content.len()).unwrap());
            out.extend_from_slice(&pending.content);
        }
    }
    out
}

/// `Record` content with one component and one component attribute whose declared
/// length is given by the test, so a declaration can disagree with its content.
fn record_with_declared_length(
    class: &mut Class,
    name: &[u8],
    descriptor: &[u8],
    attribute_name: &[u8],
    declared_length: u32,
    content: &[u8],
) -> Vec<u8> {
    let name_index = class.utf8(name);
    let descriptor_index = class.utf8(descriptor);
    let attribute_name = class.utf8(attribute_name);
    let mut out = Vec::new();
    be16(&mut out, 1); // components_count
    be16(&mut out, name_index);
    be16(&mut out, descriptor_index);
    be16(&mut out, 1); // attributes_count
    be16(&mut out, attribute_name);
    be32(&mut out, declared_length);
    out.extend_from_slice(content);
    out
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// One class where three types exist only in metadata:
///
/// * `p/OnlyAnnotated` — an annotation descriptor, plus `p/Nested`, `p/Mode` and
///   `p/Literal` from nested annotation, enum and class-literal element values,
/// * `p/OnlySigned` — a field `Signature`, and also the *string* element value
///   `"p/OnlySigned"` of that annotation, which is a value and not a type,
/// * `p/OnlyThrown` — a method `Exceptions` attribute.
fn meta_fixture() -> Built {
    let mut class = Class::new(b"p/Meta", Some(b"java/lang/Object"));

    let visible = annotations(
        &mut class,
        &[(
            b"Lp/OnlyAnnotated;",
            vec![
                (b"string", Value::Str(b"p/OnlySigned".to_vec())),
                (b"number", Value::Int(7)),
                (b"literal", Value::Class(b"Lp/Literal;".to_vec())),
                (
                    b"mode",
                    Value::Enum {
                        descriptor: b"Lp/Mode;".to_vec(),
                        constant: b"FAST".to_vec(),
                    },
                ),
                (
                    b"nested",
                    Value::Anno {
                        descriptor: b"Lp/Nested;".to_vec(),
                        elements: vec![
                            (b"flag", Value::Bool(true)),
                            (b"ints", Value::Array(vec![Value::Int(1), Value::Int(2)])),
                        ],
                    },
                ),
            ],
        )],
    );
    class.field(
        ACCESS_PRIVATE,
        b"annotated",
        b"I",
        &[attribute(b"RuntimeVisibleAnnotations", visible)],
    );

    let field_signature = signature(&mut class, b"Ljava/util/List<Lp/OnlySigned;>;");
    class.field(
        ACCESS_PRIVATE,
        b"signed",
        b"Ljava/util/List;",
        &[attribute(b"Signature", field_signature)],
    );

    let exceptions = class_list(&mut class, &[b"p/OnlyThrown"]);
    class.method(
        ACCESS_PUBLIC,
        b"run",
        b"()V",
        &[
            attribute(b"Code", empty_code()),
            attribute(b"Exceptions", exceptions),
        ],
    );
    class.finish()
}

/// Hierarchy and member descriptors.
fn hierarchy_fixture() -> Built {
    let mut class = Class::new(b"p/Sub", Some(b"p/Base"));
    class.interface(b"p/One");
    class.interface(b"p/Two");
    class.field(ACCESS_PRIVATE, b"array", b"[Ljava/lang/String;", &[]);
    class.field(ACCESS_PRIVATE, b"primitive", b"I", &[]);
    class.field(ACCESS_PRIVATE, b"object", b"Lp/FieldType;", &[]);
    class.method(
        ACCESS_PUBLIC,
        b"mixed",
        b"(I[Lp/Param;Lp/Second;)Lp/Result;",
        &[attribute(b"Code", empty_code())],
    );
    class.method(
        ACCESS_PUBLIC,
        b"repeat",
        b"(Lp/Dup;Lp/Dup;)Lp/Dup;",
        &[attribute(b"Code", empty_code())],
    );
    class.finish()
}

/// Class, field, method and record-component generic signatures, plus a string
/// constant and a member name that only *look* like type names.
fn signature_fixture() -> Built {
    let mut class = Class::new(b"p/Generic", Some(b"p/Superclass"));
    class.major(61);
    let _ = class.string(b"p/StringOnly");

    let class_signature = signature(
        &mut class,
        b"<T:Lp/Bound;U::Lp/Second;>Lp/Super<*Ljava/util/List<Lp/Elem;>;>;Lp/Holder<+Lp/Wild;>.Entry;",
    );
    class.class_attributes(&[attribute(b"Signature", class_signature)]);

    let field_signature = signature(&mut class, b"Ljava/util/Map<Lp/Key;Lp/Value;>;");
    class.field(
        ACCESS_PRIVATE,
        b"map",
        b"Ljava/util/Map;",
        &[attribute(b"Signature", field_signature)],
    );
    class.field(ACCESS_PRIVATE, b"p/NameOnly", b"I", &[]);

    let method_signature = signature(
        &mut class,
        b"<X:Lp/MBound;>(Ljava/util/List<TX;>;[Lp/MArray;)Lp/MResult;",
    );
    class.method(
        ACCESS_PUBLIC,
        b"run",
        b"()V",
        &[
            attribute(b"Code", empty_code()),
            attribute(b"Signature", method_signature),
        ],
    );

    let component_signature = signature(&mut class, b"Ljava/util/List<Lp/Component;>;");
    let both_signature = signature(&mut class, b"Ljava/util/List<Lp/Both;>;");
    // A record that is legal *and* interesting: one component carries a nested
    // `Signature` (the case a wrong declared-length accounting rejects), another
    // carries a component attribute this consumer does not read, which must be skipped
    // as its declared bytes, and the last one carries both.
    let annotation_component = annotations(
        &mut class,
        &[(
            b"Lp/ComponentAnno;",
            vec![(b"value", Value::Str(b"p/ComponentValue".to_vec()))],
        )],
    );
    let unread_component = annotations(&mut class, &[(b"Lp/Unread;", Vec::new())]);
    let components = record(
        &mut class,
        &[
            (b"plain", b"Lp/PlainComponent;", Vec::new()),
            (
                b"generic",
                b"Ljava/util/List;",
                vec![attribute(b"Signature", component_signature)],
            ),
            (
                b"skipped",
                b"Lp/SkippedComponent;",
                vec![attribute(
                    b"RuntimeVisibleAnnotations",
                    annotation_component,
                )],
            ),
            (
                b"both",
                b"Ljava/util/List;",
                vec![
                    attribute(b"Signature", both_signature),
                    attribute(b"RuntimeInvisibleAnnotations", unread_component),
                ],
            ),
        ],
    );
    class.class_attributes(&[attribute(b"Record", components)]);
    class.finish()
}

/// Every annotation attribute kind this consumer reads.
fn annotation_fixture() -> Built {
    let mut class = Class::new(b"p/Annotated", Some(b"java/lang/Object"));

    let visible = annotations(
        &mut class,
        &[(
            b"Lp/Visible;",
            vec![
                (b"value", Value::Str(b"p/Utf8Only".to_vec())),
                (b"literal", Value::Class(b"Lp/Literal;".to_vec())),
                (
                    b"mode",
                    Value::Enum {
                        descriptor: b"Lp/Mode;".to_vec(),
                        constant: b"SLOW".to_vec(),
                    },
                ),
                (b"wide", Value::Long(-9)),
                (b"ratio", Value::Double(0x3ff0_0000_0000_0000)),
            ],
        )],
    );
    class.class_attributes(&[attribute(b"RuntimeVisibleAnnotations", visible)]);

    let invisible = annotations(
        &mut class,
        &[(
            b"Lp/Invisible;",
            vec![
                (b"ints", Value::Array(vec![Value::Int(3), Value::Int(4)])),
                (
                    b"nested",
                    Value::Anno {
                        descriptor: b"Lp/Nested;".to_vec(),
                        elements: vec![(b"flag", Value::Bool(false))],
                    },
                ),
            ],
        )],
    );
    class.class_attributes(&[attribute(b"RuntimeInvisibleAnnotations", invisible)]);

    let supertype = type_annotations(
        &mut class,
        &[TypeAnno {
            target_type: 0x10,
            target_info: u16::MAX.to_be_bytes().to_vec(),
            descriptor: b"Lp/SuperTypeAnno;",
            elements: vec![(b"value", Value::Str(b"p/TypeValue".to_vec()))],
        }],
    );
    class.class_attributes(&[attribute(b"RuntimeVisibleTypeAnnotations", supertype)]);

    let return_type = type_annotations(
        &mut class,
        &[TypeAnno {
            target_type: 0x14,
            target_info: Vec::new(),
            descriptor: b"Lp/ReturnTypeAnno;",
            elements: Vec::new(),
        }],
    );
    let parameters = parameter_annotations(
        &mut class,
        &[
            vec![(b"Lp/ParamAnno;", Vec::new())],
            vec![(b"Lp/SecondParamAnno;", Vec::new())],
        ],
    );
    let default = Value::Enum {
        descriptor: b"Lp/Mode;".to_vec(),
        constant: b"DEFAULT".to_vec(),
    }
    .encode(&mut class);
    class.method(
        ACCESS_PUBLIC,
        b"run",
        b"()V",
        &[
            attribute(b"Code", empty_code()),
            attribute(b"RuntimeVisibleTypeAnnotations", return_type),
            attribute(b"RuntimeVisibleParameterAnnotations", parameters),
            attribute(b"AnnotationDefault", default),
        ],
    );
    class.finish()
}

/// A `Code` attribute content: the declared body plus its nested attributes (JVMS 4.7.3).
///
/// The metadata consumer walks this structure without decoding the instructions, so a
/// fixture only has to place the bytes a reader walks; the body is written as instructions
/// a class file really holds, so a test can slice the spans out of the class bytes.
fn code_with_nested(class: &mut Class, code: &[u8], nested: &[Pending]) -> Vec<u8> {
    let mut out = Vec::new();
    be16(&mut out, 1); // max_stack
    be16(&mut out, 1); // max_locals
    be32(&mut out, u32::try_from(code.len()).unwrap());
    out.extend_from_slice(code);
    be16(&mut out, 0); // exception_table_length
    be16(&mut out, u16::try_from(nested.len()).unwrap());
    for pending in nested {
        let name_index = class.utf8(pending.name);
        be16(&mut out, name_index);
        be32(&mut out, u32::try_from(pending.content.len()).unwrap());
        out.extend_from_slice(&pending.content);
    }
    out
}

/// `Code` content with one nested attribute whose declared length is given by the test, so
/// a declaration can disagree with the bytes the class file holds.
fn code_with_declared_length(
    class: &mut Class,
    attribute_name: &[u8],
    declared_length: u32,
    content: &[u8],
) -> Vec<u8> {
    let name_index = class.utf8(attribute_name);
    let mut out = Vec::new();
    be16(&mut out, 0); // max_stack
    be16(&mut out, 0); // max_locals
    be32(&mut out, 0); // code_length
    be16(&mut out, 0); // exception_table_length
    be16(&mut out, 1); // attributes_count
    be16(&mut out, name_index);
    be32(&mut out, declared_length);
    out.extend_from_slice(content);
    out
}

/// A record whose annotations live only in its components' nested annotation attributes.
///
/// Neither annotated type has a `CONSTANT_Class` entry, no generated field or accessor
/// carries the annotation, and the fixture's components are the only positions that
/// mention them. `p/DescriptorOnly` is named by a component descriptor alone,
/// `p/OnlyInCustom` only by bytes inside a custom component attribute this consumer never
/// reads, and `p/SharedMarker` appears in both a component attribute and a flat
/// class-level annotation so one request can pin the order of the two positions.
fn record_annotation_fixture() -> Built {
    let mut class = Class::new(b"p/RecordOnly", Some(b"java/lang/Object"));
    class.major(61);
    let visible = annotations(
        &mut class,
        &[
            (
                b"Lp/RecordMarker;",
                vec![(b"value", Value::Str(b"p/RecordValue".to_vec()))],
            ),
            (b"Lp/SharedMarker;", Vec::new()),
        ],
    );
    let invisible = annotations(&mut class, &[(b"Lp/HiddenRecordMarker;", Vec::new())]);
    let component_type = type_annotations(
        &mut class,
        &[TypeAnno {
            target_type: 0x13,
            target_info: Vec::new(),
            descriptor: b"Lp/ComponentTypeMarker;",
            elements: Vec::new(),
        }],
    );
    let unread = annotations(&mut class, &[(b"Lp/OnlyInCustom;", Vec::new())]);
    let components = record(
        &mut class,
        &[
            (
                b"value",
                b"I",
                vec![attribute(b"RuntimeVisibleAnnotations", visible)],
            ),
            (
                b"count",
                b"I",
                vec![attribute(b"RuntimeInvisibleAnnotations", invisible)],
            ),
            (
                b"other",
                b"Ljava/lang/String;",
                vec![attribute(b"RuntimeVisibleTypeAnnotations", component_type)],
            ),
            (
                b"only",
                b"Lp/DescriptorOnly;",
                vec![attribute(b"p/ComponentCustom", unread)],
            ),
        ],
    );
    let shared_flat = annotations(&mut class, &[(b"Lp/SharedMarker;", Vec::new())]);
    class.class_attributes(&[
        attribute(b"Record", components),
        attribute(b"RuntimeVisibleAnnotations", shared_flat),
    ]);
    class.finish()
}

/// A class whose type annotations live only inside a method's `Code` attribute.
///
/// `p/CodeMarker` and `p/CatchMarker` have no `CONSTANT_Class` entry and no other
/// attribute mentions them, `p/OnlyInCustom` only appears inside a custom `Code`-nested
/// attribute, and `p/SharedMarker` appears in a flat class-level annotation and inside the
/// body, so one request can pin the order of the two positions.
fn code_annotation_fixture() -> Built {
    let mut class = Class::new(b"p/CodeOnly", Some(b"java/lang/Object"));
    let created = class.class_index(b"p/Created");
    let visible = type_annotations(
        &mut class,
        &[
            TypeAnno {
                // `new` target: `target_info` is the bytecode offset of the instruction.
                target_type: 0x44,
                target_info: 0u16.to_be_bytes().to_vec(),
                descriptor: b"Lp/CodeMarker;",
                elements: vec![(b"value", Value::Str(b"p/CodeValue".to_vec()))],
            },
            // The same annotation type as the flat class-level attribute below, so one
            // request can pin the order of the two positions.
            TypeAnno {
                target_type: 0x44,
                target_info: 0u16.to_be_bytes().to_vec(),
                descriptor: b"Lp/SharedMarker;",
                elements: Vec::new(),
            },
        ],
    );
    let invisible = type_annotations(
        &mut class,
        &[TypeAnno {
            // `catch` target: `target_info` is an exception-table index, not an offset.
            target_type: 0x42,
            target_info: 0u16.to_be_bytes().to_vec(),
            descriptor: b"Lp/CatchMarker;",
            elements: Vec::new(),
        }],
    );
    let unread = annotations(&mut class, &[(b"Lp/OnlyInCustom;", Vec::new())]);
    let mut code = vec![0xbb];
    be16(&mut code, created);
    code.push(0x57); // pop
    code.push(0xb1); // return
    let body = code_with_nested(
        &mut class,
        &code,
        &[
            attribute(b"RuntimeVisibleTypeAnnotations", visible),
            attribute(b"RuntimeInvisibleTypeAnnotations", invisible),
            attribute(b"p/CodeCustom", unread),
        ],
    );
    class.method(ACCESS_PUBLIC, b"call", b"()V", &[attribute(b"Code", body)]);
    let shared_flat = annotations(&mut class, &[(b"Lp/SharedMarker;", Vec::new())]);
    class.class_attributes(&[attribute(b"RuntimeVisibleAnnotations", shared_flat)]);
    class.finish()
}

/// A class whose `Code` attribute declares a nested attribute longer than the bytes that
/// follow it, with no other attribute to read.
fn broken_code_structure_fixture() -> Built {
    let mut class = Class::new(b"p/BrokenCode", Some(b"java/lang/Object"));
    // The nested declaration claims four bytes and the class file holds two.
    let body = code_with_declared_length(
        &mut class,
        b"RuntimeVisibleTypeAnnotations",
        4,
        &[0x00, 0x00],
    );
    class.method(ACCESS_PUBLIC, b"call", b"()V", &[attribute(b"Code", body)]);
    class.finish()
}

/// The annotation type descriptor the nested-tail fixtures query.
const TAIL_MARKER: &[u8] = b"Lp/TailMarker;";

/// What one fixture's nested annotation attribute holds.
#[derive(Clone, Copy, Eq, PartialEq)]
enum NestedContent {
    /// One structure for the queried type whose single element value tag is `0x00`, a byte
    /// no `element_value` has.
    Malformed,
    /// The same structure with the string tag this consumer reads.
    Valid,
    /// Two complete structures for the queried type, followed by the malformed one.
    ValidThenMalformed,
}

/// Which nested position a fixture puts its annotation attribute in.
#[derive(Clone, Copy)]
enum NestedPosition {
    /// A type annotation inside a method's `Code` attribute.
    Code,
    /// An annotation inside a `Record` component.
    Record,
}

/// An annotation list: the declared count followed by the structures.
fn annotation_list(structures: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    be16(&mut out, u16::try_from(structures.len()).unwrap());
    for structure in structures {
        out.extend_from_slice(structure);
    }
    out
}

/// A fixture whose nested annotation attribute holds one structure for `p/TailMarker`.
///
/// The structure declares one element pair, and the fixtures differ in **one byte**: the
/// value tag of that pair. The annotation type descriptor is read before the element values,
/// so [`NestedContent::Malformed`] produces the fact for the queried symbol and then fails
/// inside the same structure, while [`NestedContent::Valid`] reads the pair and ends
/// normally. Both fixtures describe the same structure at the same position, so the
/// difference between their results is the failure itself.
fn nested_tail_fixture(position: NestedPosition, content: NestedContent) -> Built {
    let mut class = Class::new(b"p/Tail", Some(b"java/lang/Object"));
    let marker = class.utf8(TAIL_MARKER);
    let element = class.utf8(b"value");
    let value = class.utf8(b"p/TailValue");

    let structure_for = |tag: u8| {
        let mut structure = Vec::new();
        if let NestedPosition::Code = position {
            // A `new` target: the annotated instruction's offset, then an empty type path.
            structure.push(0x44);
            structure.extend_from_slice(&0u16.to_be_bytes());
            structure.push(0);
        }
        structure.extend_from_slice(&marker.to_be_bytes());
        be16(&mut structure, 1); // one element pair
        structure.extend_from_slice(&element.to_be_bytes());
        structure.push(tag);
        structure.extend_from_slice(&value.to_be_bytes());
        structure
    };
    let structures = match content {
        NestedContent::Malformed => vec![structure_for(0x00)],
        NestedContent::Valid => vec![structure_for(b's')],
        NestedContent::ValidThenMalformed => vec![
            structure_for(b's'),
            structure_for(b's'),
            structure_for(0x00),
        ],
    };

    let attribute_name: &'static [u8] = match position {
        NestedPosition::Code => b"RuntimeVisibleTypeAnnotations",
        NestedPosition::Record => b"RuntimeVisibleAnnotations",
    };
    let content = annotation_list(&structures);
    match position {
        NestedPosition::Code => {
            let code = vec![0x57, 0xb1]; // pop; return
            let body = code_with_nested(&mut class, &code, &[attribute(attribute_name, content)]);
            class.method(ACCESS_PUBLIC, b"call", b"()V", &[attribute(b"Code", body)]);
        }
        NestedPosition::Record => {
            class.major(61);
            let components = record(
                &mut class,
                &[(
                    b"value",
                    b"Lp/TailValue;",
                    vec![attribute(attribute_name, content)],
                )],
            );
            class.class_attributes(&[attribute(b"Record", components)]);
        }
    }
    class.finish()
}

/// A class whose only `Signature` attribute is the class-level one.
fn class_signature_fixture(signature_bytes: &[u8]) -> Built {
    let mut class = Class::new(b"p/ClassSignature", Some(b"p/Super"));
    let signature = signature(&mut class, signature_bytes);
    class.class_attributes(&[attribute(b"Signature", signature)]);
    class.finish()
}

/// Inner/nest relations, including an anonymous `InnerClasses` entry.
fn inner_nest_fixture() -> Built {
    let mut class = Class::new(b"p/Outer$1", Some(b"p/Outer"));
    let inner = inner_classes(
        &mut class,
        &[
            (b"p/Outer$1", Some(b"p/Outer"), None),
            (b"p/Outer$Inner", Some(b"p/Outer"), Some(b"Inner")),
        ],
    );
    let enclosing = enclosing_method(&mut class, b"p/Outer", b"run", b"()V");
    let host = nest_host(&mut class, b"p/Outer");
    let members = class_list(&mut class, &[b"p/Outer$Inner", b"p/Outer$2"]);
    let permitted = class_list(&mut class, &[b"p/Impl", b"p/Impl2"]);
    class.class_attributes(&[
        attribute(b"InnerClasses", inner),
        attribute(b"EnclosingMethod", enclosing),
        attribute(b"NestHost", host),
        attribute(b"NestMembers", members),
        attribute(b"PermittedSubclasses", permitted),
    ]);
    class.finish()
}

/// Field `ConstantValue` attributes, with one value recorded twice.
fn constant_fixture() -> (Built, ConstantIndexes) {
    let mut class = Class::new(b"p/Constants", Some(b"java/lang/Object"));
    let number = class.integer(7);
    class.field(
        ACCESS_PRIVATE,
        b"first",
        b"I",
        &[attribute(b"ConstantValue", number.to_be_bytes().to_vec())],
    );
    class.field(
        ACCESS_PRIVATE,
        b"second",
        b"I",
        &[attribute(b"ConstantValue", number.to_be_bytes().to_vec())],
    );
    let text = class.string(b"abc");
    class.field(
        ACCESS_PRIVATE,
        b"text",
        b"Ljava/lang/String;",
        &[attribute(b"ConstantValue", text.to_be_bytes().to_vec())],
    );
    let wide = class.long(-9);
    class.field(
        ACCESS_PRIVATE,
        b"wide",
        b"J",
        &[attribute(b"ConstantValue", wide.to_be_bytes().to_vec())],
    );
    (class.finish(), ConstantIndexes { number, text, wide })
}

/// Constant-pool entries of [`constant_fixture`].
struct ConstantIndexes {
    number: u16,
    text: u16,
    wide: u16,
}

/// A `module-info` class and a regular class that carries the same `Module`
/// attribute, so the `this_class` gate is directly observable.
fn module_fixture() -> Built {
    let mut class = Class::new(b"module-info", None);
    class.major(53);
    class.access_flags(ACCESS_MODULE);
    let module = module_attribute(
        &mut class,
        b"p.module",
        &[b"p/Service"],
        &[(b"p/Api", &[b"p/Impl", b"p/Impl2"])],
    );
    class.class_attributes(&[attribute(b"Module", module)]);
    class.finish()
}

fn not_module_fixture() -> Built {
    let mut class = Class::new(b"p/NotModule", Some(b"java/lang/Object"));
    let module = module_attribute(
        &mut class,
        b"p.module",
        &[b"p/Service"],
        &[(b"p/Api", &[b"p/Impl"])],
    );
    class.class_attributes(&[attribute(b"Module", module)]);
    class.finish()
}

/// A signature-only type plus an attribute no consumer knows.
fn unknown_attribute_fixture() -> Built {
    let mut class = Class::new(b"p/Unknowns", Some(b"java/lang/Object"));
    let field_signature = signature(&mut class, b"Lp/CustomSigned;");
    class.field(
        ACCESS_PRIVATE,
        b"value",
        b"I",
        &[attribute(b"Signature", field_signature)],
    );
    class.class_attributes(&[attribute(
        b"XxCustom",
        vec![0xde, 0xad, 0xbe, 0xef, 0x00, 0xff, 0x7f],
    )]);
    class.finish()
}

/// A `Signature` attribute whose string does not parse.
fn malformed_signature_fixture() -> Built {
    let mut class = Class::new(b"p/Broken", Some(b"java/lang/Object"));
    let field_signature = signature(&mut class, b"Lp/Unterminated");
    class.class_attributes(&[attribute(b"Signature", field_signature)]);
    class.finish()
}

/// A pool that spells the same bytes twice, the way a non-interning writer emits it.
///
/// A pool is interned only by convention, and `javac` does dedupe equal entries, but
/// nothing in the class-file format requires it. Then "the constant-pool entry this fact
/// came from" is not identifiable from the bytes alone, and a scan that guesses an index
/// would point at an entry the declaration may not have used.
fn ambiguous_pool_fixture() -> Built {
    let mut class = Class::new(b"p/Ambiguous", Some(b"p/Super"));
    class.duplicate_class(b"p/Super");
    // A name with a single pool entry, so the degradation is observable against a control.
    class.interface(b"p/Kept");
    // Two `CONSTANT_Utf8` entries for this field's descriptor.
    class.field(ACCESS_PRIVATE, b"value", b"Lp/Field;", &[]);
    class.duplicate_utf8(b"Lp/Field;");
    // Two `CONSTANT_Utf8` entries for this field's signature string.
    let field_signature = signature(&mut class, b"Lp/Signed;");
    class.field(
        ACCESS_PRIVATE,
        b"signed",
        b"Lp/SignedField;",
        &[attribute(b"Signature", field_signature)],
    );
    class.duplicate_utf8(b"Lp/Signed;");
    // Two `CONSTANT_Class` entries for this method's exception type.
    let exceptions = class_list(&mut class, &[b"p/Thrown"]);
    class.method(
        ACCESS_PUBLIC,
        b"run",
        b"()V",
        &[
            attribute(b"Code", empty_code()),
            attribute(b"Exceptions", exceptions),
        ],
    );
    class.duplicate_class(b"p/Thrown");
    class.finish()
}

/// A field descriptor that never terminates.
fn malformed_descriptor_fixture() -> Built {
    let mut class = Class::new(b"p/BadDescriptor", Some(b"java/lang/Object"));
    class.field(ACCESS_PRIVATE, b"value", b"Ljava/lang/String", &[]);
    class.finish()
}

/// An annotations attribute with an unknown element value tag.
fn malformed_annotation_fixture() -> Built {
    let mut class = Class::new(b"p/BadAnnotation", Some(b"java/lang/Object"));
    let mut content = Vec::new();
    be16(&mut content, 1);
    let descriptor = class.utf8(b"Lp/A;");
    be16(&mut content, descriptor);
    be16(&mut content, 1);
    let element = class.utf8(b"x");
    be16(&mut content, element);
    content.push(b'z');
    class.class_attributes(&[attribute(b"RuntimeVisibleAnnotations", content)]);
    class.finish()
}

/// A `Record` attribute that declares components without any component data.
fn malformed_record_fixture() -> Built {
    let mut class = Class::new(b"p/BadRecord", Some(b"java/lang/Object"));
    class.class_attributes(&[attribute(b"Record", vec![0x00, 0x02])]);
    class.finish()
}

/// A `ConstantValue` attribute that references a `CONSTANT_Utf8` entry.
fn malformed_constant_fixture() -> Built {
    let mut class = Class::new(b"p/BadConstant", Some(b"java/lang/Object"));
    let index = class.utf8(b"I");
    class.field(
        ACCESS_PRIVATE,
        b"value",
        b"I",
        &[attribute(b"ConstantValue", index.to_be_bytes().to_vec())],
    );
    class.finish()
}

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 10_000,
        output_bytes: 1 << 20,
        nested_depth: 4,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("fixture must open")
}

fn target_symbol(owner: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(owner.as_bytes().to_vec()),
        },
    }
}

fn target_method(owner: &str, name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(owner.as_bytes().to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

fn target_string(value: &str) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::String {
            value: JvmBytes(value.as_bytes().to_vec()),
        },
    }
}

fn target_integer(value: i32) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::Integer { value },
    }
}

fn target_class(descriptor: &str) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::Class {
            value: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

fn query_request(
    snapshot: &ArtifactSnapshot,
    relation: QueryRelation,
    target: QueryTarget,
    kinds: &[ConsumerKind],
) -> QueryRequest {
    QueryRequest {
        relation,
        target,
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, kinds.to_vec()),
        max_items: 0,
        cursor: None,
    }
}

fn query_with(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
    limits: Limits,
) -> (QueryReport, UsageSnapshot) {
    let mut budget = Budget::new(limits);
    let report = Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("query must succeed");
    (report, budget.usage())
}

fn run(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> QueryReport {
    query_with(snapshot, request, limits()).0
}

/// Asserts the scan ran to completion and explained nothing.
///
/// An item sequence alone cannot show that a unit was scanned to its end: published
/// items stay published even when the scan stops in the middle of the next structure,
/// so a positive case that only checks item shape passes on a partial or failed scan.
/// Every case that expects facts therefore states its execution result.
fn assert_complete(report: &QueryReport) {
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "a positive case must complete, got {:?} with diagnostics {:?}",
        report.execution,
        report.diagnostics
    );
    assert!(
        report.diagnostics.is_empty(),
        "a completed scan explains nothing, got {:?}",
        report.diagnostics
    );
}

/// Runs one positive query, which must be a completed scan.
fn run_complete(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> QueryReport {
    let report = run(snapshot, request);
    assert_complete(&report);
    report
}

fn operations(report: &QueryReport) -> Vec<XrefOperation> {
    report.items.iter().map(|item| item.operation).collect()
}

fn attributes(report: &QueryReport) -> Vec<Option<Vec<u8>>> {
    report
        .items
        .iter()
        .map(|item| item.evidence.attribute.as_ref().map(|name| name.0.clone()))
        .collect()
}

fn owners(report: &QueryReport) -> Vec<Vec<u8>> {
    report
        .items
        .iter()
        .map(|item| match &item.target {
            XrefTarget::Symbol {
                value: SymbolRef::Class { owner },
            } => owner.0.clone(),
            other => panic!("expected a class symbol target, got {other:?}"),
        })
        .collect()
}

fn class_offset(item: &XrefItem) -> (&PhysicalDefinitionId, u64) {
    match &item.source.location {
        Location::ClassOffset { definition, offset } => (definition, *offset),
        other => panic!("expected a class-offset provenance, got {other:?}"),
    }
}

/// Asserts the exact evidence of one class-symbol item read through `index`.
fn assert_pool_evidence(item: &XrefItem, built: &Built, index: u16, attribute: Option<&[u8]>) {
    assert_eq!(item.evidence.constant_pool_index, Some(index));
    assert_eq!(item.evidence.span, Some(built.entry_span(index)));
    let (definition, offset) = class_offset(item);
    assert!(matches!(
        definition.location,
        PhysicalClassLocation::StandaloneRoot { .. }
    ));
    assert_eq!(definition.variant, PhysicalVariant::Base);
    assert_eq!(definition.class_bytes.length, built.bytes.len() as u64);
    assert_eq!(offset, built.entry_span(index).start);
    assert_eq!(
        item.evidence.attribute,
        attribute.map(|name| ArchiveNameBytes(name.to_vec()))
    );
}

/// Bytes the fixture holds at one class-file range.
fn span_bytes<'a>(built: &'a Built, span: &ByteSpan) -> &'a [u8] {
    let start = usize::try_from(span.start).expect("offset fits usize");
    let end = start + usize::try_from(span.length).expect("length fits usize");
    &built.bytes[start..end]
}

/// Path and range of one item read at a nested attribute position.
fn attribute_span(item: &XrefItem) -> (&str, &ByteSpan) {
    match &item.source.location {
        Location::Attribute { path, span, .. } => (path, span),
        other => panic!("expected a nested attribute location, got {other:?}"),
    }
}

/// Absolute class-file range of every nested attribute inside the `Record` content, in
/// declaration order.
///
/// The fixture writer lays the declarations out, and this reads them back so a test can
/// state the range a nested fact must report instead of assuming one.
fn record_nested_attribute_spans(built: &Built) -> Vec<ByteSpan> {
    let content_span = built.attribute_content_span(b"Record");
    let content = built.attribute_content(b"Record");
    let u16_at = |at: &mut usize| {
        let value = u16::from_be_bytes([content[*at], content[*at + 1]]);
        *at += 2;
        value
    };
    let u32_at = |at: &mut usize| {
        let value = u32::from_be_bytes([
            content[*at],
            content[*at + 1],
            content[*at + 2],
            content[*at + 3],
        ]);
        *at += 4;
        value
    };
    let mut at = 0_usize;
    let components = u16_at(&mut at);
    let mut spans = Vec::new();
    for _ in 0..components {
        let _name_index = u16_at(&mut at);
        let _descriptor_index = u16_at(&mut at);
        let attributes = u16_at(&mut at);
        for _ in 0..attributes {
            let _name_index = u16_at(&mut at);
            let length = u32_at(&mut at);
            spans.push(ByteSpan::new(
                content_span.start + at as u64,
                u64::from(length),
            ));
            at += usize::try_from(length).expect("declared length fits usize");
        }
    }
    assert_eq!(at, content.len(), "the declarations must cover the content");
    spans
}

// ---------------------------------------------------------------------------
// A03: metadata-only type references
// ---------------------------------------------------------------------------

#[test]
fn metadata_only_types_hit_only_their_requested_category() {
    let built = meta_fixture();
    let snapshot = open(built.bytes.clone());

    // A03: neither type has a `CONSTANT_Class` entry of its own, and the annotation's
    // string element value is a `CONSTANT_Utf8` value rather than a
    // `CONSTANT_String` entry.
    assert!(!built.has_class(b"p/OnlyAnnotated"));
    assert!(!built.has_class(b"p/OnlySigned"));
    assert_eq!(built.string_constant(b"p/OnlySigned"), None);

    // Annotation descriptor: the annotation category answers it, with the raw
    // descriptor's pool entry as its location.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyAnnotated"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Annotation));
    assert_eq!(item.derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert_eq!(item.relation, QueryRelation::MentionsSymbol);
    assert_eq!(item.evidence.bci, None);
    assert_eq!(item.evidence.opcode, None);
    assert!(item.evidence.via.is_empty());
    assert_eq!(owners(&report), vec![b"p/OnlyAnnotated".to_vec()]);
    assert_pool_evidence(
        item,
        &built,
        built.utf8_index(b"Lp/OnlyAnnotated;"),
        Some(b"RuntimeVisibleAnnotations"),
    );

    // The same fixture reports nothing for the requested type under any other
    // category: a metadata-only type is not a constant-pool candidate.
    for kind in [
        ConsumerKind::Type,
        ConsumerKind::Signature,
        ConsumerKind::Exception,
        ConsumerKind::InnerNest,
        ConsumerKind::Constant,
        ConsumerKind::Module,
    ] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("p/OnlyAnnotated"),
                &[kind],
            ),
        );
        assert!(
            report.items.is_empty(),
            "{kind:?} must not answer an annotation-only type"
        );
    }

    // A signature-only type: the annotation records the same bytes as a *string*
    // element value, which is a value and never a type reference.
    for kind in [
        ConsumerKind::Annotation,
        ConsumerKind::Type,
        ConsumerKind::Exception,
        ConsumerKind::InnerNest,
    ] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("p/OnlySigned"),
                &[kind],
            ),
        );
        assert!(
            report.items.is_empty(),
            "{kind:?} must not answer a signature-only type"
        );
    }
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlySigned"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::GenericSignature]);
    assert_eq!(report.items[0].consumer, Some(ConsumerKind::Signature));
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.utf8_index(b"Ljava/util/List<Lp/OnlySigned;>;"),
        Some(b"Signature"),
    );

    // An exceptions-only type.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyThrown"),
            &[ConsumerKind::Exception],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::ExceptionsAttribute]
    );
    assert_eq!(report.items[0].consumer, Some(ConsumerKind::Exception));
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.class_index(b"p/OnlyThrown"),
        Some(b"Exceptions"),
    );
    for kind in [
        ConsumerKind::Type,
        ConsumerKind::Signature,
        ConsumerKind::InnerNest,
    ] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("p/OnlyThrown"),
                &[kind],
            ),
        );
        assert!(report.items.is_empty(), "{kind:?} must not answer it");
    }

    // Enum types, nested annotation types and class literal descriptors are type
    // references of the annotation category as well.
    for name in ["p/Mode", "p/Nested", "p/Literal"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Annotation],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::Annotation],
            "{name}"
        );
    }

    // Annotation element values are values of the annotation category, including the
    // string that spells another type name, the enum constant and the class literal.
    let cases: Vec<(QueryTarget, LiteralValue)> = vec![
        (
            target_string("p/OnlySigned"),
            LiteralValue::String {
                value: JvmBytes(b"p/OnlySigned".to_vec()),
            },
        ),
        (target_integer(7), LiteralValue::Integer { value: 7 }),
        (
            target_string("FAST"),
            LiteralValue::String {
                value: JvmBytes(b"FAST".to_vec()),
            },
        ),
        (
            target_class("Lp/Literal;"),
            LiteralValue::Class {
                value: JvmBytes(b"Lp/Literal;".to_vec()),
            },
        ),
        (target_integer(2), LiteralValue::Integer { value: 2 }),
    ];
    for (target, value) in cases {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::LiteralValue,
                target,
                &[ConsumerKind::Annotation],
            ),
        );
        assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
        assert_eq!(report.items[0].consumer, Some(ConsumerKind::Annotation));
        assert_eq!(report.items[0].relation, QueryRelation::LiteralValue);
        assert_eq!(
            report.items[0].target,
            XrefTarget::Literal { value },
            "literal values keep their JVM shape"
        );
        assert_eq!(
            report.items[0].evidence.attribute,
            Some(ArchiveNameBytes(b"RuntimeVisibleAnnotations".to_vec()))
        );
    }

    // A category that can only record symbols is not scanned for a value request.
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::LiteralValue,
            target_string("p/OnlySigned"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    assert_complete(&report);
    assert_eq!(usage.class_bytes, 0);
    assert_eq!(usage.attribute_bytes, 0);
    assert_eq!(report.coverage.scanned_items, 0);
}

// ---------------------------------------------------------------------------
// Hierarchy and descriptors
// ---------------------------------------------------------------------------

#[test]
fn this_class_is_not_a_self_use_but_super_and_interfaces_are() {
    let built = hierarchy_fixture();
    let snapshot = open(built.bytes.clone());

    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Sub"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(
        report.items.is_empty(),
        "a class is not a user of its own definition identity"
    );

    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Base"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::SuperClass]);
    assert_eq!(report.items[0].consumer, Some(ConsumerKind::Type));
    assert_pool_evidence(&report.items[0], &built, built.class_index(b"p/Base"), None);

    for name in ["p/One", "p/Two"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Type],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::Interface],
            "{name}"
        );
        assert_pool_evidence(
            &report.items[0],
            &built,
            built.class_index(name.as_bytes()),
            None,
        );
    }
}

#[test]
fn member_descriptors_report_each_object_type_once() {
    let built = hierarchy_fixture();
    let snapshot = open(built.bytes.clone());

    // An array descriptor names its element type, not the array class.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("java/lang/String"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::FieldDescriptor]);
    assert_eq!(owners(&report), vec![b"java/lang/String".to_vec()]);
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.utf8_index(b"[Ljava/lang/String;"),
        None,
    );
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("[Ljava/lang/String;"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(
        report.items.is_empty(),
        "the array class name is not reported as an object type"
    );

    // A primitive type is not a reference.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("I"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty());

    // A multi-parameter method reports each object type once.
    for name in ["p/Param", "p/Second", "p/Result"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Type],
            ),
        );
        assert_eq!(report.items.len(), 1, "{name}");
        assert_eq!(operations(&report), vec![XrefOperation::MethodDescriptor]);
        assert_pool_evidence(
            &report.items[0],
            &built,
            built.utf8_index(b"(I[Lp/Param;Lp/Second;)Lp/Result;"),
            None,
        );
    }

    // A type repeated inside one descriptor is reported once.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Dup"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(report.items.len(), 1);
    assert_eq!(owners(&report), vec![b"p/Dup".to_vec()]);

    // A field descriptor keeps the field operation and the raw owner bytes.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/FieldType"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::FieldDescriptor]);
    assert_eq!(owners(&report), vec![b"p/FieldType".to_vec()]);
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.utf8_index(b"Lp/FieldType;"),
        None,
    );

    // No descriptor fact is produced without the type category.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/FieldType"),
            &[ConsumerKind::Signature],
        ),
    );
    assert!(report.items.is_empty());
}

// ---------------------------------------------------------------------------
// Generic signatures and record components
// ---------------------------------------------------------------------------

#[test]
fn generic_signatures_follow_their_own_grammar() {
    let built = signature_fixture();
    let snapshot = open(built.bytes.clone());

    // Class signature: type-variable bounds, a nested type argument, an unbounded
    // wildcard bound, a bounded wildcard and a suffix-form nested class.
    for name in [
        "p/Bound",
        "p/Second",
        "p/Super",
        "p/Elem",
        "p/Holder",
        "p/Wild",
        "p/Holder$Entry",
    ] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::GenericSignature],
            "{name}"
        );
        assert_eq!(report.items[0].consumer, Some(ConsumerKind::Signature));
        assert_pool_evidence(
            &report.items[0],
            &built,
            built.utf8_index(
                b"<T:Lp/Bound;U::Lp/Second;>Lp/Super<*Ljava/util/List<Lp/Elem;>;>;Lp/Holder<+Lp/Wild;>.Entry;",
            ),
            Some(b"Signature"),
        );
        assert_eq!(owners(&report), vec![name.as_bytes().to_vec()]);
    }

    // The same type can be named by several signatures: the class signature, the
    // method signature, and for both generic record components their *descriptor* and
    // their own `Signature` attribute. A legal generic record is what a wrong
    // declared-length accounting rejects, so this case is also the B1 regression
    // (see `record_component_signatures_survive_their_declared_length`).
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("java/util/List"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![
            XrefOperation::GenericSignature,
            XrefOperation::GenericSignature,
            XrefOperation::RecordComponent,
            XrefOperation::RecordComponent,
            XrefOperation::RecordComponent,
            XrefOperation::RecordComponent,
        ]
    );
    assert_eq!(
        attributes(&report),
        vec![
            Some(b"Signature".to_vec()),
            Some(b"Signature".to_vec()),
            Some(b"Record".to_vec()),
            Some(b"Signature".to_vec()),
            Some(b"Record".to_vec()),
            Some(b"Signature".to_vec()),
        ]
    );

    // Field and method signatures.
    for name in ["java/util/Map", "p/Key", "p/Value"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::GenericSignature],
            "{name}"
        );
        assert_pool_evidence(
            &report.items[0],
            &built,
            built.utf8_index(b"Ljava/util/Map<Lp/Key;Lp/Value;>;"),
            Some(b"Signature"),
        );
    }
    for name in ["p/MBound", "p/MArray", "p/MResult"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::GenericSignature],
            "{name}"
        );
        assert_pool_evidence(
            &report.items[0],
            &built,
            built.utf8_index(b"<X:Lp/MBound;>(Ljava/util/List<TX;>;[Lp/MArray;)Lp/MResult;"),
            Some(b"Signature"),
        );
    }

    // A type variable is not a class reference, and neither is a string constant or a
    // member name that happens to spell one.
    for name in ["T", "X", "TX", "p/StringOnly", "p/NameOnly"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature, ConsumerKind::Type],
            ),
        );
        assert!(report.items.is_empty(), "{name} is not a type reference");
    }

    // Signatures are parsed, not matched: the descriptor spelling, the simple name and
    // the source form of a nested class are not the internal name the signature names.
    for name in ["Lp/Bound;", "Bound", "p/Holder.Entry", "Outer$Inner"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature],
            ),
        );
        assert!(
            report.items.is_empty(),
            "{name} is not how a signature names a type"
        );
    }

    // Record components: the recorded descriptor and the nested `Signature`.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/PlainComponent"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::RecordComponent]);
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.utf8_index(b"Lp/PlainComponent;"),
        Some(b"Record"),
    );
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Component"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::RecordComponent]);
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.utf8_index(b"Ljava/util/List<Lp/Component;>;"),
        Some(b"Signature"),
    );

    // A component attribute this consumer does not read is skipped as its declared
    // bytes, including when the same component also carries a `Signature`.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/SkippedComponent"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::RecordComponent]);
    assert_eq!(attributes(&report), vec![Some(b"Record".to_vec())]);
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Both"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::RecordComponent]);
    assert_eq!(attributes(&report), vec![Some(b"Signature".to_vec())]);
    // The unread component annotation attributes contribute no fact of this category.
    for name in ["p/ComponentValue", "p/ComponentAnno", "p/Unread"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature],
            ),
        );
        assert!(report.items.is_empty(), "{name}");
    }

    // Signatures are only read for the signature category.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Bound"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty());
}

/// One expected record component fact: target type, raw owner bytes, the attribute
/// that recorded it and the pool value it was read through.
struct ComponentFact<'a> {
    target: &'a str,
    owner: &'a [u8],
    attribute: &'a [u8],
    pool_value: &'a [u8],
}

/// The B1 regression: a legal generic record must be read to its end.
///
/// A component `Signature` declares one constant-pool index, and the rejected
/// accounting compared that declaration against a magic four bytes, so every generic
/// record was reported as malformed and its unit stopped. This test pins the accounting:
/// the scan completes, every record component type is still reported, and the fixture's
/// nested `Signature` declarations really are two content bytes.
#[test]
fn record_component_signatures_survive_their_declared_length() {
    let built = signature_fixture();
    let snapshot = open(built.bytes.clone());

    let components = [
        ComponentFact {
            target: "p/PlainComponent",
            owner: b"p/PlainComponent",
            attribute: b"Record",
            pool_value: b"Lp/PlainComponent;",
        },
        ComponentFact {
            target: "p/Component",
            owner: b"p/Component",
            attribute: b"Signature",
            pool_value: b"Ljava/util/List<Lp/Component;>;",
        },
        ComponentFact {
            target: "p/SkippedComponent",
            owner: b"p/SkippedComponent",
            attribute: b"Record",
            pool_value: b"Lp/SkippedComponent;",
        },
        ComponentFact {
            target: "p/Both",
            owner: b"p/Both",
            attribute: b"Signature",
            pool_value: b"Ljava/util/List<Lp/Both;>;",
        },
    ];
    for fact in components {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(fact.target),
                &[ConsumerKind::Signature],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::RecordComponent],
            "{}",
            fact.target
        );
        assert_eq!(
            owners(&report),
            vec![fact.owner.to_vec()],
            "{}",
            fact.target
        );
        assert_pool_evidence(
            &report.items[0],
            &built,
            built.utf8_index(fact.pool_value),
            Some(fact.attribute),
        );
    }

    // The fixture really is the case under test: every nested component `Signature`
    // declares the two bytes of one constant-pool index, not four.
    let record = built.attribute_content(b"Record");
    let declarations = record_component_attributes(&built, &record);
    let signature_lengths: Vec<u32> = declarations
        .iter()
        .filter(|(name, _)| name.as_slice() == b"Signature")
        .map(|(_, length)| *length)
        .collect();
    assert_eq!(signature_lengths, vec![2, 2]);
    assert!(
        declarations.iter().all(|(_, length)| *length > 0),
        "every component attribute declares some content: {declarations:?}"
    );
}

#[test]
fn record_component_declaration_must_agree_with_its_content() {
    // A component `Signature` that declares one byte cannot hold its one index, so the
    // declaration and the content disagree: structured error, and no fact from that
    // attribute.
    let mut class = Class::new(b"p/ShortRecord", Some(b"java/lang/Object"));
    let signature_index = class.utf8(b"Lp/Short;");
    let record = record_with_declared_length(
        &mut class,
        b"value",
        b"Lp/ShortField;",
        b"Signature",
        1,
        &signature_index.to_be_bytes(),
    );
    class.class_attributes(&[attribute(b"Record", record)]);
    let built = class.finish();
    let snapshot = open(built.bytes.clone());

    let (report, _) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Short"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert!(
        report.items.is_empty(),
        "a rejected declaration contributes no fact"
    );
    match &report.execution {
        ExecutionReport::Failed {
            reason: TerminationReason::Error { code },
            ..
        } => assert_eq!(code, "query_record_malformed"),
        other => panic!("a contradicting declaration must stop the scan, got {other:?}"),
    }

    // The component descriptor is a separate, verified fact of the same record.
    let (report, _) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/ShortField"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert_eq!(operations(&report), vec![XrefOperation::RecordComponent]);
    assert_eq!(attributes(&report), vec![Some(b"Record".to_vec())]);
}

// ---------------------------------------------------------------------------
// Annotations
// ---------------------------------------------------------------------------

#[test]
fn annotation_attributes_cover_visible_invisible_type_parameter_and_default() {
    let built = annotation_fixture();
    let snapshot = open(built.bytes.clone());

    let cases: Vec<(&str, XrefOperation, &[u8])> = vec![
        (
            "p/Visible",
            XrefOperation::Annotation,
            b"RuntimeVisibleAnnotations",
        ),
        (
            "p/Literal",
            XrefOperation::Annotation,
            b"RuntimeVisibleAnnotations",
        ),
        (
            "p/Invisible",
            XrefOperation::Annotation,
            b"RuntimeInvisibleAnnotations",
        ),
        (
            "p/Nested",
            XrefOperation::Annotation,
            b"RuntimeInvisibleAnnotations",
        ),
        (
            "p/SuperTypeAnno",
            XrefOperation::TypeAnnotation,
            b"RuntimeVisibleTypeAnnotations",
        ),
        (
            "p/ReturnTypeAnno",
            XrefOperation::TypeAnnotation,
            b"RuntimeVisibleTypeAnnotations",
        ),
        (
            "p/ParamAnno",
            XrefOperation::Annotation,
            b"RuntimeVisibleParameterAnnotations",
        ),
        (
            "p/SecondParamAnno",
            XrefOperation::Annotation,
            b"RuntimeVisibleParameterAnnotations",
        ),
    ];
    for (name, operation, attribute) in cases {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Annotation],
            ),
        );
        assert_eq!(operations(&report), vec![operation], "{name}");
        assert_eq!(
            attributes(&report),
            vec![Some(attribute.to_vec())],
            "{name} keeps the raw attribute name"
        );
        assert_eq!(report.items[0].consumer, Some(ConsumerKind::Annotation));
    }

    // The enum type of a visible element value and of the default value are two
    // distinct structural facts.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Mode"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::Annotation, XrefOperation::AnnotationDefault]
    );
    assert_eq!(
        attributes(&report),
        vec![
            Some(b"RuntimeVisibleAnnotations".to_vec()),
            Some(b"AnnotationDefault".to_vec()),
        ]
    );

    // A string element value is a `CONSTANT_Utf8` value: the fixture deliberately has
    // no `CONSTANT_String` entry for it.
    assert_eq!(built.string_constant(b"p/Utf8Only"), None);
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::LiteralValue,
            target_string("p/Utf8Only"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    assert_eq!(
        attributes(&report),
        vec![Some(b"RuntimeVisibleAnnotations".to_vec())]
    );

    // Enum constants, primitive values, long/double bit patterns and class literals
    // are values of the annotation category.
    let cases: Vec<(QueryTarget, LiteralValue, &[u8], XrefOperation)> = vec![
        (
            target_string("SLOW"),
            LiteralValue::String {
                value: JvmBytes(b"SLOW".to_vec()),
            },
            b"RuntimeVisibleAnnotations",
            XrefOperation::Annotation,
        ),
        (
            target_string("DEFAULT"),
            LiteralValue::String {
                value: JvmBytes(b"DEFAULT".to_vec()),
            },
            b"AnnotationDefault",
            XrefOperation::AnnotationDefault,
        ),
        (
            target_integer(3),
            LiteralValue::Integer { value: 3 },
            b"RuntimeInvisibleAnnotations",
            XrefOperation::Annotation,
        ),
        (
            target_integer(4),
            LiteralValue::Integer { value: 4 },
            b"RuntimeInvisibleAnnotations",
            XrefOperation::Annotation,
        ),
        (
            QueryTarget::Literal {
                value: LiteralValue::Long { value: -9 },
            },
            LiteralValue::Long { value: -9 },
            b"RuntimeVisibleAnnotations",
            XrefOperation::Annotation,
        ),
        (
            QueryTarget::Literal {
                value: LiteralValue::Double {
                    value: 0x3ff0_0000_0000_0000,
                },
            },
            LiteralValue::Double {
                value: 0x3ff0_0000_0000_0000,
            },
            b"RuntimeVisibleAnnotations",
            XrefOperation::Annotation,
        ),
        (
            target_class("Lp/Literal;"),
            LiteralValue::Class {
                value: JvmBytes(b"Lp/Literal;".to_vec()),
            },
            b"RuntimeVisibleAnnotations",
            XrefOperation::Annotation,
        ),
    ];
    for (target, value, attribute, operation) in cases {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::LiteralValue,
                target,
                &[ConsumerKind::Annotation],
            ),
        );
        assert_eq!(operations(&report), vec![operation]);
        assert_eq!(report.items[0].target, XrefTarget::Literal { value });
        assert_eq!(attributes(&report), vec![Some(attribute.to_vec())]);
    }

    // Type annotation values and the parameter annotations' own descriptors.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::LiteralValue,
            target_string("p/TypeValue"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::TypeAnnotation]);
    assert_eq!(
        attributes(&report),
        vec![Some(b"RuntimeVisibleTypeAnnotations".to_vec())]
    );

    // Annotation content is only read for the annotation category.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Visible"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty());
}

/// The one class-file range that holds exactly these bytes.
///
/// A fixture this small holds the encoding of one structure once, so the range a fact
/// reports must be that range: the test states the bytes it wrote and lets the class
/// bytes themselves say where they are.
fn unique_span(built: &Built, needle: &[u8]) -> ByteSpan {
    let mut found = None;
    if built.bytes.len() >= needle.len() {
        for start in 0..=built.bytes.len() - needle.len() {
            if &built.bytes[start..start + needle.len()] == needle {
                assert!(
                    found.is_none(),
                    "the byte pattern must be unique in the fixture"
                );
                found = Some(ByteSpan::new(start as u64, needle.len() as u64));
            }
        }
    }
    found.expect("the fixture must hold this byte pattern")
}

#[test]
fn record_component_annotations_are_read_without_a_signature_request() {
    let built = record_annotation_fixture();
    let snapshot = open(built.bytes.clone());
    // No annotated type has a `CONSTANT_Class` entry: a hit can only come from the
    // component attribute this consumer reads, not from the pool.
    assert!(!built.has_class(b"p/RecordMarker"));
    assert!(!built.has_class(b"p/HiddenRecordMarker"));
    assert!(!built.has_class(b"p/ComponentTypeMarker"));

    // One annotation structure: the descriptor index, one element and its value.
    let mut structure = Vec::new();
    structure.extend_from_slice(&built.utf8_index(b"Lp/RecordMarker;").to_be_bytes());
    structure.extend_from_slice(&1u16.to_be_bytes());
    structure.extend_from_slice(&built.utf8_index(b"value").to_be_bytes());
    structure.push(b's');
    structure.extend_from_slice(&built.utf8_index(b"p/RecordValue").to_be_bytes());
    let expected = unique_span(&built, &structure);
    let visible_attribute = record_nested_attribute_spans(&built)[0].clone();

    // The annotation category alone answers it: no `Signature` was requested, and no
    // generated field or accessor carries the same annotation.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/RecordMarker"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Annotation));
    assert_eq!(item.derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    let (path, span) = attribute_span(item);
    assert_eq!(path, "Record.components[0].RuntimeVisibleAnnotations");
    assert_eq!(span, &expected, "the item keeps the annotation's own range");
    assert!(
        span.start >= visible_attribute.start
            && span.start + span.length <= visible_attribute.start + visible_attribute.length,
        "the range lies inside the component attribute that recorded it"
    );
    match &item.source.location {
        Location::Attribute { owner, .. } => assert!(matches!(
            owner.location,
            PhysicalClassLocation::StandaloneRoot { .. }
        )),
        other => panic!("expected a nested attribute provenance, got {other:?}"),
    }
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"RuntimeVisibleAnnotations".to_vec()))
    );
    assert_eq!(
        item.evidence.constant_pool_index,
        Some(built.utf8_index(b"Lp/RecordMarker;"))
    );
    assert_eq!(item.evidence.span, Some(expected.clone()));
    assert_eq!(item.evidence.bci, None, "a component annotation has no BCI");
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(report.coverage.unsupported_categories.is_empty());

    // A combined request answers the same position: the nested component annotation is not
    // a product of the other categories in the schema.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/RecordMarker"),
            &[
                ConsumerKind::Annotation,
                ConsumerKind::Signature,
                ConsumerKind::Type,
                ConsumerKind::InnerNest,
            ],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    assert_eq!(
        attribute_span(&report.items[0]).1,
        &expected,
        "the combined request keeps the nested position"
    );

    // The element value of the same position is a literal of the same annotation.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::LiteralValue,
            target_string("p/RecordValue"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    let (path, span) = attribute_span(&report.items[0]);
    assert_eq!(path, "Record.components[0].RuntimeVisibleAnnotations");
    assert_eq!(span, &expected);
    assert_eq!(
        report.items[0].evidence.constant_pool_index,
        Some(built.utf8_index(b"p/RecordValue"))
    );

    // An invisible component annotation is a position of its own, and the type
    // annotation of a third component keeps its structure range and names no BCI.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/HiddenRecordMarker"),
            &[ConsumerKind::Annotation],
        ),
    );
    let (path, _) = attribute_span(&report.items[0]);
    assert_eq!(path, "Record.components[1].RuntimeInvisibleAnnotations");
    // A field target carries no `target_info` bytes, so the structure is the target type,
    // an empty type path and the annotation itself.
    let mut type_structure = vec![0x13, 0x00];
    type_structure.extend_from_slice(&built.utf8_index(b"Lp/ComponentTypeMarker;").to_be_bytes());
    type_structure.extend_from_slice(&0u16.to_be_bytes());
    let expected = unique_span(&built, &type_structure);
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/ComponentTypeMarker"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::TypeAnnotation]);
    let (path, span) = attribute_span(&report.items[0]);
    assert_eq!(path, "Record.components[2].RuntimeVisibleTypeAnnotations");
    assert_eq!(
        span, &expected,
        "a type annotation starts at its target_type"
    );
    assert_eq!(report.items[0].evidence.bci, None);

    // The component descriptor stays a `Signature` product: it is not read for an
    // annotation request, and it is read for a signature request.
    let (report, _) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/DescriptorOnly"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    assert_complete(&report);
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/DescriptorOnly"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::RecordComponent]);
    assert_eq!(attributes(&report), vec![Some(b"Record".to_vec())]);

    // Bytes that only a custom component attribute holds are not a position this
    // consumer reads: no item, and the scan still covers its schema completely.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyInCustom"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert!(report.items.is_empty());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // Order: the `Record` walk runs before the flat class/field/method annotations, so
    // the component position comes first for the same annotation type.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/SharedMarker"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::Annotation, XrefOperation::Annotation]
    );
    let (path, span) = attribute_span(&report.items[0]);
    assert_eq!(path, "Record.components[0].RuntimeVisibleAnnotations");
    assert_eq!(
        &span_bytes(&built, span)[..2],
        &built.utf8_index(b"Lp/SharedMarker;").to_be_bytes(),
        "the nested structure is the one this item describes"
    );
    let (_, offset) = class_offset(&report.items[1]);
    assert_eq!(
        offset,
        built
            .entry_span(built.utf8_index(b"Lp/SharedMarker;"))
            .start,
        "the flat class-level annotation keeps its class offset"
    );
}

#[test]
fn code_type_annotations_are_read_without_decoding_the_body() {
    let built = code_annotation_fixture();
    let snapshot = open(built.bytes.clone());
    assert!(!built.has_class(b"p/CodeMarker"));
    assert!(!built.has_class(b"p/CatchMarker"));
    let code_entry = built.attribute_bytes_of(&[b"Code"]);

    // The `new` target names a bytecode offset, so the item records that BCI; the whole
    // structure is the range the annotation occupies inside the `Code` content.
    let mut structure = vec![0x44, 0x00, 0x00, 0x00];
    structure.extend_from_slice(&built.utf8_index(b"Lp/CodeMarker;").to_be_bytes());
    structure.extend_from_slice(&1u16.to_be_bytes());
    structure.extend_from_slice(&built.utf8_index(b"value").to_be_bytes());
    structure.push(b's');
    structure.extend_from_slice(&built.utf8_index(b"p/CodeValue").to_be_bytes());
    let expected = unique_span(&built, &structure);
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/CodeMarker"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );
    assert_complete(&report);
    assert_eq!(operations(&report), vec![XrefOperation::TypeAnnotation]);
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Annotation));
    assert_eq!(item.certainty, XrefCertainty::Exact);
    let (path, span) = attribute_span(item);
    assert_eq!(path, "methods[0].Code.RuntimeVisibleTypeAnnotations");
    assert_eq!(span, &expected, "the item keeps the structure's own range");
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"RuntimeVisibleTypeAnnotations".to_vec()))
    );
    assert_eq!(
        item.evidence.constant_pool_index,
        Some(built.utf8_index(b"Lp/CodeMarker;"))
    );
    assert_eq!(item.evidence.span, Some(expected.clone()));
    assert_eq!(
        item.evidence.bci,
        Some(0),
        "a `new` target names the instruction's offset"
    );
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(
        !report
            .coverage
            .unsupported_categories
            .contains(&ConsumerKind::Debug),
        "a type annotation inside a body is not a debug-table fact"
    );
    // The `Code` entry is billed once, and the nested content is inside that charge: the
    // metadata consumer never decodes an instruction.
    assert_eq!(
        usage.attribute_bytes,
        built.attribute_bytes()
            + built.attribute_bytes_of(&[b"RuntimeVisibleAnnotations"])
            + code_entry,
        "one pass over the flat annotation content and one over the Code entry"
    );
    assert_eq!(usage.code_bytes, 0);

    // A combined request answers the same position: the type annotation inside the body is
    // not a product of the other categories in the schema.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/CodeMarker"),
            &[
                ConsumerKind::Annotation,
                ConsumerKind::Signature,
                ConsumerKind::Constant,
                ConsumerKind::Exception,
            ],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::TypeAnnotation]);
    assert_eq!(attribute_span(&report.items[0]).1, &expected);

    // A `catch` target names an exception-table index, not an offset, so no BCI is
    // claimed for it (JVMS 4.7.20).
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/CatchMarker"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::TypeAnnotation]);
    let (path, _) = attribute_span(&report.items[0]);
    assert_eq!(path, "methods[0].Code.RuntimeInvisibleTypeAnnotations");
    assert_eq!(report.items[0].evidence.bci, None);

    // A request that does not name the annotation category never reads a `Code` content,
    // so the structure inside it cannot fail that request.
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/CodeMarker"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    assert_complete(&report);
    assert_eq!(
        usage.attribute_bytes,
        built.attribute_bytes(),
        "only the shell pass is billed without the annotation category"
    );
    assert_eq!(usage.code_bytes, 0);

    // Bytes only a custom `Code`-nested attribute holds are not a position this consumer
    // reads.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyInCustom"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert!(report.items.is_empty());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // Order: the flat class-level annotations run before the annotations inside a body.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/SharedMarker"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::Annotation, XrefOperation::TypeAnnotation]
    );
    let (_, offset) = class_offset(&report.items[0]);
    assert_eq!(
        offset,
        built
            .entry_span(built.utf8_index(b"Lp/SharedMarker;"))
            .start,
        "the flat class-level annotation comes first and keeps its class offset"
    );
    let (path, span) = attribute_span(&report.items[1]);
    assert_eq!(path, "methods[0].Code.RuntimeVisibleTypeAnnotations");
    let nested = span_bytes(&built, span);
    assert_eq!(
        nested[0], 0x44,
        "a type annotation starts at its target_type"
    );
    assert_eq!(
        &nested[4..6],
        &built.utf8_index(b"Lp/SharedMarker;").to_be_bytes(),
        "the nested structure is the one this item describes"
    );
}

#[test]
fn code_nested_structure_is_read_only_when_the_position_is_requested() {
    let built = broken_code_structure_fixture();
    let snapshot = open(built.bytes.clone());

    // The nested declaration contradicts the bytes that follow it, but a request that
    // does not name the annotation category never reads that content at all.
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Nothing"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    assert_complete(&report);
    assert_eq!(usage.attribute_bytes, built.attribute_bytes());

    // The category that does read it stops with a structured error instead of a partial
    // fact set that would read as complete.
    let (report, _) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Nothing"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    assert_eq!(
        failed_code(&report),
        Some("classfile_invalid_attribute_content"),
        "{:?}",
        report.diagnostics
    );
    assert_ne!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
}

/// The design's R2 record counterexample: real `javac 23.0.1 --release 17` output whose
/// only reference to the annotation type is a record component's nested attribute. See
/// `tests/fixtures/r2-annotation-positions/README.md` for source, command and digest.
const RECORD_ONLY: &[u8] = include_bytes!("fixtures/r2-annotation-positions/v17/RecordOnly.class");

/// The design's R2 body counterexample: real `javac 23.0.1 --release 8` output whose only
/// reference to the annotation type is a type annotation inside a method's `Code`.
const CODE_ONLY: &[u8] = include_bytes!("fixtures/r2-annotation-positions/v8/CodeOnly.class");

#[test]
fn a_nested_structure_that_fails_publishes_none_of_its_facts() {
    // Both nested positions, with one structure that produces the fact for the queried
    // symbol and then fails inside itself. The fact is only meaningful at the position it
    // was read from, so a run that cannot reach that structure's end may not publish it
    // anywhere: an item with flat coordinates would claim a position this class does not
    // have for this fact, and would also lose the nested attribute span and its BCI.
    let cases: [(NestedPosition, &str, &[u8], Option<u32>); 2] = [
        (
            NestedPosition::Code,
            "methods[0].Code.RuntimeVisibleTypeAnnotations",
            b"RuntimeVisibleTypeAnnotations",
            Some(0),
        ),
        (
            NestedPosition::Record,
            "Record.components[0].RuntimeVisibleAnnotations",
            b"RuntimeVisibleAnnotations",
            None,
        ),
    ];
    for (position, path, attribute_name, bci) in cases {
        let built = nested_tail_fixture(position, NestedContent::Malformed);
        let snapshot = open(built.bytes.clone());
        let (report, usage) = query_with(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("p/TailMarker"),
                &[ConsumerKind::Annotation],
            ),
            limits(),
        );
        assert!(
            report.items.is_empty(),
            "{path}: a fact whose structure did not parse is not published: {:?}",
            report.items
        );
        assert_eq!(
            failed_code(&report),
            Some("query_annotation_malformed"),
            "{path}: {:?}",
            report.diagnostics
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "query_annotation_malformed"),
            "{path}: the stop is explained"
        );
        assert_ne!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "{path}"
        );
        assert_eq!(
            usage.result_items, 0,
            "{path}: a discarded fact is never billed"
        );

        // The control: the same fixture with the one tag byte replaced by a tag this
        // consumer reads. The same fact then comes back, at the structure's own range and
        // with the position's own BCI, so the discard above is not a lost category.
        let built = nested_tail_fixture(position, NestedContent::Valid);
        let snapshot = open(built.bytes.clone());
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("p/TailMarker"),
                &[ConsumerKind::Annotation],
            ),
        );
        assert_eq!(report.items.len(), 1, "{path}: {:?}", report.items);
        let (item_path, span) = attribute_span(&report.items[0]);
        assert_eq!(item_path, path, "{path}");
        assert_eq!(report.items[0].evidence.bci, bci, "{path}");
        assert_eq!(
            report.items[0].evidence.attribute,
            Some(ArchiveNameBytes(attribute_name.to_vec())),
            "{path}"
        );
        assert_eq!(
            report.items[0].evidence.constant_pool_index,
            Some(built.utf8_index(TAIL_MARKER)),
            "{path}"
        );
        assert_eq!(report.items[0].evidence.span, Some(span.clone()), "{path}");
        // The reported range really holds the structure this item describes: the annotation
        // type of the queried symbol, at the offset the position puts it (after
        // `target_type`, `target_info` and `type_path` for a type annotation).
        let bytes = span_bytes(&built, span);
        let descriptor = match position {
            NestedPosition::Code => &bytes[4..6],
            NestedPosition::Record => &bytes[0..2],
        };
        assert_eq!(
            descriptor,
            &built.utf8_index(TAIL_MARKER).to_be_bytes(),
            "{path}"
        );

        // The boundary the discard draws: a structure that ended *before* the failing one is
        // a complete fact of its own and stays published, with its own range — the same rule
        // a flat position follows for the annotations before the failure.
        let built = nested_tail_fixture(position, NestedContent::ValidThenMalformed);
        let snapshot = open(built.bytes.clone());
        let (report, usage) = query_with(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("p/TailMarker"),
                &[ConsumerKind::Annotation],
            ),
            limits(),
        );
        assert_eq!(report.items.len(), 2, "{path}: {:?}", report.items);
        assert_eq!(failed_code(&report), Some("query_annotation_malformed"));
        assert_eq!(
            usage.result_items, 2,
            "{path}: only published facts are billed"
        );
        let spans: Vec<ByteSpan> = report
            .items
            .iter()
            .map(|item| {
                let (item_path, span) = attribute_span(item);
                assert_eq!(item_path, path, "{path}");
                assert_eq!(item.evidence.bci, bci, "{path}");
                span.clone()
            })
            .collect();
        assert_eq!(
            spans[1],
            ByteSpan::new(spans[0].start + spans[0].length, spans[0].length),
            "{path}: the two complete structures keep their own adjacent ranges"
        );
    }
}

#[test]
fn a_flat_annotation_prefix_keeps_its_class_offset() {
    // The flat class, field and method positions are unchanged by the nested-position
    // work: a fact read before the failure is still published, at the class offset of the
    // pool entry it was read through, and it is charged.
    let built = malformed_annotation_fixture();
    let snapshot = open(built.bytes.clone());
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/A"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    assert_eq!(failed_code(&report), Some("query_annotation_malformed"));
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.utf8_index(b"Lp/A;"),
        Some(b"RuntimeVisibleAnnotations"),
    );
    assert_eq!(report.items[0].evidence.bci, None);
    assert_eq!(usage.result_items, 1);
}

#[test]
fn a_class_signature_needs_a_class_type_signature() {
    // JVMS 4.7.9.1: a `ClassSignature` names its superclass and superinterfaces with
    // `ClassTypeSignature`s, so a type variable or an array is not a legal superclass. The
    // lenient reading used to walk them as reference types and report the class after
    // them; both fixtures below query exactly the class that reading extracted.
    for signature_bytes in [b"Tp/Bound;Lp/Super;".as_slice(), b"[Lp/Super;"] {
        let built = class_signature_fixture(signature_bytes);
        let snapshot = open(built.bytes.clone());
        let (report, usage) = query_with(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("p/Super"),
                &[ConsumerKind::Signature],
            ),
            limits(),
        );
        assert!(
            report.items.is_empty(),
            "{signature_bytes:?}: a superclass is a class type signature: {:?}",
            report.items
        );
        assert_eq!(
            failed_code(&report),
            Some("query_signature_malformed"),
            "{signature_bytes:?}"
        );
        assert_ne!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "{signature_bytes:?}"
        );
        assert_eq!(usage.result_items, 0, "{signature_bytes:?}");
    }

    // Control: the same fixture with a class type signature reports the type, so the
    // rejections above are about the production and not about the query shape.
    let built = class_signature_fixture(b"Lp/Super;");
    let snapshot = open(built.bytes.clone());
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Super"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::GenericSignature]);
    assert_eq!(owners(&report), vec![b"p/Super".to_vec()]);
}

#[test]
fn a_compiled_record_component_annotation_is_found_at_its_own_position() {
    let snapshot = open(RECORD_ONLY.to_vec());

    // Annotation-only: the compiled sample has one record component, and the annotation
    // lives in that component's nested attribute rather than in a field, an accessor or a
    // class-level attribute.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("RecordMarker"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::Annotation],
        "{:?}",
        report.items
    );
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Annotation));
    assert_eq!(item.certainty, XrefCertainty::Exact);
    let (path, span) = attribute_span(item);
    assert_eq!(path, "Record.components[0].RuntimeVisibleAnnotations");
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"RuntimeVisibleAnnotations".to_vec()))
    );
    // The reported range is the annotation structure itself: it starts with the constant
    // pool index of the annotation's type descriptor, which is the entry the item names.
    let annotation = &RECORD_ONLY
        [usize::try_from(span.start).unwrap()..usize::try_from(span.start + span.length).unwrap()];
    assert_eq!(
        item.evidence.constant_pool_index.map(u16::to_be_bytes),
        Some([annotation[0], annotation[1]])
    );
    assert_eq!(item.evidence.bci, None);
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
}

#[test]
fn a_compiled_code_type_annotation_is_found_without_decoding_the_body() {
    let snapshot = open(CODE_ONLY.to_vec());

    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("CodeMarker"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );
    assert_complete(&report);
    assert_eq!(
        operations(&report),
        vec![XrefOperation::TypeAnnotation],
        "{:?}",
        report.items
    );
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Annotation));
    let (path, span) = attribute_span(item);
    assert_eq!(path, "methods[1].Code.RuntimeVisibleTypeAnnotations");
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"RuntimeVisibleTypeAnnotations".to_vec()))
    );
    // A CAST type annotation: `target_type` 0x47, the checkcast's offset, the type
    // argument index, an empty type path and then the annotation itself.
    let annotation = &CODE_ONLY
        [usize::try_from(span.start).unwrap()..usize::try_from(span.start + span.length).unwrap()];
    assert_eq!(annotation[0], 0x47, "the sample annotates a cast");
    assert_eq!(
        item.evidence.bci,
        Some(u32::from(u16::from_be_bytes([
            annotation[1],
            annotation[2]
        ])))
    );
    assert_eq!(
        item.evidence.constant_pool_index.map(u16::to_be_bytes),
        Some([annotation[5], annotation[6]]),
        "the structure ends at the annotation whose type the item names"
    );
    // Reading the nested metadata decodes no instruction and claims no debug table.
    assert_eq!(usage.code_bytes, 0);
    assert!(report.coverage.unsupported_categories.is_empty());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // A request that names no annotation category never reaches the position, so it cannot
    // report it.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("CodeMarker"),
            &[ConsumerKind::Signature],
        ),
    );
    assert!(report.items.is_empty());
}

/// A03: categories P1 declares but cannot scan are a schema fact, not a negative result.
///
/// `Verification` and `Debug` need `StackMapTable`/LVT decoding P0 does not provide, so a
/// request that names them must list them as unsupported and must not claim
/// `complete-within-schema` — for them, and for the whole request that contains them.
#[test]
fn requested_verification_and_debug_never_claim_a_complete_schema() {
    let snapshot = open(RECORD_ONLY.to_vec());

    // The positive control: the annotation category alone scans the compiled sample's
    // record-component position to completion.
    let (annotation_only, annotation_usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("RecordMarker"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );
    assert_complete(&annotation_only);
    assert_eq!(
        operations(&annotation_only),
        vec![XrefOperation::Annotation]
    );
    assert!(annotation_only.coverage.unsupported_categories.is_empty());
    assert_eq!(
        annotation_only
            .coverage
            .dimensions
            .artifact_structural
            .state,
        CoverageState::CompleteWithinSchema
    );

    // Adding the unsupported categories answers exactly the same facts: the declaration
    // does not turn the scan into a failure, and the sample is still read for the
    // categories that are implemented.
    for unsupported in [
        vec![ConsumerKind::Verification],
        vec![ConsumerKind::Debug],
        vec![ConsumerKind::Verification, ConsumerKind::Debug],
    ] {
        let mut kinds = vec![ConsumerKind::Annotation];
        kinds.extend(unsupported.iter().copied());
        let (report, usage) = query_with(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("RecordMarker"),
                &kinds,
            ),
            limits(),
        );
        assert_eq!(
            report.items, annotation_only.items,
            "{unsupported:?} must not change the facts"
        );
        assert_eq!(
            report.coverage.unsupported_categories, unsupported,
            "{unsupported:?} must be named as unsupported"
        );
        assert_ne!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "{unsupported:?} is requested but never scanned"
        );
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "{unsupported:?}: an unsupported category is not an execution failure"
        );
        assert!(
            report.diagnostics.is_empty(),
            "{unsupported:?}: no failure is reported for a declared category: {:?}",
            report.diagnostics
        );
        assert_eq!(usage.result_items, annotation_usage.result_items);
        assert_eq!(usage.class_bytes, annotation_usage.class_bytes);
    }

    // A request that names only an unsupported category reads and reports nothing, and
    // still may not claim the schema was covered.
    for kind in [ConsumerKind::Verification, ConsumerKind::Debug] {
        let (report, usage) = query_with(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("RecordMarker"),
                &[kind],
            ),
            limits(),
        );
        assert!(report.items.is_empty());
        assert_eq!(report.coverage.unsupported_categories, vec![kind]);
        assert_ne!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert_eq!(
            (usage.read_bytes, usage.class_bytes, usage.attribute_bytes),
            (0, 0, 0),
            "{kind:?}: P1 does not scan this category, so it reads no class bytes"
        );
    }
}

// ---------------------------------------------------------------------------
// A03: the method signature grammar (`Result` and `{ThrowsSignature}`)
// ---------------------------------------------------------------------------

/// Real `javac 23.0.1 --release 17` samples. Source, command and digests are recorded in
/// `tests/fixtures/method-signature-grammar/README.md`.
const GENERIC_RETURN_CLASS: &[u8] =
    include_bytes!("fixtures/method-signature-grammar/v17/GenericReturn.class");
const TYPE_VAR_RESULT_CLASS: &[u8] =
    include_bytes!("fixtures/method-signature-grammar/v17/TypeVarResult.class");
const THROWS_VAR_CLASS: &[u8] =
    include_bytes!("fixtures/method-signature-grammar/v17/ThrowsVar.class");
const THROWS_MIXED_CLASS: &[u8] =
    include_bytes!("fixtures/method-signature-grammar/v17/ThrowsMixed.class");

/// A generic return type: `()Ljava/util/List<Ljava/lang/String;>;`.
const GENERIC_RETURN_SIGNATURE: &[u8] = b"()Ljava/util/List<Ljava/lang/String;>;";
/// A type variable as the result: `(TT;)TT;`.
const TYPE_VAR_RESULT_SIGNATURE: &[u8] = b"(TT;)TT;";
/// `throws` a type variable: `()V^TE;`.
const THROWS_VAR_SIGNATURE: &[u8] = b"()V^TE;";
/// `throws E, java.io.IOException`: `()V^TE;^Ljava/io/IOException;`.
const THROWS_MIXED_SIGNATURE: &[u8] = b"()V^TE;^Ljava/io/IOException;";
/// The class signature `TypeVarResult` carries: `<T:Ljava/lang/Object;>Ljava/lang/Object;`.
const OBJECT_BOUND_SIGNATURE: &[u8] = b"<T:Ljava/lang/Object;>Ljava/lang/Object;";
/// The class signature `ThrowsVar` and `ThrowsMixed` carry.
const EXCEPTION_BOUND_SIGNATURE: &[u8] = b"<E:Ljava/lang/Exception;>Ljava/lang/Object;";

/// Whether these bytes occur in the class file at all.
///
/// Used to state that a fixture really is the sample a test describes, so an empty result
/// cannot come from a signature that was never in the file.
fn contains_bytes(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.windows(needle.len()).any(|window| window == needle)
}

/// Class-file range of the one `CONSTANT_Utf8` entry holding exactly these bytes.
///
/// A compiled sample keeps its signature as one pool string, so a test can require the
/// reported index and span to be that entry instead of trusting a number the implementation
/// also produced. The assertion is read back out of the class bytes: the walk counts pool
/// entries the way JVMS 4.4 does (a `Long`/`Double` reserves the following slot).
fn assert_signature_entry(bytes: &[u8], item: &XrefItem, value: &[u8]) -> ByteSpan {
    let span = item
        .evidence
        .span
        .clone()
        .expect("a signature item records its pool entry span");
    let start = usize::try_from(span.start).expect("fixture offset fits usize");
    assert_eq!(
        span.length,
        u64::try_from(value.len() + 3).expect("length fits u64"),
        "a CONSTANT_Utf8 entry is its tag, its length and its payload"
    );
    assert_eq!(bytes[start], 1, "the span starts at the CONSTANT_Utf8 tag");
    assert_eq!(
        u16::from_be_bytes([bytes[start + 1], bytes[start + 2]]) as usize,
        value.len()
    );
    assert_eq!(
        &bytes[start + 3..start + 3 + value.len()],
        value,
        "the entry holds the signature the sample compiles to"
    );

    let count = u16::from_be_bytes([bytes[8], bytes[9]]);
    let mut at = 10usize;
    let mut index = 1u16;
    let mut found = None;
    while index < count {
        if at == start {
            found = Some(index);
        }
        let tag = bytes[at];
        at += 1;
        let width = match tag {
            1 => {
                let length = usize::from(u16::from_be_bytes([bytes[at], bytes[at + 1]]));
                at += 2 + length;
                0
            }
            3 | 4 => 4,
            5 | 6 => {
                index += 1; // the reserved slot
                8
            }
            7 | 8 | 16 | 19 | 20 => 2,
            9 | 10 | 11 | 12 | 17 | 18 => 4,
            15 => 3,
            other => panic!("fixture pool holds an unknown tag {other}"),
        };
        at += width;
        index += 1;
    }
    assert!(
        at <= bytes.len(),
        "the pool walk must stay inside the class bytes"
    );
    let expected = found.expect("the fixture pool must hold an entry at that span");
    assert_eq!(
        item.evidence.constant_pool_index,
        Some(expected),
        "the reported index is this entry's"
    );
    span
}

/// Asserts that one item answers a signature request with the sample's signature entry.
fn assert_signature_item(bytes: &[u8], item: &XrefItem, signature: &[u8]) {
    assert_eq!(item.operation, XrefOperation::GenericSignature);
    assert_eq!(item.consumer, Some(ConsumerKind::Signature));
    assert_eq!(item.derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"Signature".to_vec()))
    );
    assert_eq!(
        item.evidence.bci, None,
        "a signature has no bytecode position"
    );
    assert!(item.evidence.via.is_empty());
    assert_signature_entry(bytes, item, signature);
    let (definition, offset) = class_offset(item);
    assert!(matches!(
        definition.location,
        PhysicalClassLocation::StandaloneRoot { .. }
    ));
    assert_eq!(offset, item.evidence.span.as_ref().unwrap().start);
}

#[test]
fn a_compiled_generic_return_signature_is_read() {
    let bytes = GENERIC_RETURN_CLASS;
    let snapshot = open(bytes.to_vec());
    assert!(contains_bytes(bytes, GENERIC_RETURN_SIGNATURE));

    // `java/lang/String` exists only inside the signature: no `CONSTANT_Class` entry, and no
    // other `Utf8` spells it (the method's own descriptor is `()Ljava/util/List;`). Before
    // the fix this legal sample failed with `query_signature_malformed` instead of reporting
    // the type.
    for name in ["java/lang/String", "java/util/List"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::GenericSignature],
            "{name}"
        );
        assert_eq!(report.items.len(), 1, "{name}: {:?}", report.items);
        assert_signature_item(bytes, &report.items[0], GENERIC_RETURN_SIGNATURE);
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        assert!(report.coverage.unsupported_categories.is_empty());
    }

    // The category gate: a request that never names `Signature` does not read the attribute
    // at all, so the same bytes cannot fail it and the type stays unreported.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("java/lang/String"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty(), "{:?}", report.items);
}

#[test]
fn a_compiled_type_variable_result_is_not_a_class_reference() {
    let bytes = TYPE_VAR_RESULT_CLASS;
    let snapshot = open(bytes.to_vec());
    assert!(contains_bytes(bytes, TYPE_VAR_RESULT_SIGNATURE));

    // The sample's class signature parsed before the fix; it is the control that the class
    // really was scanned. `java/lang/Object` is named by it twice but reported once, and the
    // method signature names no class type at all.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("java/lang/Object"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    assert_signature_item(bytes, &report.items[0], OBJECT_BOUND_SIGNATURE);

    // The method signature `(TT;)TT;` is read to its end and contributes no item: a type
    // variable is not a class reference. Before the fix the whole scan was `Failed`.
    for name in ["T", "TT"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature, ConsumerKind::Type],
            ),
        );
        assert!(report.items.is_empty(), "{name}: {:?}", report.items);
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "{name}"
        );
    }
}

#[test]
fn a_compiled_throws_signature_reports_only_its_class_type() {
    let bytes = THROWS_MIXED_CLASS;
    let snapshot = open(bytes.to_vec());
    assert!(contains_bytes(bytes, THROWS_MIXED_SIGNATURE));

    // `throws E, java.io.IOException`: the class type in the `ThrowsSignature` list is a
    // type reference of the signature category. The sample also has a `CONSTANT_Class` entry
    // for that name (its `Exceptions` attribute needs one), which a `Signature`-only request
    // never reads, so the item can only come from the signature itself.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("java/io/IOException"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    assert_signature_item(bytes, &report.items[0], THROWS_MIXED_SIGNATURE);
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);

    // The class signature's bound is the only other item this sample holds, and the type
    // variable of the `ThrowsSignature` list is never one.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("java/lang/Exception"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    assert_signature_item(bytes, &report.items[0], EXCEPTION_BOUND_SIGNATURE);
    for name in ["E", "TE"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature, ConsumerKind::Type],
            ),
        );
        assert!(report.items.is_empty(), "{name}: {:?}", report.items);
    }
}

#[test]
fn a_compiled_throws_type_variable_is_read_without_an_item() {
    let bytes = THROWS_VAR_CLASS;
    let snapshot = open(bytes.to_vec());
    assert!(contains_bytes(bytes, THROWS_VAR_SIGNATURE));

    // `()V^TE;` alone: legal, so the scan is `Complete` with no diagnostic, and the only
    // thing this sample can report is the type variable's bound from the class signature.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("java/lang/Exception"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    assert_signature_item(bytes, &report.items[0], EXCEPTION_BOUND_SIGNATURE);
    for name in ["E", "TE"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature, ConsumerKind::Type],
            ),
        );
        assert!(report.items.is_empty(), "{name}: {:?}", report.items);
    }
}

/// A class with one method whose `Signature` attribute holds these bytes.
fn method_signature_fixture(text: &[u8]) -> Built {
    let mut class = Class::new(b"p/Signed", Some(b"java/lang/Object"));
    let signature = signature(&mut class, text);
    class.method(
        ACCESS_PUBLIC,
        b"m",
        b"()V",
        &[
            attribute(b"Code", empty_code()),
            attribute(b"Signature", signature),
        ],
    );
    class.finish()
}

#[test]
fn a_method_signature_grammar_still_rejects_malformed_content() {
    // A legal signature of this shape is the control: a generic parameter type, a result and
    // a `ThrowsSignature` list holding both a class type and a type variable.
    let signature = b"(Ljava/util/List<Lp/Elem;>;)V^Lp/Thrown;^TX;";
    let built = method_signature_fixture(signature);
    let snapshot = open(built.bytes.clone());
    for name in ["p/Elem", "p/Thrown"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Signature],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::GenericSignature],
            "{name}"
        );
        assert_pool_evidence(
            &report.items[0],
            &built,
            built.utf8_index(signature),
            Some(b"Signature"),
        );
    }
    // The type variable of the same `ThrowsSignature` list is not a class reference.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("TX"),
            &[ConsumerKind::Signature, ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty(), "{:?}", report.items);

    // The same fixture with bytes the grammar cannot accept: each one is a structured stop
    // with its own code, never a partial fact set that would read as complete.
    let cases: [(&str, &[u8]); 6] = [
        // A generic result whose class type is never terminated.
        ("unterminated result type", b"()Ljava/util/List<Lp/Elem;"),
        // A `ThrowsSignature` class type is part of the grammar too.
        ("unterminated throws type", b"()V^Lp/Unterminated"),
        // A type variable needs its own terminator.
        ("unterminated throws variable", b"()V^TE"),
        // The marker needs the type it introduces.
        ("empty throws signature", b"()V^"),
        // A legal signature followed by bytes no production covers.
        ("trailing bytes", b"()VX;"),
        // The result position cannot be empty.
        ("missing result", b"()"),
    ];
    for (what, signature) in cases {
        let built = method_signature_fixture(signature);
        let snapshot = open(built.bytes.clone());
        let (report, _) = query_with(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol("p/Anything"),
                &[ConsumerKind::Signature],
            ),
            limits(),
        );
        assert_eq!(
            failed_code(&report),
            Some("query_signature_malformed"),
            "{what}: {:?}",
            report.diagnostics
        );
        assert!(
            report.items.is_empty(),
            "{what} contributes no fact: {:?}",
            report.items
        );
        assert_ne!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "{what}"
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "query_signature_malformed"),
            "{what} is explained by a diagnostic"
        );
    }
}

// ---------------------------------------------------------------------------
// Inner/nest relations
// ---------------------------------------------------------------------------

#[test]
fn inner_nest_relations_keep_their_kind() {
    let built = inner_nest_fixture();
    let snapshot = open(built.bytes.clone());

    // The anonymous entry still records both of its class references.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Outer$1"),
            &[ConsumerKind::InnerNest],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::InnerClass]);
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.class_index(b"p/Outer$1"),
        Some(b"InnerClasses"),
    );

    // The declaring class is both an outer class, the enclosing class and the nest
    // host, in the order the attributes are read.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Outer"),
            &[ConsumerKind::InnerNest],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![
            XrefOperation::InnerClass,
            XrefOperation::InnerClass,
            XrefOperation::EnclosingMethod,
            XrefOperation::NestHost,
        ]
    );
    assert_eq!(
        attributes(&report),
        vec![
            Some(b"InnerClasses".to_vec()),
            Some(b"InnerClasses".to_vec()),
            Some(b"EnclosingMethod".to_vec()),
            Some(b"NestHost".to_vec()),
        ]
    );

    // The enclosing member is a method symbol of the enclosing class.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_method("p/Outer", "run", "()V"),
            &[ConsumerKind::InnerNest],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::EnclosingMethod]);
    assert_eq!(
        report.items[0].target,
        XrefTarget::Symbol {
            value: SymbolRef::Method {
                owner: JvmBytes(b"p/Outer".to_vec()),
                name: JvmBytes(b"run".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            },
        }
    );

    // A member class is recorded by `InnerClasses` and by `NestMembers`.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Outer$Inner"),
            &[ConsumerKind::InnerNest],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::InnerClass, XrefOperation::NestMembers]
    );
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Outer$2"),
            &[ConsumerKind::InnerNest],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::NestMembers]);
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Impl"),
            &[ConsumerKind::InnerNest],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::PermittedSubclasses]
    );

    // The simple name of a member entry is a name, not a class reference.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("Inner"),
            &[ConsumerKind::InnerNest],
        ),
    );
    assert!(report.items.is_empty());

    // `InnerNest` is not read for another category.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Outer$2"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty());
}

// ---------------------------------------------------------------------------
// Constant values and modules
// ---------------------------------------------------------------------------

#[test]
fn constant_values_are_reported_as_values_only() {
    let (built, indexes) = constant_fixture();
    let snapshot = open(built.bytes.clone());

    // One item per declaration that records the value, and the target is the value.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::LiteralValue,
            target_integer(7),
            &[ConsumerKind::Constant],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::ConstantValue, XrefOperation::ConstantValue]
    );
    assert_eq!(
        report.items[0].target,
        XrefTarget::Literal {
            value: LiteralValue::Integer { value: 7 }
        }
    );
    assert_eq!(report.items[0].consumer, Some(ConsumerKind::Constant));
    assert_pool_evidence(
        &report.items[0],
        &built,
        indexes.number,
        Some(b"ConstantValue"),
    );
    assert_eq!(
        report.items[0].source, report.items[1].source,
        "the same value in two fields is two facts at the same pool entry"
    );

    // A string constant and a long value keep their JVM shape.
    for (target, value, index) in [
        (
            target_string("abc"),
            LiteralValue::String {
                value: JvmBytes(b"abc".to_vec()),
            },
            indexes.text,
        ),
        (
            QueryTarget::Literal {
                value: LiteralValue::Long { value: -9 },
            },
            LiteralValue::Long { value: -9 },
            indexes.wide,
        ),
    ] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::LiteralValue,
                target,
                &[ConsumerKind::Constant],
            ),
        );
        assert_eq!(operations(&report), vec![XrefOperation::ConstantValue]);
        assert_eq!(report.items[0].target, XrefTarget::Literal { value });
        assert_pool_evidence(&report.items[0], &built, index, Some(b"ConstantValue"));
    }

    // A value is not a symbol reference, so the constant category never answers a
    // symbol request. (Another consumer also owns `Constant`, so this asserts the
    // absence of a fabricated symbol rather than the absence of a class read.)
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Constants"),
            &[ConsumerKind::Constant],
        ),
    );
    assert!(report.items.is_empty());
}

#[test]
fn module_uses_and_provides_are_class_nodes_on_module_info_only() {
    let built = module_fixture();
    let snapshot = open(built.bytes.clone());

    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Service"),
            &[ConsumerKind::Module],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::ModuleUses]);
    assert_eq!(report.items[0].consumer, Some(ConsumerKind::Module));
    assert_pool_evidence(
        &report.items[0],
        &built,
        built.class_index(b"p/Service"),
        Some(b"Module"),
    );

    for name in ["p/Api", "p/Impl", "p/Impl2"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Module],
            ),
        );
        assert_eq!(
            operations(&report),
            vec![XrefOperation::ModuleProvides],
            "{name}"
        );
        assert_eq!(owners(&report), vec![name.as_bytes().to_vec()]);
        assert_eq!(
            report.items[0].evidence.attribute,
            Some(ArchiveNameBytes(b"Module".to_vec()))
        );
    }

    // A class that is not `module-info` records no module relation, even with the
    // attribute present.
    let built = not_module_fixture();
    let snapshot = open(built.bytes.clone());
    for name in ["p/Service", "p/Api", "p/Impl"] {
        let report = run_complete(
            &snapshot,
            &query_request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target_symbol(name),
                &[ConsumerKind::Module],
            ),
        );
        assert!(report.items.is_empty(), "{name} on a non-module class");
    }
}

// ---------------------------------------------------------------------------
// Scope, unknown attributes, malformed content, budget and paging
// ---------------------------------------------------------------------------

#[test]
fn unrequested_categories_are_not_read_and_unknown_attributes_stay_invisible() {
    let built = meta_fixture();
    let snapshot = open(built.bytes.clone());
    let shells = built.attribute_bytes();
    let annotations = built.attribute_bytes_of(&[b"RuntimeVisibleAnnotations"]);
    let signature_attribute = built.attribute_bytes_of(&[b"Signature"]);

    // The reader fact layer bills one pass over every attribute shell; the content of
    // a category this request did not name is never read again. Both categories used
    // here belong to this consumer alone, so the byte counts are its own.
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlySigned"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert_complete(&report);
    assert_eq!(
        usage.attribute_bytes,
        shells + signature_attribute,
        "an unrequested annotation attribute is never read"
    );
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyAnnotated"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );
    assert_complete(&report);
    // An annotation request also reads each method's `Code` entry once, because a type
    // annotation inside a body is a standard annotation position (decision 25); the
    // signature attribute is still never read.
    assert_eq!(
        usage.attribute_bytes,
        shells + annotations + built.attribute_bytes_of(&[b"Code"]),
        "an unrequested signature attribute is never read, and the annotation scan reads \
         each Code entry once for the type annotations inside it"
    );

    // An attribute no consumer knows is never read and cannot fail the scan.
    let built = unknown_attribute_fixture();
    let snapshot = open(built.bytes.clone());
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/CustomSigned"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert_eq!(operations(&report), vec![XrefOperation::GenericSignature]);
    assert_eq!(
        usage.attribute_bytes,
        built.attribute_bytes() + built.attribute_bytes_of(&[b"Signature"]),
        "the unknown attribute is charged once as a shell and never read"
    );
    let (report, _) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/CustomSigned"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    // An unknown attribute is not this consumer's to read, and its presence does not
    // stop a scan of the categories that were requested.
    assert_complete(&report);
}

/// A pool entry is evidence only when it is the entry the fact came from.
///
/// When the pool spells the same bytes twice, no entry is identifiable, so the scan
/// keeps the fact and drops the claim it cannot support: no index, and the span falls
/// back to the range that recorded the fact. A control fact in the same fixture keeps its
/// exact entry, so the degradation is attributable to the ambiguity and not to a general
/// loss of evidence.
#[test]
fn ambiguous_pool_bytes_degrade_the_evidence_honestly() {
    let built = ambiguous_pool_fixture();
    let snapshot = open(built.bytes.clone());
    let whole_class = ByteSpan::new(0, built.bytes.len() as u64);

    // Control: one pool entry for this name, so the item names it.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Kept"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Interface]);
    assert_pool_evidence(&report.items[0], &built, built.class_index(b"p/Kept"), None);

    // Two `CONSTANT_Class` entries for the super class name: a header fact has no
    // attribute to fall back to, so the span is the class-wide range.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Super"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::SuperClass]);
    let item = &report.items[0];
    assert_eq!(item.evidence.constant_pool_index, None);
    assert_eq!(item.evidence.span, Some(whole_class.clone()));
    assert_eq!(class_offset(item).1, 0);
    assert_eq!(
        item.certainty,
        XrefCertainty::Exact,
        "the fact itself is exact; only its locating is degraded"
    );

    // Two `CONSTANT_Utf8` entries for a descriptor string: the descriptor is still a
    // verified fact, but the entry it came from is unknown.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Field"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::FieldDescriptor]);
    assert_eq!(owners(&report), vec![b"p/Field".to_vec()]);
    assert_eq!(report.items[0].evidence.constant_pool_index, None);
    assert_eq!(report.items[0].evidence.span, Some(whole_class.clone()));

    // Two `CONSTANT_Utf8` entries for a signature string: the fallback is the attribute
    // content that recorded the signature.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Signed"),
            &[ConsumerKind::Signature],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::GenericSignature]);
    assert_eq!(report.items[0].evidence.constant_pool_index, None);
    assert_eq!(
        report.items[0].evidence.span,
        Some(built.attribute_content_span(b"Signature"))
    );
    assert_eq!(
        report.items[0].evidence.attribute,
        Some(ArchiveNameBytes(b"Signature".to_vec()))
    );

    // Two `CONSTANT_Class` entries for an exception type: the fallback is the
    // `Exceptions` attribute content.
    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Thrown"),
            &[ConsumerKind::Exception],
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::ExceptionsAttribute]
    );
    assert_eq!(report.items[0].evidence.constant_pool_index, None);
    assert_eq!(
        report.items[0].evidence.span,
        Some(built.attribute_content_span(b"Exceptions"))
    );
}

#[test]
fn malformed_metadata_content_is_a_structured_stop() {
    let built = malformed_signature_fixture();
    let snapshot = open(built.bytes.clone());
    let (report, _) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/Unterminated"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    match &report.execution {
        // The reader fact layer's discipline: content that does not parse is a
        // structured stop, so an incomplete fact set is never published as complete.
        ExecutionReport::Failed {
            reason: TerminationReason::Error { code },
            ..
        } => assert_eq!(code, "query_signature_malformed"),
        other => panic!("a malformed signature must stop with a code, got {other:?}"),
    }
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "query_signature_malformed"),
        "the stop is explained by a diagnostic"
    );
    assert_ne!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // The same class answers nothing for a category whose content is fine.
    let (report, _) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("java/lang/Object"),
            &[ConsumerKind::Type],
        ),
        limits(),
    );
    assert_eq!(operations(&report), vec![XrefOperation::SuperClass]);
    // The malformed content belongs to a category this request did not name, so this
    // scan stays complete.
    assert_complete(&report);
}

/// One malformed-content case: the fixture, the request it is probed with, the code
/// the parser must report, and the facts read exactly before the stop.
struct MalformedCase {
    built: Built,
    relation: QueryRelation,
    target: QueryTarget,
    kind: ConsumerKind,
    code: &'static str,
    prefix: Vec<XrefOperation>,
}

#[test]
fn each_parser_reports_its_own_malformed_content() {
    // One case per parser this consumer owns: a truncated structure is a structured
    // stop with its own code, never a partial fact set that would read as complete.
    // Facts read exactly before the stop stay published as the reliable prefix.
    let cases = vec![
        MalformedCase {
            built: malformed_descriptor_fixture(),
            relation: QueryRelation::MentionsSymbol,
            target: target_symbol("p/BadDescriptor"),
            kind: ConsumerKind::Type,
            code: "query_descriptor_malformed",
            prefix: Vec::new(),
        },
        MalformedCase {
            built: malformed_signature_fixture(),
            relation: QueryRelation::MentionsSymbol,
            target: target_symbol("p/Unterminated"),
            kind: ConsumerKind::Signature,
            code: "query_signature_malformed",
            prefix: Vec::new(),
        },
        MalformedCase {
            built: malformed_annotation_fixture(),
            relation: QueryRelation::MentionsSymbol,
            target: target_symbol("p/A"),
            kind: ConsumerKind::Annotation,
            code: "query_annotation_malformed",
            // The annotation type parsed before the unknown element value tag, so that
            // one verified fact is the reliable prefix.
            prefix: vec![XrefOperation::Annotation],
        },
        MalformedCase {
            built: malformed_record_fixture(),
            relation: QueryRelation::MentionsSymbol,
            target: target_symbol("p/BadRecord"),
            kind: ConsumerKind::Signature,
            code: "query_record_malformed",
            prefix: Vec::new(),
        },
        MalformedCase {
            built: malformed_constant_fixture(),
            relation: QueryRelation::LiteralValue,
            target: target_integer(7),
            kind: ConsumerKind::Constant,
            code: "query_constant_value_malformed",
            prefix: Vec::new(),
        },
    ];
    for case in cases {
        let MalformedCase {
            built,
            relation,
            target,
            kind,
            code,
            prefix,
        } = case;
        let snapshot = open(built.bytes.clone());
        let (report, _) = query_with(
            &snapshot,
            &query_request(&snapshot, relation, target, &[kind]),
            limits(),
        );
        assert_eq!(operations(&report), prefix, "{code}");
        match &report.execution {
            ExecutionReport::Failed {
                reason: TerminationReason::Error { code: actual },
                ..
            } => assert_eq!(actual, code),
            other => panic!("{code} must stop the scan, got {other:?}"),
        }
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == code),
            "{code} is explained by a diagnostic"
        );
        assert_ne!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "{code}"
        );
    }
}

#[test]
fn scans_are_deterministic_and_paginate_without_gaps() {
    let built = inner_nest_fixture();
    let snapshot = open(built.bytes.clone());
    let full_request = query_request(
        &snapshot,
        QueryRelation::MentionsSymbol,
        target_symbol("p/Outer"),
        &[ConsumerKind::InnerNest],
    );
    let full = run_complete(&snapshot, &full_request);
    assert!(full.items.len() >= 4);
    assert!(!full.page.has_more);
    assert!(full.page.cursor.is_none());

    // The same request on the same snapshot produces the same sequence.
    let repeat = run_complete(&snapshot, &full_request);
    assert_eq!(repeat.items, full.items);
    assert_eq!(repeat.coverage.scanned_items, full.coverage.scanned_items);

    // One item per page reproduces the full scan exactly.
    let mut collected: Vec<XrefItem> = Vec::new();
    let mut cursor = None;
    let mut pages = 0;
    loop {
        let mut page_request = full_request.clone();
        page_request.max_items = 1;
        page_request.cursor = cursor.clone();
        let page = run_complete(&snapshot, &page_request);
        pages += 1;
        collected.extend(page.items.iter().cloned());
        match page.page.cursor.clone() {
            Some(next) => {
                assert!(page.page.has_more);
                cursor = Some(next);
            }
            None => {
                assert!(!page.page.has_more);
                break;
            }
        }
        assert!(pages < 32, "continuation must terminate");
    }
    assert_eq!(pages, full.items.len());
    assert_eq!(collected, full.items);
}

#[test]
fn tight_budgets_and_cancellation_keep_the_reliable_prefix() {
    // A request with several items, so a one-item budget really truncates it.
    let built = inner_nest_fixture();
    let snapshot = open(built.bytes.clone());
    let request = query_request(
        &snapshot,
        QueryRelation::MentionsSymbol,
        target_symbol("p/Outer"),
        &[ConsumerKind::InnerNest],
    );
    let full = run_complete(&snapshot, &request);
    assert!(full.items.len() >= 4);

    // One result item is published, the rest stays unpublished and unbilled.
    let mut tight = limits();
    tight.result_items = 1;
    let (report, usage) = query_with(&snapshot, &request, tight);
    assert_eq!(report.items, vec![full.items[0].clone()]);
    assert_eq!(usage.result_items, 1);
    assert!(report.page.has_more);
    assert!(report.page.cursor.is_some());
    assert!(matches!(report.execution, ExecutionReport::Partial { .. }));
    assert_ne!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_result_items")
    );

    // A budget that pays for the attribute shells but not for the annotation content
    // stops inside the decode pass.
    let built = meta_fixture();
    let snapshot = open(built.bytes.clone());
    let annotation_request = query_request(
        &snapshot,
        QueryRelation::MentionsSymbol,
        target_symbol("p/OnlyAnnotated"),
        &[ConsumerKind::Annotation],
    );
    let annotation_bytes = built.attribute_bytes_of(&[b"RuntimeVisibleAnnotations"]);
    let mut tight = limits();
    tight.attribute_bytes = built.attribute_bytes() + annotation_bytes - 1;
    let (report, usage) = query_with(&snapshot, &annotation_request, tight);
    assert!(report.items.is_empty());
    assert_eq!(usage.result_items, 0);
    assert!(matches!(report.execution, ExecutionReport::Partial { .. }));
    assert_ne!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_attribute_bytes")
    );

    // A cancelled request reads nothing and never reports completion.
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = Engine::new()
        .query(&snapshot, &annotation_request, &mut budget)
        .expect("query must succeed");
    assert!(report.items.is_empty());
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(budget.usage().class_bytes, 0);
    assert_ne!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
}

#[test]
fn archive_entries_are_scanned_only_when_they_are_class_files() {
    let class = meta_fixture();
    let archive = zip(&[
        (b"META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\r\n"),
        (b"p/Meta.class", class.bytes.as_slice()),
        (b"p/Copied.txt", b"not a class file at all"),
        (b"docs/readme.txt", b"Lp/OnlyAnnotated;"),
    ]);
    let snapshot = open(archive);

    let report = run_complete(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyAnnotated"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    let (definition, _) = class_offset(&report.items[0]);
    let entry = definition
        .location
        .entry()
        .expect("an archive class keeps its entry identity");
    assert_eq!(entry.raw_name, ArchiveNameBytes(b"p/Meta.class".to_vec()));

    // Names outside the candidate range are never read, even when an entry carries the
    // target bytes as plain text. The request names a category only this consumer
    // scans, so the entry bytes below are its own read: a category that other consumer
    // streams also own would materialize the same unit once per stream.
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlySigned"),
            &[ConsumerKind::Signature],
        ),
        limits(),
    );
    assert_eq!(operations(&report), vec![XrefOperation::GenericSignature]);
    assert_complete(&report);
    assert_eq!(
        usage.entry_bytes,
        u64::try_from(class.bytes.len()).unwrap(),
        "only the class candidate is materialized"
    );

    // The same bytes behind a `.class` name are a candidate whose magic is missing:
    // the entry is located, the reliable prefix stays published and the scan does not
    // claim completion.
    let damaged = zip(&[
        (b"p/Meta.class", class.bytes.as_slice()),
        (b"p/Copied.class", b"not a class file at all"),
        (b"docs/readme.txt", b"Lp/OnlyAnnotated;"),
    ]);
    let snapshot = open(damaged);
    let report = run(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyAnnotated"),
            &[ConsumerKind::Annotation],
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    assert_eq!(
        failed_code(&report),
        Some("query_class_candidate_malformed")
    );
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        xref_ranges(&report.coverage.dimensions.artifact_structural.skipped),
        vec![(2, 3)],
        "the sibling behind the stop is declared skipped"
    );
    assert!(report.page.has_more);
}

/// Error code of a `Failed { Error }` execution.
fn failed_code(report: &QueryReport) -> Option<&str> {
    match &report.execution {
        ExecutionReport::Failed {
            reason: TerminationReason::Error { code },
            ..
        } => Some(code.as_str()),
        _ => None,
    }
}

/// `container:root:xref_scan_entries` ranges of one structural dimension side.
fn xref_ranges(ranges: &[CoverageRange]) -> Vec<(u64, u64)> {
    ranges
        .iter()
        .filter(|range| range.label == "container:root:xref_scan_entries")
        .map(|range| (range.start, range.end))
        .collect()
}

#[test]
fn a_truncated_class_candidate_fails_with_its_entry_origin() {
    let class = meta_fixture();
    let archive = zip(&[
        (b"p/Meta.class", class.bytes.as_slice()),
        (b"p/Broken.class", &[0xca, 0xfe, 0xba]),
        (b"p/Other.class", class.bytes.as_slice()),
    ]);
    let snapshot = open(archive);
    let report = run(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyAnnotated"),
            &[ConsumerKind::Annotation],
        ),
    );

    // The first candidate keeps its facts, and the damaged one is located by entry.
    assert_eq!(operations(&report), vec![XrefOperation::Annotation]);
    assert_eq!(
        failed_code(&report),
        Some("query_class_candidate_malformed")
    );
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "query_class_candidate_malformed")
        .expect("the damaged candidate is reported");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    match diagnostic
        .provenance
        .as_ref()
        .map(|provenance| &provenance.location)
    {
        Some(Location::Entry { id, span }) => {
            assert_eq!(id.raw_name.0, b"p/Broken.class");
            assert_eq!(id.ordinal, 1);
            assert_eq!(*span, ByteSpan::new(0, 3));
        }
        other => panic!("expected the damaged entry as the diagnostic origin, got {other:?}"),
    }
    assert!(
        diagnostic.message.contains("Broken.class"),
        "{diagnostic:?}"
    );
    assert!(diagnostic.message.contains('3'), "{diagnostic:?}");
    assert!(diagnostic.message.contains("CA FE BA BE"), "{diagnostic:?}");
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        xref_ranges(&report.coverage.dimensions.artifact_structural.skipped),
        vec![(2, 3)]
    );
    assert!(report.page.has_more);
}

#[test]
fn the_class_candidate_rule_is_case_sensitive_and_shared() {
    let class = meta_fixture();
    let garbage = b"this entry is not a class file";
    // The upper-case name holds the *same legal class bytes* as the lower-case one: a
    // case-folding rule would report its items a second time and materialize it, so the
    // assertions below are falsified by that mistake instead of agreeing with it.
    let archive = zip(&[
        (b"p/Good.class", class.bytes.as_slice()),
        (b".class", class.bytes.as_slice()),
        (b"p/Good.CLASS", class.bytes.as_slice()),
        (b"docs/readme.txt", garbage),
    ]);
    let snapshot = open(archive);
    let (report, usage) = query_with(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target_symbol("p/OnlyAnnotated"),
            &[ConsumerKind::Annotation],
        ),
        limits(),
    );

    // The lower-case name and the bare `.class` are both candidates, and the metadata
    // consumer that used to fold case and to require a longer name now agrees with the
    // other class scanners.
    assert_eq!(
        operations(&report),
        vec![XrefOperation::Annotation, XrefOperation::Annotation]
    );
    let entry_names = report
        .items
        .iter()
        .map(|item| {
            let (definition, _) = class_offset(item);
            definition
                .location
                .entry()
                .expect("an archive class keeps its entry identity")
                .raw_name
                .0
                .clone()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        entry_names,
        vec![b"p/Good.class".to_vec(), b".class".to_vec()]
    );
    // The upper-case name and the plain resource stay outside the range: even though the
    // upper-case entry is a legal class file, it produces no items and no damage
    // diagnostic.
    assert_complete(&report);
    assert_eq!(
        usage.entry_bytes,
        2 * u64::try_from(class.bytes.len()).unwrap(),
        "only the two candidates are materialized"
    );
}

fn zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for &(name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}
