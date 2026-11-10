//! Java literals over supplied classfile metadata. No artifact reads, resolver or binding authority.
//! Valid JVM strings retain UTF-16 units; source identifiers require scalar Java spelling.
use crate::emit::char_literal;
use crate::names::SourceClassNames;
use crate::report::{
    ClassAnnotationAttribute, MemberAnnotationUses, ParameterAnnotationUses, TypeAnnotationUses,
};
use jarde_reader::classfile::{
    Base, ClassMemberFacts, CpEntryFacts, CpEntryKind, DescriptorKind, ElementConstantTag,
    ElementValueFacts, ElementValuePairFacts, cp_entry, descriptor_facts,
};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{JvmBytes, JvmString};

fn jvm_text(raw: &JvmBytes) -> Result<JvmString> {
    JvmString::from_mutf8(raw.0.clone()).ok_or_else(|| {
        Error::invalid_input(
            "classfile_modified_utf8_invalid",
            "metadata literal or identifier has invalid Modified UTF-8",
        )
    })
}
fn identifier(raw: &JvmBytes) -> Result<Option<String>> {
    Ok(jvm_text(raw)?
        .to_unicode()
        .ok()
        .filter(|name| crate::is_java_identifier(name)))
}
fn descriptor_type(
    raw: &[u8],
    kind: DescriptorKind,
    names: SourceClassNames<'_>,
) -> Result<Option<String>> {
    jvm_text(&JvmBytes(raw.to_vec()))?;
    let Ok(facts) = descriptor_facts(raw, kind) else {
        return Ok(None);
    };
    let Some(component) = facts.single() else {
        return Ok(None);
    };
    if let Some(raw) = component.object_name() {
        return Ok(names
            .class_name(raw)
            .map(|name| format!("{name}{}", "[]".repeat(component.dimensions() as usize))));
    }
    Ok(crate::type_of_component(component).map(|ty| ty.spell().to_owned()))
}
fn annotation_type_name(raw: &[u8], names: SourceClassNames<'_>) -> Result<Option<String>> {
    let name = descriptor_type(raw, DescriptorKind::Field, names)?;
    if !raw.starts_with(b"L") || !raw.ends_with(b";") {
        return Ok(None);
    }
    Ok(name.filter(|name| name.split('.').all(crate::is_java_identifier)))
}

fn unspellable(attribute: &str) -> Error {
    Error::unsupported(
        "metadata_literal_unspellable",
        format!("{attribute} has no supported complete Java literal"),
    )
}

/// Resolve an existing ConstantValue. Source inapplicability is an explicit refusal;
/// malformed input remains an error, never a replacement-valued or silently absent literal.
pub fn resolve_constant_value(
    descriptor: &[u8],
    index: u16,
    pool: &[CpEntryFacts],
) -> Result<MemberDefault> {
    let entry = cp_entry(pool, index)?;
    let resolved = match (descriptor, &entry.kind) {
        ([b'I'], CpEntryKind::Integer { value }) => Some(MemberDefault::Integer(*value)),
        ([b'J'], CpEntryKind::Long { value }) => Some(MemberDefault::Long(*value)),
        ([b'B'], CpEntryKind::Integer { value }) => Some(MemberDefault::Byte(*value)),
        ([b'S'], CpEntryKind::Integer { value }) => Some(MemberDefault::Short(*value)),
        ([b'C'], CpEntryKind::Integer { value }) => {
            u16::try_from(*value).ok().map(MemberDefault::Char)
        }
        ([b'Z'], CpEntryKind::Integer { value }) => match value {
            0 => Some(MemberDefault::Boolean(false)),
            1 => Some(MemberDefault::Boolean(true)),
            _ => None,
        },
        (b"Ljava/lang/String;", CpEntryKind::String { value, .. }) => {
            Some(MemberDefault::String(jvm_text(value)?))
        }
        _ => None,
    };
    resolved.ok_or_else(|| unspellable("ConstantValue"))
}

/// One declaration's literal, as the class file's own attribute states it.
///
/// Two attributes state a literal this presentation writes, and both state it as one of these values:
/// a member's `AnnotationDefault` (JVMS 4.7.22), written after the `default` keyword of its
/// declaration, and a field's `ConstantValue` (JVMS 4.7.2), written as that field's initializer. The
/// value is the attribute's own constant with the pool entries its indexes name resolved.
/// Only a complete supported value is constructed: an unsupported child rejects the whole
/// array or annotation. Field F/D constants remain outside the current initializer domain;
/// their resolver returns an explicit refusal rather than implying attribute absence.
///
/// The two paths share this one vocabulary so they spell one constant one way: `3L` is the same
/// spelling whichever attribute states it, and a string is escaped by the one rule this engine
/// escapes string literals with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberDefault {
    /// `B`: a `byte`, written as the decimal integer it is.
    Byte(i32),
    /// `C`: a `char`, written as the character literal of its UTF-16 code unit.
    Char(u16),
    /// `I`: an `int`, written as the decimal integer it is.
    Integer(i32),
    /// `S`: a `short`, written as the decimal integer it is.
    Short(i32),
    /// `Z`: a `boolean`, written as `true` or `false`.
    Boolean(bool),
    /// `J`: a `long`, written as the decimal integer it is with the `L` its type needs.
    Long(i64),
    /// `F`/`D`: an annotation constant's raw IEEE bits, never used for a field initializer.
    Float(u32),
    Double(u64),
    /// `s`: a string, written as the Java string literal its text is.
    String(JvmString),
    /// `e`: an enum constant, written as its type's name, a dot, and the constant's own simple name.
    Enum {
        type_name: String,
        constant_name: String,
    },
    /// `c`: a class literal, written as its type's name and `.class`.
    Class {
        type_name: String,
    },
    /// `@`: a nested annotation with its type and explicitly named values in attribute order.
    Annotation {
        type_name: String,
        elements: Vec<(String, MemberDefault)>,
    },
    /// `[`: the elements the array declares, written `{ elem, elem }`.
    Array(Vec<MemberDefault>),
}

impl MemberDefault {
    /// The literal this default is written as, after the `default` keyword.
    pub fn spelling(&self) -> String {
        match self {
            Self::Byte(value) | Self::Integer(value) | Self::Short(value) => value.to_string(),
            Self::Boolean(value) => if *value { "true" } else { "false" }.to_owned(),
            Self::Long(value) => format!("{value}L"),
            Self::Float(bits) => float_default_spelling(*bits)
                .expect("only a faithfully spellable annotation float is resolved"),
            Self::Double(bits) => double_default_spelling(*bits)
                .expect("only a faithfully spellable annotation double is resolved"),
            Self::Char(value) => char_literal(*value),
            Self::String(value) => crate::emit::jvm_string_literal(value),
            Self::Enum {
                type_name,
                constant_name,
            } => format!("{type_name}.{constant_name}"),
            Self::Class { type_name } => format!("{type_name}.class"),
            Self::Annotation {
                type_name,
                elements,
            } => {
                let elements: Vec<String> = elements
                    .iter()
                    .map(|(name, value)| format!("{name} = {}", value.spelling()))
                    .collect();
                format!("@{type_name}({})", elements.join(", "))
            }
            Self::Array(elements) if elements.is_empty() => "{}".to_owned(),
            Self::Array(elements) => {
                let written: Vec<String> = elements.iter().map(Self::spelling).collect();
                format!("{{{}}}", written.join(", "))
            }
        }
    }
}

/// Resolve an existing AnnotationDefault atomically. Absence belongs to the attribute reader;
/// a present but unsupported value is a refusal, while damaged encoding retains its own error.
pub fn resolve_default(
    value: &ElementValueFacts,
    pool: &[CpEntryFacts],
    names: SourceClassNames<'_>,
) -> Result<MemberDefault> {
    resolve_value(value, pool, names)?.ok_or_else(|| unspellable("AnnotationDefault"))
}

fn resolve_value(
    value: &ElementValueFacts,
    pool: &[CpEntryFacts],
    names: SourceClassNames<'_>,
) -> Result<Option<MemberDefault>> {
    let resolved = match value {
        ElementValueFacts::Constant { tag, index } => {
            let Ok(entry) = cp_entry(pool, index.0) else {
                return Ok(None);
            };
            match (tag, &entry.kind) {
                (ElementConstantTag::Byte, CpEntryKind::Integer { value }) => {
                    Some(MemberDefault::Byte(*value))
                }
                (ElementConstantTag::Char, CpEntryKind::Integer { value }) => {
                    u16::try_from(*value).ok().map(MemberDefault::Char)
                }
                (ElementConstantTag::Integer, CpEntryKind::Integer { value }) => {
                    Some(MemberDefault::Integer(*value))
                }
                (ElementConstantTag::Short, CpEntryKind::Integer { value }) => {
                    Some(MemberDefault::Short(*value))
                }
                (ElementConstantTag::Boolean, CpEntryKind::Integer { value }) => match value {
                    0 => Some(MemberDefault::Boolean(false)),
                    1 => Some(MemberDefault::Boolean(true)),
                    _ => None,
                },
                (ElementConstantTag::Long, CpEntryKind::Long { value }) => {
                    Some(MemberDefault::Long(*value))
                }
                (ElementConstantTag::Float, CpEntryKind::Float { bits }) => {
                    float_default_spelling(*bits).map(|_| MemberDefault::Float(*bits))
                }
                (ElementConstantTag::Double, CpEntryKind::Double { bits }) => {
                    double_default_spelling(*bits).map(|_| MemberDefault::Double(*bits))
                }
                _ => None,
            }
        }
        ElementValueFacts::Utf8(raw) => Some(MemberDefault::String(jvm_text(raw)?)),
        ElementValueFacts::Enum {
            type_descriptor,
            constant_name,
        } => {
            let Some(type_name) = annotation_type_name(&type_descriptor.0, names)? else {
                return Ok(None);
            };
            let Some(constant_name) = identifier(constant_name)? else {
                return Ok(None);
            };
            Some(MemberDefault::Enum {
                type_name,
                constant_name,
            })
        }
        ElementValueFacts::Class(raw) => descriptor_type(&raw.0, DescriptorKind::Return, names)?
            .map(|type_name| MemberDefault::Class { type_name }),
        ElementValueFacts::Annotation {
            type_descriptor,
            elements,
        } => {
            let Some(type_name) = annotation_type_name(&type_descriptor.0, names)? else {
                return Ok(None);
            };
            let Some(elements) = resolve_elements(elements, pool, names)? else {
                return Ok(None);
            };
            Some(MemberDefault::Annotation {
                type_name,
                elements,
            })
        }
        ElementValueFacts::Array(values) => {
            let mut resolved = Vec::with_capacity(values.len());
            for value in values {
                let Some(value) = resolve_value(value, pool, names)? else {
                    return Ok(None);
                };
                resolved.push(value);
            }
            Some(MemberDefault::Array(resolved))
        }
    };
    Ok(resolved)
}
fn resolve_elements(
    elements: &[ElementValuePairFacts],
    pool: &[CpEntryFacts],
    names: SourceClassNames<'_>,
) -> Result<Option<Vec<(String, MemberDefault)>>> {
    let mut resolved = Vec::with_capacity(elements.len());
    for element in elements {
        let Some(name) = identifier(&element.name)? else {
            return Ok(None);
        };
        let Some(value) = resolve_value(&element.value, pool, names)? else {
            return Ok(None);
        };
        resolved.push((name, value));
    }
    Ok(Some(resolved))
}

/// The outer annotation omits empty parentheses; nested literal values retain their existing form.
pub fn spell_annotation_body(
    annotation: &ElementValueFacts,
    pool: &[CpEntryFacts],
    names: SourceClassNames<'_>,
) -> std::result::Result<String, String> {
    let ElementValueFacts::Annotation {
        type_descriptor,
        elements,
    } = annotation
    else {
        return Err("annotation entry is not an annotation fact".to_owned());
    };
    let Some(type_name) =
        annotation_type_name(&type_descriptor.0, names).map_err(|error| error.to_string())?
    else {
        return Err(format!(
            "annotation type `{}` is not a Java source name",
            String::from_utf8_lossy(&type_descriptor.0)
        ));
    };
    let Some(elements) =
        resolve_elements(elements, pool, names).map_err(|error| error.to_string())?
    else {
        return Err(format!(
            "annotation `@{type_name}` has an element with no faithful Java spelling"
        ));
    };
    if elements.is_empty() {
        Ok(format!("@{type_name}"))
    } else {
        Ok(format!(
            "@{type_name}({})",
            elements
                .iter()
                .map(|(name, value)| format!("{name} = {}", value.spelling()))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

/// Spell finite IEEE values as exact Java hexadecimal floating-point literals without converting
/// the raw facts through host floating-point arithmetic. Unsupported NaN patterns have no spelling.
fn float_default_spelling(bits: u32) -> Option<String> {
    let sign = if bits >> 31 == 0 { "" } else { "-" };
    let exponent = (bits >> 23) & 0xff;
    let fraction = bits & 0x7f_ffff;
    match (exponent, fraction) {
        (0xff, 0) => Some(format!(
            "Float.{}INFINITY",
            if sign.is_empty() {
                "POSITIVE_"
            } else {
                "NEGATIVE_"
            }
        )),
        (0xff, 0x40_0000) if sign.is_empty() => Some("Float.NaN".to_owned()),
        (0xff, _) => None,
        (0, 0) => Some(format!("{sign}0.0f")),
        (0, fraction) => Some(format!("{sign}0x0.{:06x}p-126f", fraction << 1)),
        _ => Some(format!(
            "{sign}0x1.{:06x}p{}f",
            fraction << 1,
            i32::try_from(exponent).unwrap() - 127
        )),
    }
}

fn double_default_spelling(bits: u64) -> Option<String> {
    let sign = if bits >> 63 == 0 { "" } else { "-" };
    let exponent = (bits >> 52) & 0x7ff;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    match (exponent, fraction) {
        (0x7ff, 0) => Some(format!(
            "Double.{}INFINITY",
            if sign.is_empty() {
                "POSITIVE_"
            } else {
                "NEGATIVE_"
            }
        )),
        (0x7ff, 0x0008_0000_0000_0000) if sign.is_empty() => Some("Double.NaN".to_owned()),
        (0x7ff, _) => None,
        (0, 0) => Some(format!("{sign}0.0d")),
        (0, fraction) => Some(format!("{sign}0x0.{fraction:013x}p-1022d")),
        _ => Some(format!(
            "{sign}0x1.{fraction:013x}p{}d",
            i32::try_from(exponent).unwrap() - 1023
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_unspellable_names_and_damaged_strings_are_distinct_atomic_failures() {
        let named = |name: &str| ElementValueFacts::Enum {
            type_descriptor: JvmBytes::from_unicode("Lp/Kind;"),
            constant_name: JvmBytes::from_unicode(name),
        };
        for name in ["bad-name", "class"] {
            assert!(matches!(
                resolve_default(
                    &ElementValueFacts::Array(vec![
                        ElementValueFacts::Utf8(JvmBytes::from_unicode("ok")),
                        named(name)
                    ]),
                    &[], SourceClassNames::default()
                ),
                Err(Error::Unsupported { code, .. }) if code == "metadata_literal_unspellable"
            ));
        }
        let lone = ElementValueFacts::Enum {
            type_descriptor: JvmBytes::from_unicode("Lp/Kind;"),
            constant_name: JvmBytes(vec![0xed, 0xa0, 0x80]),
        };
        assert!(
            matches!(resolve_default(&lone, &[], SourceClassNames::default()), Err(Error::Unsupported { code, .. }) if code == "metadata_literal_unspellable")
        );
        let damaged =
            ElementValueFacts::Array(vec![ElementValueFacts::Utf8(JvmBytes(vec![0xed, 0xa0]))]);
        assert!(
            matches!(resolve_default(&damaged,&[],SourceClassNames::default()),Err(Error::InvalidInput{code,..}) if code=="classfile_modified_utf8_invalid")
        );
        let wrong_enum = ElementValueFacts::Enum {
            type_descriptor: JvmBytes::from_unicode("[Lp/Kind;"),
            constant_name: JvmBytes::from_unicode("ONE"),
        };
        assert!(
            matches!(resolve_default(&wrong_enum, &[], SourceClassNames::default()), Err(Error::Unsupported { code, .. }) if code == "metadata_literal_unspellable")
        );
    }
}

/// Spell class annotations over supplied physical attributes; publication stays with the caller.
pub fn spell_class_annotation_uses(
    attributes: &[ClassAnnotationAttribute],
    pool: &[CpEntryFacts],
    source_names: SourceClassNames<'_>,
    uses: &mut Vec<String>,
    refusals: &mut Vec<String>,
) {
    let mut counts = std::collections::BTreeMap::<Vec<u8>, usize>::new();
    for attribute in attributes {
        for annotation in &attribute.annotations {
            if let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = annotation
            {
                *counts.entry(type_descriptor.0.clone()).or_default() += 1;
            }
        }
    }
    for attribute in attributes {
        for annotation in &attribute.annotations {
            match spell_class_annotation(annotation, pool, &counts, source_names) {
                Ok(spelling) => uses.push(spelling),
                Err(reason) => refusals.push(reason),
            }
        }
    }
}

fn spell_class_annotation(
    annotation: &ElementValueFacts,
    pool: &[CpEntryFacts],
    counts: &std::collections::BTreeMap<Vec<u8>, usize>,
    source_names: SourceClassNames<'_>,
) -> std::result::Result<String, String> {
    let ElementValueFacts::Annotation {
        type_descriptor, ..
    } = annotation
    else {
        return Err("class annotation entry is not an annotation fact".to_owned());
    };
    if counts.get(&type_descriptor.0).copied().unwrap_or(0) > 1 {
        return Err(format!(
            "duplicate annotation type `{}` is conservatively refused",
            String::from_utf8_lossy(&type_descriptor.0)
        ));
    }
    spell_annotation_body(annotation, pool, source_names)
}

#[derive(Clone, Copy)]
pub enum TypeUseOwner {
    Field,
    Method,
}

pub fn spell_type_annotation_uses(
    mut uses: TypeAnnotationUses,
    owner: TypeUseOwner,
    descriptor: &[u8],
    pool: &[CpEntryFacts],
    declarations: &MemberAnnotationUses,
    parameter_declarations: Option<&ParameterAnnotationUses>,
    parameter_count: Option<usize>,
    source_names: SourceClassNames<'_>,
) -> TypeAnnotationUses {
    uses.parameter_uses = vec![Vec::new(); parameter_count.unwrap_or_default()];
    let declaration_types = declarations
        .attributes
        .iter()
        .flat_map(|attribute| {
            attribute
                .annotations
                .iter()
                .filter_map(|annotation| match annotation {
                    ElementValueFacts::Annotation {
                        type_descriptor, ..
                    } => Some(type_descriptor.0.clone()),
                    _ => None,
                })
        })
        .collect::<std::collections::BTreeSet<_>>();
    let parameter_declaration_types = parameter_declarations.map(|parameters| {
        let mut by_position =
            vec![std::collections::BTreeSet::<Vec<u8>>::new(); parameter_count.unwrap_or_default()];
        for attribute in &parameters.attributes {
            if attribute.parameter_count.map(usize::from) != parameter_count {
                continue;
            }
            for (position, annotations) in attribute.parameters.iter().enumerate() {
                for annotation in annotations {
                    if let ElementValueFacts::Annotation {
                        type_descriptor, ..
                    } = annotation
                        && let Some(slot) = by_position.get_mut(position)
                    {
                        slot.insert(type_descriptor.0.clone());
                    }
                }
            }
        }
        by_position
    });
    let parameter_declarations_unmapped = parameter_declarations.is_some_and(|parameters| {
        parameters
            .attributes
            .iter()
            .any(|attribute| attribute.parameter_count.map(usize::from) != parameter_count)
    });
    let mut counts = std::collections::BTreeMap::<(u8, Vec<u8>, Vec<u8>), usize>::new();
    for attribute in &uses.attributes {
        for annotation in &attribute.annotations {
            if let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = &annotation.annotation
            {
                *counts
                    .entry((
                        annotation.target_type,
                        annotation.target_info.clone(),
                        type_descriptor.0.clone(),
                    ))
                    .or_default() += 1;
            }
        }
    }
    for attribute in &mut uses.attributes {
        for annotation in &mut attribute.annotations {
            let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = &annotation.annotation
            else {
                annotation.refusal =
                    Some("type annotation entry is not an annotation fact".to_owned());
                continue;
            };
            let position = match (
                owner,
                annotation.target_type,
                annotation.target_info.as_slice(),
            ) {
                (TypeUseOwner::Field, 0x13, []) => None,
                (TypeUseOwner::Method, 0x14, []) => None,
                (TypeUseOwner::Method, 0x16, [position]) => Some(usize::from(*position)),
                (TypeUseOwner::Field, 0x13, _) => {
                    annotation.refusal = Some("FIELD target_info must be empty".to_owned());
                    continue;
                }
                (TypeUseOwner::Method, 0x14, _) => {
                    annotation.refusal = Some("METHOD_RETURN target_info must be empty".to_owned());
                    continue;
                }
                (TypeUseOwner::Method, 0x16, _) => {
                    annotation.refusal = Some(
                        "METHOD_FORMAL_PARAMETER target_info must contain one index byte"
                            .to_owned(),
                    );
                    continue;
                }
                (TypeUseOwner::Field, _, _) => {
                    annotation.refusal =
                        Some("type annotation target does not belong to a field".to_owned());
                    continue;
                }
                (TypeUseOwner::Method, _, _) => {
                    annotation.refusal = Some(
                        "type annotation target is outside the supported method targets".to_owned(),
                    );
                    continue;
                }
            };
            if !annotation.type_path.is_empty() {
                annotation.refusal =
                    Some("non-empty type_path is not source-spellable in this change".to_owned());
                continue;
            }
            if let Some(position) = position
                && parameter_count.is_none_or(|count| position >= count)
            {
                annotation.refusal = Some(format!(
                    "formal parameter position {position} is outside the descriptor parameter count"
                ));
                continue;
            }
            if position.is_some() && parameter_declarations_unmapped {
                annotation.refusal = Some(
                    "parameter declaration annotations do not map to descriptor positions"
                        .to_owned(),
                );
                continue;
            }
            if counts
                .get(&(
                    annotation.target_type,
                    annotation.target_info.clone(),
                    type_descriptor.0.clone(),
                ))
                .copied()
                .unwrap_or(0)
                > 1
            {
                annotation.refusal = Some(
                    "duplicate annotation type at the same target is conservatively refused"
                        .to_owned(),
                );
                continue;
            }
            let declaration_conflict = match position {
                Some(position) => parameter_declaration_types
                    .as_ref()
                    .and_then(|by_position| by_position.get(position))
                    .is_some_and(|types| types.contains(&type_descriptor.0)),
                None => declaration_types.contains(&type_descriptor.0),
            };
            if declaration_conflict {
                annotation.refusal = Some(
                    "same annotation type is present on the declaration and type target".to_owned(),
                );
                continue;
            }
            let Ok(spelling) = spell_annotation_body(&annotation.annotation, pool, source_names)
            else {
                annotation.refusal =
                    Some("annotation value has no faithful Java source spelling".to_owned());
                continue;
            };
            if qualify_type_descriptor(descriptor, owner, position).is_none() {
                annotation.refusal = Some("target type is primitive, array, single-segment, or has ambiguous nested-class spelling".to_owned());
                continue;
            }
            annotation.spelling = Some(spelling.clone());
            match position {
                Some(position) => uses.parameter_uses[position].push(spelling),
                None if matches!(owner, TypeUseOwner::Field) => uses.field_uses.push(spelling),
                None => uses.return_uses.push(spelling),
            }
        }
    }
    for attribute in &uses.attributes {
        for annotation in &attribute.annotations {
            if let Some(reason) = &annotation.refusal {
                uses.refusals.push(format!(
                    "target=0x{:02x} info={:?} path={:?}: {reason}",
                    annotation.target_type, annotation.target_info, annotation.type_path
                ));
            }
        }
    }
    uses
}

fn qualify_type_descriptor(
    descriptor: &[u8],
    owner: TypeUseOwner,
    parameter: Option<usize>,
) -> Option<String> {
    let facts = descriptor_facts(
        descriptor,
        match owner {
            TypeUseOwner::Field => DescriptorKind::Field,
            TypeUseOwner::Method => DescriptorKind::Method,
        },
    )
    .ok()?;
    let component = match owner {
        TypeUseOwner::Field => facts.single()?,
        TypeUseOwner::Method => match parameter {
            Some(position) => facts.parameters().get(position)?,
            None => facts.result()?,
        },
    };
    if component.is_array() || !matches!(component.base(), Base::Object(_)) {
        return None;
    }
    let name = crate::type_of_component(component)?.spell().to_owned();
    let (package, simple) = name.rsplit_once('.')?;
    if simple.contains('$') || package.is_empty() {
        return None;
    }
    Some(format!("{package}.@{{annotation}} {simple}"))
}

pub fn spell_member_annotation_uses(
    mut annotations: MemberAnnotationUses,
    pool: &[CpEntryFacts],
    source_names: SourceClassNames<'_>,
) -> MemberAnnotationUses {
    let mut counts = std::collections::BTreeMap::<Vec<u8>, usize>::new();
    for attribute in &annotations.attributes {
        for annotation in &attribute.annotations {
            if let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = annotation
            {
                *counts.entry(type_descriptor.0.clone()).or_default() += 1;
            }
        }
    }
    for attribute in &annotations.attributes {
        for annotation in &attribute.annotations {
            match annotation {
                ElementValueFacts::Annotation {
                    type_descriptor, ..
                } if counts.get(&type_descriptor.0).copied().unwrap_or_default() > 1 => {
                    annotations.refusals.push(format!(
                        "duplicate annotation type `{}` at one member position is conservatively refused",
                        String::from_utf8_lossy(&type_descriptor.0)
                    ));
                }
                _ => match spell_annotation_body(annotation, pool, source_names) {
                    Ok(spelling) => annotations.uses.push(spelling),
                    Err(reason) => annotations.refusals.push(reason),
                },
            }
        }
    }
    annotations
}

pub fn spell_parameter_annotation_uses(
    mut annotations: ParameterAnnotationUses,
    pool: &[CpEntryFacts],
    expected_count: Option<usize>,
    source_names: SourceClassNames<'_>,
) -> ParameterAnnotationUses {
    let Some(expected_count) = expected_count else {
        for attribute in &annotations.attributes {
            if attribute.parameter_count.is_some() {
                annotations.refusals.push(format!(
                    "parameter annotation attribute `{}` has no descriptor positions to attach to",
                    String::from_utf8_lossy(&attribute.attribute.name.raw().0)
                ));
            }
        }
        return annotations;
    };
    annotations.uses_by_position = vec![Vec::new(); expected_count];
    let mut by_position = vec![Vec::<&ElementValueFacts>::new(); expected_count];
    for attribute in &annotations.attributes {
        let Some(parameter_count) = attribute.parameter_count else {
            continue;
        };
        if usize::from(parameter_count) != expected_count
            || attribute.parameters.len() != expected_count
        {
            annotations.refusals.push(format!(
                "parameter annotation attribute `{}` declares {parameter_count} position(s) for a descriptor with {expected_count} parameter(s); its groups are not aligned",
                String::from_utf8_lossy(&attribute.attribute.name.raw().0)
            ));
            continue;
        }
        for (position, group) in attribute.parameters.iter().enumerate() {
            by_position[position].extend(group);
        }
    }
    for (position, entries) in by_position.into_iter().enumerate() {
        let mut counts = std::collections::BTreeMap::<Vec<u8>, usize>::new();
        for annotation in &entries {
            if let ElementValueFacts::Annotation {
                type_descriptor, ..
            } = annotation
            {
                *counts.entry(type_descriptor.0.clone()).or_default() += 1;
            }
        }
        for annotation in entries {
            match annotation {
                ElementValueFacts::Annotation {
                    type_descriptor, ..
                } if counts.get(&type_descriptor.0).copied().unwrap_or_default() > 1 => {
                    annotations.refusals.push(format!(
                        "parameter position {position}: duplicate annotation type `{}` is conservatively refused",
                        String::from_utf8_lossy(&type_descriptor.0)
                    ));
                }
                _ => match spell_annotation_body(annotation, pool, source_names) {
                    Ok(spelling) => annotations.uses_by_position[position].push(spelling),
                    Err(reason) => annotations
                        .refusals
                        .push(format!("parameter position {position}: {reason}")),
                },
            }
        }
    }
    annotations
}

pub(crate) fn refuse_uses(annotations: &mut MemberAnnotationUses, reason: &str) {
    for spelling in annotations.uses.drain(..) {
        annotations.refusals.push(format!("{reason}: {spelling}"));
    }
}

pub(crate) fn refuse_parameter_uses(annotations: &mut ParameterAnnotationUses, reason: &str) {
    for (position, uses) in annotations.uses_by_position.iter_mut().enumerate() {
        for spelling in uses.drain(..) {
            annotations.refusals.push(format!(
                "parameter position {position}: {reason}: {spelling}"
            ));
        }
    }
}

/// The narrow Java 8 package-info source certificate over supplied selected-read facts.
/// The caller owns attribute reads, execution, output charging and publication.
pub fn proved_package_info_source(
    facts: &ClassMemberFacts,
    class_version: (u16, u16),
    structure_complete: bool,
    execution: &jarde_reader::model::ExecutionReport,
    annotation_attributes: &[ClassAnnotationAttribute],
    annotation_uses: &[String],
    annotation_refusals: &[String],
) -> std::result::Result<String, &'static str> {
    let raw_name = &facts.this_class.raw().0;
    let Some(package_bytes) = raw_name.strip_suffix(b"/package-info") else {
        return Err("internal name is not a package path ending in package-info");
    };
    if !crate::names::legal_package_name(package_bytes) {
        return Err("package path is not a legal non-empty Java package name");
    }
    if class_version != (52, 0) || facts.access_flags != 0x1600 {
        return Err(
            "classfile version or access flags are not the standard Java 8 package-info shape",
        );
    }
    if facts
        .super_class
        .as_ref()
        .map(|name| name.raw().0.as_slice())
        != Some(b"java/lang/Object".as_slice())
        || !facts.interfaces.is_empty()
        || facts.field_count != 0
        || facts.method_count != 0
        || facts.stopped_at.is_some()
        || !structure_complete
    {
        return Err("class head or empty member tables are not completely proved");
    }
    if !matches!(
        execution,
        jarde_reader::model::ExecutionReport::Complete { .. }
    ) {
        return Err("class-source execution did not complete");
    }
    if facts.attributes.len() != 1
        || facts.attributes[0].name.raw().0 != b"RuntimeVisibleAnnotations"
        || annotation_attributes.len() != 1
        || annotation_attributes[0].attribute.name.raw().0 != b"RuntimeVisibleAnnotations"
        || !annotation_refusals.is_empty()
    {
        return Err("only one complete RuntimeVisibleAnnotations attribute is supported");
    }
    let annotation_count = annotation_attributes
        .iter()
        .map(|attribute| attribute.annotations.len())
        .sum::<usize>();
    if annotation_count != 1 || annotation_count != annotation_uses.len() {
        return Err("exactly one complete writable package annotation is required");
    }
    if !matches!(
        &annotation_attributes[0].annotations[0],
        ElementValueFacts::Annotation { type_descriptor, elements }
            if type_descriptor.0 == b"Ljava/lang/Deprecated;" && elements.is_empty()
    ) {
        return Err("the only supported package annotation is empty java.lang.Deprecated");
    }
    let package = crate::names::source_class_name(&JvmBytes(package_bytes.to_vec()))
        .ok_or("package path has no faithful Java source spelling")?;
    let mut text = String::new();
    for annotation in annotation_uses {
        text.push_str(annotation);
        text.push('\n');
    }
    text.push_str(&format!("package {package};\n"));
    Ok(text)
}
