//! Bounded parsing of the generic `Signature` grammar in JVMS 4.7.9.1.
//!
//! A class file can carry a legal Signature whose shape is too rich for a particular source
//! projection. This module preserves the full parsed shape and class-reference order so each
//! consumer can make that decision without implementing a second grammar.

use crate::budget::{Budget, CountedBudgetDimension};
use crate::classfile::{DescriptorKind, descriptor_facts};
use crate::error::{Error, Result};
use std::collections::HashSet;

const SIGNATURE_CODE: &str = "jvm_signature_malformed";
const MAX_SIGNATURE_BYTES: usize = u16::MAX as usize;
const MAX_SIGNATURE_DEPTH: usize = 128;
const MAX_SIGNATURE_NODES: usize = 4096;

/// A parsed class `Signature` attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub superclass: ClassType,
    pub interfaces: Vec<ClassType>,
}

impl ClassSignature {
    /// Class names in first-appearance order, with duplicates removed.
    pub fn class_references(&self, budget: &mut Budget) -> Result<Vec<Vec<u8>>> {
        let mut references = Vec::new();
        let mut seen = HashSet::new();
        references_from_parameters(&self.type_parameters, &mut references, &mut seen, budget)?;
        references_from_type(
            &SignatureType::Class(self.superclass.clone()),
            &mut references,
            &mut seen,
            budget,
        )?;
        for interface in &self.interfaces {
            references_from_type(
                &SignatureType::Class(interface.clone()),
                &mut references,
                &mut seen,
                budget,
            )?;
        }
        Ok(references)
    }
}

/// A parsed field `Signature` attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldSignature {
    pub ty: SignatureType,
}

impl FieldSignature {
    /// Class names in first-appearance order, with duplicates removed.
    pub fn class_references(&self, budget: &mut Budget) -> Result<Vec<Vec<u8>>> {
        let mut references = Vec::new();
        references_from_type(&self.ty, &mut references, &mut HashSet::new(), budget)?;
        Ok(references)
    }
}

/// A parsed method `Signature` attribute. `None` is a `void` result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<SignatureType>,
    pub result: Option<SignatureType>,
    pub throws: Vec<SignatureType>,
}

impl MethodSignature {
    /// Class names in first-appearance order, with duplicates removed.
    pub fn class_references(&self, budget: &mut Budget) -> Result<Vec<Vec<u8>>> {
        let mut references = Vec::new();
        let mut seen = HashSet::new();
        references_from_parameters(&self.type_parameters, &mut references, &mut seen, budget)?;
        for parameter in &self.parameters {
            references_from_type(parameter, &mut references, &mut seen, budget)?;
        }
        if let Some(result) = &self.result {
            references_from_type(result, &mut references, &mut seen, budget)?;
        }
        for exception in &self.throws {
            references_from_type(exception, &mut references, &mut seen, budget)?;
        }
        Ok(references)
    }
}

/// A method or class type parameter and its declared bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParameter {
    pub name: Vec<u8>,
    /// `None` encodes the empty class-bound slot (`T::Interface`).
    pub class_bound: Option<SignatureType>,
    pub interface_bounds: Vec<SignatureType>,
}

/// A type in the generic Signature grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignatureType {
    Base(u8),
    Array(Box<SignatureType>),
    TypeVariable(Vec<u8>),
    Class(ClassType),
}

/// A class type, retaining source-level nested segments and their own arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassType {
    pub segments: Vec<ClassTypeSegment>,
}

/// One outer or nested class segment. `binary_name` uses `/` for packages and `$` for the
/// segment's binary nesting path, matching the class name emitted by the query xref consumer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassTypeSegment {
    pub binary_name: Vec<u8>,
    pub arguments: Vec<TypeArgument>,
}

/// One generic type argument, including all wildcard forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeArgument {
    Any,
    Exact(SignatureType),
    Extends(SignatureType),
    Super(SignatureType),
}

/// Parse a class `Signature` attribute.
pub fn parse_class_signature(bytes: &[u8], budget: &mut Budget) -> Result<ClassSignature> {
    let mut parser = Parser::new(bytes, budget)?;
    parser.node()?;
    let type_parameters = parser.type_parameters(0)?;
    let superclass = parser.class_type(0)?;
    let mut interfaces = Vec::new();
    while !parser.reader.is_empty() {
        parser.node()?;
        interfaces.push(parser.class_type(0)?);
    }
    parser.reader.expect_end()?;
    Ok(ClassSignature {
        type_parameters,
        superclass,
        interfaces,
    })
}

/// Parse a field `Signature` attribute.
pub fn parse_field_signature(bytes: &[u8], budget: &mut Budget) -> Result<FieldSignature> {
    let mut parser = Parser::new(bytes, budget)?;
    parser.node()?;
    let ty = parser.reference_type(0)?;
    parser.reader.expect_end()?;
    Ok(FieldSignature { ty })
}

/// Parse a method `Signature` attribute.
pub fn parse_method_signature(bytes: &[u8], budget: &mut Budget) -> Result<MethodSignature> {
    let mut parser = Parser::new(bytes, budget)?;
    parser.node()?;
    let type_parameters = parser.type_parameters(0)?;
    parser.reader.expect_byte(b'(')?;
    let mut parameters = Vec::new();
    while parser.reader.peek() != Some(b')') {
        parameters.push(parser.java_type(0)?);
    }
    parser.reader.expect_byte(b')')?;
    let result = if parser.reader.peek() == Some(b'V') {
        parser.reader.skip(1)?;
        None
    } else {
        Some(parser.java_type(0)?)
    };
    let mut throws = Vec::new();
    while parser.reader.peek() == Some(b'^') {
        parser.reader.skip(1)?;
        match parser.reader.peek() {
            Some(b'L') => throws.push(parser.class_type_signature(0)?),
            Some(b'T') => throws.push(parser.type_variable(0)?),
            _ => {
                return Err(parser
                    .reader
                    .malformed("a throws signature must name a class type or a type variable"));
            }
        }
    }
    parser.reader.expect_end()?;
    Ok(MethodSignature {
        type_parameters,
        parameters,
        result,
        throws,
    })
}

/// Erasure facts proven for one method Signature against its physical descriptor and
/// `Exceptions` attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodSignatureErasureProof {
    pub type_parameters: Vec<TypeParameterErasure>,
    /// One erased descriptor type per method parameter, in declaration order.
    pub parameters: Vec<Vec<u8>>,
    /// `None` is `void`.
    pub result: Option<Vec<u8>>,
    /// One erased internal class name per generic throws type.
    pub throws: Vec<Vec<u8>>,
}

/// The first-bound erasure established for one class- or method-level type variable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParameterErasure {
    pub name: Vec<u8>,
    pub descriptor: Vec<u8>,
}

/// Class-level variable scope and hierarchy erasures proven against a physical class header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSignatureErasureProof {
    pub type_parameters: Vec<TypeParameterErasure>,
    pub superclass: Vec<u8>,
    pub interfaces: Vec<Vec<u8>>,
}

/// Prove that a field Signature is closed over an already-proven class scope and erases to its
/// complete physical field descriptor. This establishes only a classfile fact; it does not prove
/// that the generic type can be written in Java source or that field uses remain source-compatible.
pub fn prove_field_signature_erasure_with_class_scope(
    signature: &FieldSignature,
    descriptor: &[u8],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<()> {
    validate_type_variables(&signature.ty, &[], class_scope, budget)?;
    let physical = descriptor_facts(descriptor, DescriptorKind::Field)?;
    step(budget)?;
    let erased = erase_type(
        &signature.ty,
        &[],
        &[],
        class_scope,
        &mut Vec::new(),
        budget,
    )?;
    let Some(descriptor_type) = physical.single() else {
        return erasure_mismatch("field");
    };
    let Some(descriptor_bytes) = descriptor_type.bytes(descriptor) else {
        return erasure_mismatch("field");
    };
    if erased.as_slice() != descriptor_bytes {
        return erasure_mismatch("field");
    }
    Ok(())
}

/// Prove that a class Signature declares a unique, closed type-variable scope and describes the
/// physical superclass and interfaces in the same order. This is a classfile fact proof; it does
/// not prove Java source spellings or referenced-class availability.
pub fn prove_class_signature_erasure(
    signature: &ClassSignature,
    superclass: &[u8],
    interfaces: &[Vec<u8>],
    budget: &mut Budget,
) -> Result<ClassSignatureErasureProof> {
    let mut variables = Vec::with_capacity(signature.type_parameters.len());
    for parameter in &signature.type_parameters {
        step(budget)?;
        variables.push(parameter.name.clone());
    }
    for (index, parameter) in signature.type_parameters.iter().enumerate() {
        step(budget)?;
        if variables[..index]
            .iter()
            .any(|name| name == &parameter.name)
        {
            return scope_refused(&format!(
                "duplicate class type parameter `{}`",
                String::from_utf8_lossy(&parameter.name)
            ));
        }
        if let Some(bound) = &parameter.class_bound {
            validate_type_variables(bound, &variables, &[], budget)?;
        }
        for bound in &parameter.interface_bounds {
            validate_type_variables(bound, &variables, &[], budget)?;
        }
    }

    let mut type_parameters = Vec::with_capacity(signature.type_parameters.len());
    for index in 0..signature.type_parameters.len() {
        let descriptor = erase_type_parameter(
            index,
            &signature.type_parameters,
            &variables,
            &[],
            &mut Vec::new(),
            budget,
        )?;
        type_parameters.push(TypeParameterErasure {
            name: signature.type_parameters[index].name.clone(),
            descriptor,
        });
    }

    validate_type_variables(
        &SignatureType::Class(signature.superclass.clone()),
        &variables,
        &[],
        budget,
    )?;
    for interface in &signature.interfaces {
        validate_type_variables(
            &SignatureType::Class(interface.clone()),
            &variables,
            &[],
            budget,
        )?;
    }

    step(budget)?;
    let signature_superclass = class_internal_name(&signature.superclass)?;
    if signature_superclass != superclass {
        return erasure_mismatch("superclass");
    }
    if signature.interfaces.len() != interfaces.len() {
        return erasure_mismatch("interface count");
    }
    let mut proven_interfaces = Vec::with_capacity(signature.interfaces.len());
    for (index, (interface, physical)) in signature.interfaces.iter().zip(interfaces).enumerate() {
        step(budget)?;
        let erased = class_internal_name(interface)?;
        if erased != *physical {
            return erasure_mismatch(&format!("interface {index}"));
        }
        proven_interfaces.push(erased);
    }

    Ok(ClassSignatureErasureProof {
        type_parameters,
        superclass: signature_superclass,
        interfaces: proven_interfaces,
    })
}

fn class_internal_name(class: &ClassType) -> Result<Vec<u8>> {
    let Some(segment) = class.segments.last() else {
        return scope_refused("class type has no segments");
    };
    Ok(segment.binary_name.clone())
}

/// Prove a parsed method Signature's variable scope and JVM erasure against its physical
/// descriptor and `Exceptions` attribute. This is a reader fact proof only: it intentionally does
/// not decide whether the source presenter can spell the full generic shape, whether type-use
/// annotations or varargs can be placed, or whether call-site overload binding survives.
///
/// Type variables resolve first in this method's formal parameter list, which shadows a
/// same-named entry in the optional already-proven class scope. Bounds erase by the JVMS
/// first-bound rule: the class bound when present, otherwise the first interface bound, otherwise
/// `Object`. Every parameter and result is compared with the exact component bytes in the
/// descriptor. `exceptions` contains internal names from the same member's `Exceptions`
/// attribute, in attribute order. An absent optional throws suffix does not constrain the
/// `Exceptions` attribute; when present, its erasures must match in order.
pub fn prove_method_signature_erasure(
    signature: &MethodSignature,
    descriptor: &[u8],
    exceptions: &[Vec<u8>],
    budget: &mut Budget,
) -> Result<MethodSignatureErasureProof> {
    prove_method_signature_erasure_with_class_scope(signature, descriptor, exceptions, &[], budget)
}

/// As [`prove_method_signature_erasure`], with a class-variable scope already proven by
/// [`prove_class_signature_erasure`]. Method variables form an inner lexical scope and shadow
/// same-named class variables; no unresolved name is inferred from its bound or use site.
pub fn prove_method_signature_erasure_with_class_scope(
    signature: &MethodSignature,
    descriptor: &[u8],
    exceptions: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<MethodSignatureErasureProof> {
    let mut variables = Vec::with_capacity(signature.type_parameters.len());
    for parameter in &signature.type_parameters {
        step(budget)?;
        if variables
            .iter()
            .any(|name: &Vec<u8>| name == &parameter.name)
        {
            return scope_refused(&format!(
                "duplicate method type parameter `{}`",
                String::from_utf8_lossy(&parameter.name)
            ));
        }
        variables.push(parameter.name.clone());
    }

    for parameter in &signature.type_parameters {
        if let Some(bound) = &parameter.class_bound {
            validate_type_variables(bound, &variables, class_scope, budget)?;
        }
        for bound in &parameter.interface_bounds {
            validate_type_variables(bound, &variables, class_scope, budget)?;
        }
    }
    for parameter in &signature.parameters {
        validate_type_variables(parameter, &variables, class_scope, budget)?;
    }
    if let Some(result) = &signature.result {
        validate_type_variables(result, &variables, class_scope, budget)?;
    }
    for exception in &signature.throws {
        validate_type_variables(exception, &variables, class_scope, budget)?;
    }

    // Install the entire local scope before following bounds, then detect cycles while deriving
    // first-bound erasures. This preserves the grammar's distinction between a name and a class.
    let mut type_parameter_erasures = Vec::with_capacity(signature.type_parameters.len());
    for index in 0..signature.type_parameters.len() {
        let descriptor = erase_type_parameter(
            index,
            &signature.type_parameters,
            &variables,
            class_scope,
            &mut Vec::new(),
            budget,
        )?;
        type_parameter_erasures.push(TypeParameterErasure {
            name: signature.type_parameters[index].name.clone(),
            descriptor,
        });
    }

    let physical = descriptor_facts(descriptor, DescriptorKind::Method)?;
    if physical.parameters().len() != signature.parameters.len() {
        return erasure_mismatch("parameter count");
    }

    let mut parameters = Vec::with_capacity(signature.parameters.len());
    for (position, (generic, descriptor_component)) in signature
        .parameters
        .iter()
        .zip(physical.parameters())
        .enumerate()
    {
        step(budget)?;
        let erased = erase_type(
            generic,
            &signature.type_parameters,
            &variables,
            class_scope,
            &mut Vec::new(),
            budget,
        )?;
        let Some(descriptor_bytes) = descriptor_component.bytes(descriptor) else {
            return erasure_mismatch(&format!("parameter {position}"));
        };
        if erased.as_slice() != descriptor_bytes {
            return erasure_mismatch(&format!("parameter {position}"));
        }
        parameters.push(erased);
    }

    let result = match (&signature.result, physical.result()) {
        (None, None) => None,
        (Some(generic), Some(descriptor_component)) => {
            step(budget)?;
            let erased = erase_type(
                generic,
                &signature.type_parameters,
                &variables,
                class_scope,
                &mut Vec::new(),
                budget,
            )?;
            let Some(descriptor_bytes) = descriptor_component.bytes(descriptor) else {
                return erasure_mismatch("return");
            };
            if erased.as_slice() != descriptor_bytes {
                return erasure_mismatch("return");
            }
            Some(erased)
        }
        _ => return erasure_mismatch("return"),
    };

    let mut throws = Vec::with_capacity(signature.throws.len());
    for (index, exception) in signature.throws.iter().enumerate() {
        step(budget)?;
        let erased = erase_type(
            exception,
            &signature.type_parameters,
            &variables,
            class_scope,
            &mut Vec::new(),
            budget,
        )?;
        let Some(name) = erased
            .strip_prefix(b"L")
            .and_then(|name| name.strip_suffix(b";"))
        else {
            return erasure_mismatch(&format!("throws {index}"));
        };
        throws.push(name.to_vec());
    }
    if !throws.is_empty() && throws.as_slice() != exceptions {
        return erasure_mismatch("Exceptions attribute");
    }

    Ok(MethodSignatureErasureProof {
        type_parameters: type_parameter_erasures,
        parameters,
        result,
        throws,
    })
}

fn erase_type_parameter(
    index: usize,
    parameters: &[TypeParameter],
    variables: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    visiting: &mut Vec<usize>,
    budget: &mut Budget,
) -> Result<Vec<u8>> {
    step(budget)?;
    if visiting.len() >= MAX_SIGNATURE_DEPTH {
        return scope_refused("type-variable erasure dependency exceeds the depth limit");
    }
    if visiting.contains(&index) {
        return scope_refused("cyclic type-variable erasure");
    }
    let Some(parameter) = parameters.get(index) else {
        return scope_refused("type parameter index is outside its scope");
    };
    let bound = parameter
        .class_bound
        .as_ref()
        .or_else(|| parameter.interface_bounds.first());
    let Some(bound) = bound else {
        return Ok(b"Ljava/lang/Object;".to_vec());
    };
    visiting.push(index);
    let erasure = erase_type(bound, parameters, variables, class_scope, visiting, budget);
    visiting.pop();
    erasure
}

fn erase_type(
    ty: &SignatureType,
    parameters: &[TypeParameter],
    variables: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    visiting: &mut Vec<usize>,
    budget: &mut Budget,
) -> Result<Vec<u8>> {
    step(budget)?;
    match ty {
        SignatureType::Base(code) => Ok(vec![*code]),
        SignatureType::Array(component) => {
            let mut descriptor = vec![b'['];
            descriptor.extend(erase_type(
                component,
                parameters,
                variables,
                class_scope,
                visiting,
                budget,
            )?);
            Ok(descriptor)
        }
        SignatureType::TypeVariable(name) => {
            if let Some(index) = variables.iter().position(|variable| variable == name) {
                return erase_type_parameter(
                    index,
                    parameters,
                    variables,
                    class_scope,
                    visiting,
                    budget,
                );
            }
            if let Some(parameter) = class_scope.iter().find(|parameter| &parameter.name == name) {
                return Ok(parameter.descriptor.clone());
            }
            scope_refused(&format!(
                "type variable `{}` is not declared in the available Signature scope",
                String::from_utf8_lossy(name)
            ))
        }
        SignatureType::Class(class) => {
            let Some(segment) = class.segments.last() else {
                return scope_refused("class type has no segments");
            };
            let mut descriptor = Vec::with_capacity(segment.binary_name.len() + 2);
            descriptor.push(b'L');
            descriptor.extend_from_slice(&segment.binary_name);
            descriptor.push(b';');
            Ok(descriptor)
        }
    }
}

fn validate_type_variables(
    ty: &SignatureType,
    variables: &[Vec<u8>],
    class_scope: &[TypeParameterErasure],
    budget: &mut Budget,
) -> Result<()> {
    step(budget)?;
    match ty {
        SignatureType::Base(_) => Ok(()),
        SignatureType::Array(component) => {
            validate_type_variables(component, variables, class_scope, budget)
        }
        SignatureType::TypeVariable(name) => {
            if variables.iter().any(|variable| variable == name)
                || class_scope.iter().any(|parameter| parameter.name == *name)
            {
                Ok(())
            } else {
                scope_refused(&format!(
                    "type variable `{}` is not declared in the available Signature scope",
                    String::from_utf8_lossy(name)
                ))
            }
        }
        SignatureType::Class(class) => {
            for segment in &class.segments {
                for argument in &segment.arguments {
                    match argument {
                        TypeArgument::Any => {}
                        TypeArgument::Exact(ty)
                        | TypeArgument::Extends(ty)
                        | TypeArgument::Super(ty) => {
                            validate_type_variables(ty, variables, class_scope, budget)?;
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

fn step(budget: &mut Budget) -> Result<()> {
    budget.charge(CountedBudgetDimension::AnalysisSteps, 1)
}

fn scope_refused<T>(message: &str) -> Result<T> {
    Err(Error::unsupported("jvm_signature_scope_unproved", message))
}

fn erasure_mismatch<T>(position: &str) -> Result<T> {
    Err(Error::invalid_input(
        "jvm_signature_erasure_mismatch",
        format!("erased Signature disagrees with the physical descriptor at {position}"),
    ))
}

fn references_from_parameters(
    parameters: &[TypeParameter],
    references: &mut Vec<Vec<u8>>,
    seen: &mut HashSet<Vec<u8>>,
    budget: &mut Budget,
) -> Result<()> {
    for parameter in parameters {
        if let Some(bound) = &parameter.class_bound {
            references_from_type(bound, references, seen, budget)?;
        }
        for bound in &parameter.interface_bounds {
            references_from_type(bound, references, seen, budget)?;
        }
    }
    Ok(())
}

fn references_from_type(
    ty: &SignatureType,
    references: &mut Vec<Vec<u8>>,
    seen: &mut HashSet<Vec<u8>>,
    budget: &mut Budget,
) -> Result<()> {
    step(budget)?;
    match ty {
        SignatureType::Base(_) | SignatureType::TypeVariable(_) => {}
        SignatureType::Array(component) => {
            references_from_type(component, references, seen, budget)?
        }
        SignatureType::Class(class) => {
            for segment in &class.segments {
                if seen.insert(segment.binary_name.clone()) {
                    references.push(segment.binary_name.clone());
                }
                for argument in &segment.arguments {
                    match argument {
                        TypeArgument::Any => {}
                        TypeArgument::Exact(ty)
                        | TypeArgument::Extends(ty)
                        | TypeArgument::Super(ty) => {
                            references_from_type(ty, references, seen, budget)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

struct Parser<'a, 'b> {
    reader: Reader<'a>,
    budget: &'b mut Budget,
    nodes: usize,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn new(bytes: &'a [u8], budget: &'b mut Budget) -> Result<Self> {
        if bytes.len() > MAX_SIGNATURE_BYTES {
            return Err(Error::unsupported(
                "jvm_signature_limit_exceeded",
                format!("signature length exceeds {MAX_SIGNATURE_BYTES} bytes"),
            ));
        }
        Ok(Self {
            reader: Reader::new(bytes),
            budget,
            nodes: 0,
        })
    }

    fn node(&mut self) -> Result<()> {
        self.nodes += 1;
        if self.nodes > MAX_SIGNATURE_NODES {
            return Err(Error::unsupported(
                "jvm_signature_limit_exceeded",
                format!("signature contains more than {MAX_SIGNATURE_NODES} grammar nodes"),
            ));
        }
        self.budget.charge(CountedBudgetDimension::AnalysisSteps, 1)
    }

    fn depth(&self, depth: usize) -> Result<()> {
        if depth > MAX_SIGNATURE_DEPTH {
            Err(Error::unsupported(
                "jvm_signature_limit_exceeded",
                format!("signature nesting exceeds {MAX_SIGNATURE_DEPTH} levels"),
            ))
        } else {
            Ok(())
        }
    }

    fn type_parameters(&mut self, depth: usize) -> Result<Vec<TypeParameter>> {
        self.depth(depth)?;
        if self.reader.peek() != Some(b'<') {
            return Ok(Vec::new());
        }
        self.reader.skip(1)?;
        let mut parameters = Vec::new();
        while self.reader.peek() != Some(b'>') {
            self.node()?;
            let name = self.reader.identifier()?;
            self.reader.expect_byte(b':')?;
            let class_bound = if matches!(self.reader.peek(), Some(b':') | Some(b'>')) {
                None
            } else {
                Some(self.reference_type(depth + 1)?)
            };
            let mut interface_bounds = Vec::new();
            while self.reader.peek() == Some(b':') {
                self.reader.skip(1)?;
                interface_bounds.push(self.reference_type(depth + 1)?);
            }
            parameters.push(TypeParameter {
                name,
                class_bound,
                interface_bounds,
            });
        }
        self.reader.expect_byte(b'>')?;
        Ok(parameters)
    }

    fn java_type(&mut self, depth: usize) -> Result<SignatureType> {
        self.depth(depth)?;
        self.node()?;
        match self.reader.peek() {
            Some(b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z') => {
                Ok(SignatureType::Base(self.reader.u8()?))
            }
            Some(b'V') => Err(self.reader.malformed("void is not a java type signature")),
            _ => self.reference_type_after_node(depth),
        }
    }

    fn reference_type(&mut self, depth: usize) -> Result<SignatureType> {
        self.depth(depth)?;
        self.node()?;
        self.reference_type_after_node(depth)
    }

    fn reference_type_after_node(&mut self, depth: usize) -> Result<SignatureType> {
        match self.reader.peek() {
            Some(b'L') => Ok(SignatureType::Class(self.class_type(depth + 1)?)),
            Some(b'T') => self.type_variable(depth + 1),
            Some(b'[') => {
                self.reader.skip(1)?;
                Ok(SignatureType::Array(Box::new(self.java_type(depth + 1)?)))
            }
            _ => Err(self.reader.malformed("reference type signature expected")),
        }
    }

    fn type_variable(&mut self, depth: usize) -> Result<SignatureType> {
        self.depth(depth)?;
        self.node()?;
        self.reader.expect_byte(b'T')?;
        let name = self.reader.identifier()?;
        self.reader.expect_byte(b';')?;
        Ok(SignatureType::TypeVariable(name))
    }

    fn class_type_signature(&mut self, depth: usize) -> Result<SignatureType> {
        Ok(SignatureType::Class(self.class_type(depth)?))
    }

    fn class_type(&mut self, depth: usize) -> Result<ClassType> {
        self.depth(depth)?;
        self.node()?;
        self.reader.expect_byte(b'L')?;
        let mut binary_name = self.reader.identifier()?;
        while self.reader.peek() == Some(b'/') {
            self.reader.skip(1)?;
            binary_name.push(b'/');
            binary_name.extend_from_slice(&self.reader.identifier()?);
        }
        let mut segments = vec![ClassTypeSegment {
            binary_name: binary_name.clone(),
            arguments: self.type_arguments(depth + 1)?,
        }];
        loop {
            match self.reader.peek() {
                Some(b'.') => {
                    self.node()?;
                    self.reader.skip(1)?;
                    binary_name.push(b'$');
                    binary_name.extend_from_slice(&self.reader.identifier()?);
                    segments.push(ClassTypeSegment {
                        binary_name: binary_name.clone(),
                        arguments: self.type_arguments(depth + 1)?,
                    });
                }
                Some(b';') => {
                    self.reader.skip(1)?;
                    break;
                }
                _ => {
                    return Err(self
                        .reader
                        .malformed("class type signature is not terminated"));
                }
            }
        }
        Ok(ClassType { segments })
    }

    fn type_arguments(&mut self, depth: usize) -> Result<Vec<TypeArgument>> {
        self.depth(depth)?;
        if self.reader.peek() != Some(b'<') {
            return Ok(Vec::new());
        }
        self.reader.skip(1)?;
        if self.reader.peek() == Some(b'>') {
            return Err(self.reader.malformed("type argument list is empty"));
        }
        let mut arguments = Vec::new();
        while self.reader.peek() != Some(b'>') {
            self.node()?;
            let argument = match self.reader.peek() {
                Some(b'*') => {
                    self.reader.skip(1)?;
                    TypeArgument::Any
                }
                Some(b'+') => {
                    self.reader.skip(1)?;
                    TypeArgument::Extends(self.reference_type(depth + 1)?)
                }
                Some(b'-') => {
                    self.reader.skip(1)?;
                    TypeArgument::Super(self.reference_type(depth + 1)?)
                }
                _ => TypeArgument::Exact(self.reference_type(depth + 1)?),
            };
            arguments.push(argument);
        }
        self.reader.expect_byte(b'>')?;
        Ok(arguments)
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn is_empty(&self) -> bool {
        self.at == self.bytes.len()
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(count)
            .ok_or_else(|| self.malformed("signature region length overflow"))?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| self.malformed("signature ended before the grammar did"))?;
        self.at = end;
        Ok(bytes)
    }

    fn skip(&mut self, count: usize) -> Result<()> {
        self.take(count).map(|_| ())
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn expect_byte(&mut self, expected: u8) -> Result<()> {
        let found = self.u8()?;
        if found == expected {
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
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.malformed("signature does not end at the grammar boundary"))
        }
    }

    fn identifier(&mut self) -> Result<Vec<u8>> {
        let start = self.at;
        while let Some(byte) = self.peek() {
            if matches!(byte, b'.' | b';' | b'[' | b'/' | b'<' | b'>' | b':') {
                break;
            }
            self.at += 1;
        }
        if start == self.at {
            return Err(self.malformed("identifier expected"));
        }
        Ok(self.bytes[start..self.at].to_vec())
    }

    fn malformed(&self, message: &str) -> Error {
        Error::invalid_input(
            SIGNATURE_CODE,
            format!("{message} (at byte {} of the signature)", self.at),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::{BudgetDimension, CancellationToken, Limits};

    fn test_budget(analysis_steps: u64) -> Budget {
        Budget::new(Limits {
            analysis_steps,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        })
    }

    #[test]
    fn parses_method_variables_and_preserves_class_reference_order() {
        let mut budget = test_budget(u64::MAX);
        let signature =
            parse_method_signature(b"<T:Ljava/lang/Number;>(TT;TT;Z)TT;", &mut budget).unwrap();
        assert_eq!(signature.type_parameters.len(), 1);
        assert_eq!(signature.type_parameters[0].name, b"T");
        assert!(matches!(
            signature.type_parameters[0].class_bound,
            Some(SignatureType::Class(_))
        ));
        assert_eq!(signature.parameters.len(), 3);
        assert_eq!(
            signature.parameters[0],
            SignatureType::TypeVariable(b"T".to_vec())
        );
        assert_eq!(signature.parameters[2], SignatureType::Base(b'Z'));
        assert_eq!(
            signature.result,
            Some(SignatureType::TypeVariable(b"T".to_vec()))
        );
        assert_eq!(
            signature.class_references(&mut budget).unwrap(),
            vec![b"java/lang/Number".to_vec()]
        );

        let mut budget = test_budget(u64::MAX);
        let nested = parse_method_signature(
            b"<T:Lp/Bound;>(Ljava/util/Map<Lp/Arg;>.Entry<Lp/InnerArg;>;)[Lp/Out;^Lp/Thrown;^TT;",
            &mut budget,
        )
        .unwrap();
        assert_eq!(
            nested.class_references(&mut budget).unwrap(),
            vec![
                b"p/Bound".to_vec(),
                b"java/util/Map".to_vec(),
                b"p/Arg".to_vec(),
                b"java/util/Map$Entry".to_vec(),
                b"p/InnerArg".to_vec(),
                b"p/Out".to_vec(),
                b"p/Thrown".to_vec(),
            ]
        );
    }

    #[test]
    fn consumes_the_full_method_grammar_and_keeps_complex_types_structured() {
        let mut budget = test_budget(u64::MAX);
        let signature = parse_method_signature(
            b"<T:Ljava/lang/Number;>([Ljava/util/List<+TT;>;)[Ljava/util/List<+TT;>;^Ljava/io/IOException;",
            &mut budget,
        )
        .unwrap();
        assert_eq!(signature.parameters.len(), 1);
        assert!(matches!(signature.parameters[0], SignatureType::Array(_)));
        assert_eq!(signature.throws.len(), 1);
        assert_eq!(
            signature.class_references(&mut budget).unwrap(),
            vec![
                b"java/lang/Number".to_vec(),
                b"java/util/List".to_vec(),
                b"java/io/IOException".to_vec(),
            ]
        );
        assert!(matches!(
            parse_method_signature(b"()Vtrailing", &mut budget),
            Err(Error::InvalidInput { ref code, .. }) if code == SIGNATURE_CODE
        ));
    }

    #[test]
    fn class_and_field_productions_share_the_same_structured_grammar() {
        let mut budget = test_budget(u64::MAX);
        let class = parse_class_signature(
            b"<T:Ljava/lang/Number;>Ljava/lang/Object;Ljava/io/Serializable;",
            &mut budget,
        )
        .unwrap();
        assert_eq!(class.interfaces.len(), 1);
        assert_eq!(
            class.class_references(&mut budget).unwrap(),
            vec![
                b"java/lang/Number".to_vec(),
                b"java/lang/Object".to_vec(),
                b"java/io/Serializable".to_vec(),
            ]
        );
        let field =
            parse_field_signature(b"Ljava/util/List<Ljava/lang/String;>;", &mut budget).unwrap();
        assert_eq!(
            field.class_references(&mut budget).unwrap(),
            vec![b"java/util/List".to_vec(), b"java/lang/String".to_vec()]
        );
    }

    #[test]
    fn field_signature_erasure_proves_parameterized_wildcard_and_class_variables() {
        let mut budget = test_budget(u64::MAX);
        let class = parse_class_signature(b"<T:Ljava/lang/Number;>Ljava/lang/Object;", &mut budget)
            .unwrap();
        let scope = prove_class_signature_erasure(&class, b"java/lang/Object", &[], &mut budget)
            .unwrap()
            .type_parameters;

        for signature_bytes in [
            &b"Ljava/util/List<Ljava/lang/String;>;"[..],
            b"Ljava/util/List<*>;",
            b"Ljava/util/List<+Ljava/lang/Number;>;",
            b"Ljava/util/List<-Ljava/lang/Integer;>;",
        ] {
            let signature = parse_field_signature(signature_bytes, &mut budget).unwrap();
            prove_field_signature_erasure_with_class_scope(
                &signature,
                b"Ljava/util/List;",
                &scope,
                &mut budget,
            )
            .unwrap();
        }

        for (signature_bytes, descriptor) in [
            (&b"TT;"[..], &b"Ljava/lang/Number;"[..]),
            (b"[TT;", b"[Ljava/lang/Number;"),
        ] {
            let signature = parse_field_signature(signature_bytes, &mut budget).unwrap();
            prove_field_signature_erasure_with_class_scope(
                &signature,
                descriptor,
                &scope,
                &mut budget,
            )
            .unwrap();
        }
    }

    #[test]
    fn field_signature_erasure_rejects_unbound_variables_and_descriptor_mismatch() {
        let mut budget = test_budget(u64::MAX);
        let unbound = parse_field_signature(b"TT;", &mut budget).unwrap();
        assert!(matches!(
            prove_field_signature_erasure_with_class_scope(&unbound, b"Ljava/lang/Object;", &[], &mut budget),
            Err(Error::Unsupported { ref code, .. }) if code == "jvm_signature_scope_unproved"
        ));

        let mismatch = parse_field_signature(b"Ljava/util/List<*>;", &mut budget).unwrap();
        assert!(matches!(
            prove_field_signature_erasure_with_class_scope(&mismatch, b"Ljava/util/Set;", &[], &mut budget),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));
    }

    #[test]
    fn field_signature_erasure_obeys_analysis_budget_and_cancellation() {
        let mut unlimited = test_budget(u64::MAX);
        let signature =
            parse_field_signature(b"Ljava/util/List<Ljava/lang/String;>;", &mut unlimited).unwrap();
        let mut limited = test_budget(0);
        assert!(matches!(
            prove_field_signature_erasure_with_class_scope(
                &signature,
                b"Ljava/util/List;",
                &[],
                &mut limited,
            ),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let cancellation = CancellationToken::new();
        let mut cancelled = Budget::with_cancellation_token(
            Limits {
                analysis_steps: u64::MAX,
                elapsed_millis: u64::MAX,
                ..Limits::default()
            },
            cancellation.clone(),
        );
        cancellation.cancel();
        assert!(matches!(
            prove_field_signature_erasure_with_class_scope(
                &signature,
                b"Ljava/util/List;",
                &[],
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));
    }

    #[test]
    fn parser_rejects_resource_limit_exhaustion_before_unbounded_work() {
        let too_deep = format!(
            "({})V",
            "[".repeat(MAX_SIGNATURE_DEPTH + 2) + "Ljava/lang/Object;"
        );
        let mut budget = test_budget(u64::MAX);
        assert!(matches!(
            parse_method_signature(too_deep.as_bytes(), &mut budget),
            Err(Error::Unsupported { ref code, .. }) if code == "jvm_signature_limit_exceeded"
        ));

        let many_parameters = format!("({})V", "I".repeat(MAX_SIGNATURE_NODES + 1));
        let mut budget = test_budget(u64::MAX);
        assert!(matches!(
            parse_method_signature(many_parameters.as_bytes(), &mut budget),
            Err(Error::Unsupported { ref code, .. }) if code == "jvm_signature_limit_exceeded"
        ));

        let mut budget = test_budget(1);
        assert!(matches!(
            parse_method_signature(b"(II)V", &mut budget),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let cancellation = CancellationToken::new();
        let mut budget = Budget::with_cancellation_token(
            Limits {
                analysis_steps: u64::MAX,
                elapsed_millis: u64::MAX,
                ..Limits::default()
            },
            cancellation.clone(),
        );
        cancellation.cancel();
        assert!(matches!(
            parse_method_signature(b"(I)V", &mut budget),
            Err(Error::Cancelled { .. })
        ));
    }

    #[test]
    fn method_signature_erasure_matches_positions_and_first_bound_rules() {
        let mut budget = test_budget(u64::MAX);
        let signature =
            parse_method_signature(b"<T:Ljava/lang/Number;>(TT;TT;Z)TT;", &mut budget).unwrap();
        let proof = prove_method_signature_erasure(
            &signature,
            b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;",
            &[],
            &mut budget,
        )
        .unwrap();
        assert_eq!(proof.type_parameters[0].name, b"T");
        assert_eq!(proof.type_parameters[0].descriptor, b"Ljava/lang/Number;");
        assert_eq!(proof.parameters[0], b"Ljava/lang/Number;");
        assert_eq!(proof.result, Some(b"Ljava/lang/Number;".to_vec()));

        // An empty class-bound slot uses the first interface bound for erasure.
        let interface_bound =
            parse_method_signature(b"<T::Ljava/io/Serializable;>(TT;)TT;", &mut budget).unwrap();
        let proof = prove_method_signature_erasure(
            &interface_bound,
            b"(Ljava/io/Serializable;)Ljava/io/Serializable;",
            &[],
            &mut budget,
        )
        .unwrap();
        assert_eq!(
            proof.type_parameters[0].descriptor,
            b"Ljava/io/Serializable;"
        );

        let unbounded = parse_method_signature(b"<T:>(TT;)TT;", &mut budget).unwrap();
        let proof = prove_method_signature_erasure(
            &unbounded,
            b"(Ljava/lang/Object;)Ljava/lang/Object;",
            &[],
            &mut budget,
        )
        .unwrap();
        assert_eq!(proof.type_parameters[0].descriptor, b"Ljava/lang/Object;");
    }

    #[test]
    fn class_scope_proves_first_bounds_hierarchy_and_method_variable_erasure() {
        let mut budget = test_budget(u64::MAX);
        let class = parse_class_signature(
            b"<T:Ljava/lang/Number;:Ljava/lang/Comparable<TT;>;U:TT;>Ljava/lang/Object;Ljava/io/Serializable;Ljava/lang/Cloneable;",
            &mut budget,
        )
        .unwrap();
        let class_proof = prove_class_signature_erasure(
            &class,
            b"java/lang/Object",
            &[
                b"java/io/Serializable".to_vec(),
                b"java/lang/Cloneable".to_vec(),
            ],
            &mut budget,
        )
        .unwrap();
        assert_eq!(class_proof.type_parameters[0].name, b"T");
        assert_eq!(
            class_proof.type_parameters[0].descriptor,
            b"Ljava/lang/Number;"
        );
        assert_eq!(class_proof.type_parameters[1].name, b"U");
        assert_eq!(
            class_proof.type_parameters[1].descriptor,
            b"Ljava/lang/Number;"
        );
        assert_eq!(class_proof.superclass, b"java/lang/Object");
        assert_eq!(
            class_proof.interfaces,
            vec![
                b"java/io/Serializable".to_vec(),
                b"java/lang/Cloneable".to_vec()
            ]
        );

        let method = parse_method_signature(b"(TT;TU;)TT;", &mut budget).unwrap();
        let method_proof = prove_method_signature_erasure_with_class_scope(
            &method,
            b"(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;",
            &[],
            &class_proof.type_parameters,
            &mut budget,
        )
        .unwrap();
        assert_eq!(method_proof.parameters[0], b"Ljava/lang/Number;");
        assert_eq!(method_proof.parameters[1], b"Ljava/lang/Number;");
        assert_eq!(method_proof.result, Some(b"Ljava/lang/Number;".to_vec()));

        let method_local_bound = parse_method_signature(b"<V:TT;>(TV;)TV;", &mut budget).unwrap();
        let local_proof = prove_method_signature_erasure_with_class_scope(
            &method_local_bound,
            b"(Ljava/lang/Number;)Ljava/lang/Number;",
            &[],
            &class_proof.type_parameters,
            &mut budget,
        )
        .unwrap();
        assert_eq!(
            local_proof.type_parameters[0].descriptor,
            b"Ljava/lang/Number;"
        );

        let unbounded_shadow = parse_method_signature(b"<T:>(TT;)TT;", &mut budget).unwrap();
        let unbounded_proof = prove_method_signature_erasure_with_class_scope(
            &unbounded_shadow,
            b"(Ljava/lang/Object;)Ljava/lang/Object;",
            &[],
            &class_proof.type_parameters,
            &mut budget,
        )
        .unwrap();
        assert_eq!(
            unbounded_proof.type_parameters[0].descriptor,
            b"Ljava/lang/Object;"
        );
        assert_eq!(unbounded_proof.parameters[0], b"Ljava/lang/Object;");
        assert_eq!(unbounded_proof.result, Some(b"Ljava/lang/Object;".to_vec()));

        // The first interface bound is method-local too, even when the class T erases to Number.
        let bounded_shadow =
            parse_method_signature(b"<T::Ljava/lang/CharSequence;>(TT;)TT;", &mut budget).unwrap();
        let bounded_proof = prove_method_signature_erasure_with_class_scope(
            &bounded_shadow,
            b"(Ljava/lang/CharSequence;)Ljava/lang/CharSequence;",
            &[],
            &class_proof.type_parameters,
            &mut budget,
        )
        .unwrap();
        assert_eq!(
            bounded_proof.type_parameters[0].descriptor,
            b"Ljava/lang/CharSequence;"
        );
        assert_eq!(bounded_proof.parameters[0], b"Ljava/lang/CharSequence;");
        assert_eq!(
            bounded_proof.result,
            Some(b"Ljava/lang/CharSequence;".to_vec())
        );
    }

    #[test]
    fn class_scope_rejects_unbound_duplicate_and_cyclic_variables() {
        let mut budget = test_budget(u64::MAX);
        for bytes in [
            &b"<T:TU;>Ljava/lang/Object;"[..],
            &b"<T:TT;>Ljava/lang/Object;"[..],
            &b"<T:Ljava/lang/Number;T:Ljava/lang/Object;>Ljava/lang/Object;"[..],
            &b"<T:Ljava/lang/Number;>Lp/Base<TU;>;"[..],
        ] {
            let class = parse_class_signature(bytes, &mut budget).unwrap();
            assert!(matches!(
                prove_class_signature_erasure(&class, b"java/lang/Object", &[], &mut budget),
                Err(Error::Unsupported { ref code, .. }) if code == "jvm_signature_scope_unproved"
            ));
        }
    }

    #[test]
    fn class_scope_rejects_method_local_scope_errors_and_wrong_shadow_erasure() {
        let mut budget = test_budget(u64::MAX);
        let class = parse_class_signature(b"<T:Ljava/lang/Number;>Ljava/lang/Object;", &mut budget)
            .unwrap();
        let class_scope =
            prove_class_signature_erasure(&class, b"java/lang/Object", &[], &mut budget)
                .unwrap()
                .type_parameters;

        for bytes in [
            &b"<T::Ljava/lang/CharSequence;T::Ljava/lang/CharSequence;>(TT;)TT;"[..],
            b"<T::Ljava/lang/CharSequence;>(TU;)TT;",
            b"<T:TT;>(TT;)TT;",
        ] {
            let method = parse_method_signature(bytes, &mut budget).unwrap();
            assert!(matches!(
                prove_method_signature_erasure_with_class_scope(
                    &method,
                    b"(Ljava/lang/Number;)Ljava/lang/Number;",
                    &[],
                    &class_scope,
                    &mut budget,
                ),
                Err(Error::Unsupported { ref code, .. }) if code == "jvm_signature_scope_unproved"
            ));
        }

        // Signature metadata does not participate in JVM bytecode verification; an attribute
        // whose local bound no longer matches the physical descriptor must still be refused.
        let wrong_erasure =
            parse_method_signature(b"<T::Ljava/lang/CharSequence;>(TT;)TT;", &mut budget).unwrap();
        assert!(matches!(
            prove_method_signature_erasure_with_class_scope(
                &wrong_erasure,
                b"(Ljava/lang/Number;)Ljava/lang/Number;",
                &[],
                &class_scope,
                &mut budget,
            ),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));
    }

    #[test]
    fn class_scope_shadowing_checks_method_local_throws_erasure() {
        let mut budget = test_budget(u64::MAX);
        let class = parse_class_signature(b"<T:Ljava/lang/Number;>Ljava/lang/Object;", &mut budget)
            .unwrap();
        let class_scope =
            prove_class_signature_erasure(&class, b"java/lang/Object", &[], &mut budget)
                .unwrap()
                .type_parameters;
        let method =
            parse_method_signature(b"<T:Ljava/lang/Exception;>(TT;)TT;^TT;", &mut budget).unwrap();
        let exceptions = vec![b"java/lang/Exception".to_vec()];
        let proof = prove_method_signature_erasure_with_class_scope(
            &method,
            b"(Ljava/lang/Exception;)Ljava/lang/Exception;",
            &exceptions,
            &class_scope,
            &mut budget,
        )
        .unwrap();
        assert_eq!(
            proof.type_parameters[0].descriptor,
            b"Ljava/lang/Exception;"
        );
        assert_eq!(proof.parameters[0], b"Ljava/lang/Exception;");
        assert_eq!(proof.result, Some(b"Ljava/lang/Exception;".to_vec()));
        assert_eq!(proof.throws, exceptions);

        assert!(matches!(
            prove_method_signature_erasure_with_class_scope(
                &method,
                b"(Ljava/lang/Exception;)Ljava/lang/Exception;",
                &[b"java/io/IOException".to_vec()],
                &class_scope,
                &mut budget,
            ),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));
    }

    #[test]
    fn class_scope_shadow_proof_obeys_analysis_budget_and_cancellation() {
        let mut unlimited = test_budget(u64::MAX);
        let class =
            parse_class_signature(b"<T:Ljava/lang/Number;>Ljava/lang/Object;", &mut unlimited)
                .unwrap();
        let class_scope =
            prove_class_signature_erasure(&class, b"java/lang/Object", &[], &mut unlimited)
                .unwrap()
                .type_parameters;
        let method =
            parse_method_signature(b"<T::Ljava/lang/CharSequence;>(TT;)TT;", &mut unlimited)
                .unwrap();

        let mut limited = test_budget(0);
        assert!(matches!(
            prove_method_signature_erasure_with_class_scope(
                &method,
                b"(Ljava/lang/CharSequence;)Ljava/lang/CharSequence;",
                &[],
                &class_scope,
                &mut limited,
            ),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let cancellation = CancellationToken::new();
        let mut cancelled = Budget::with_cancellation_token(
            Limits {
                analysis_steps: u64::MAX,
                elapsed_millis: u64::MAX,
                ..Limits::default()
            },
            cancellation.clone(),
        );
        cancellation.cancel();
        assert!(matches!(
            prove_method_signature_erasure_with_class_scope(
                &method,
                b"(Ljava/lang/CharSequence;)Ljava/lang/CharSequence;",
                &[],
                &class_scope,
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));
    }

    #[test]
    fn class_scope_rejects_physical_superclass_and_interface_mismatches() {
        let mut budget = test_budget(u64::MAX);
        let class = parse_class_signature(
            b"<T:Ljava/lang/Number;>Ljava/lang/Object;Ljava/io/Serializable;Ljava/lang/Cloneable;",
            &mut budget,
        )
        .unwrap();
        assert!(matches!(
            prove_class_signature_erasure(&class, b"java/lang/Number", &[], &mut budget),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));
        assert!(matches!(
            prove_class_signature_erasure(
                &class,
                b"java/lang/Object",
                &[b"java/lang/Cloneable".to_vec(), b"java/io/Serializable".to_vec()],
                &mut budget,
            ),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));
        assert!(matches!(
            prove_class_signature_erasure(
                &class,
                b"java/lang/Object",
                &[b"java/io/Serializable".to_vec()],
                &mut budget,
            ),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));
    }

    #[test]
    fn class_scope_proof_obeys_budget_and_cancellation() {
        let mut unlimited = test_budget(u64::MAX);
        let class =
            parse_class_signature(b"<T:Ljava/lang/Number;>Ljava/lang/Object;", &mut unlimited)
                .unwrap();
        let mut limited = test_budget(0);
        assert!(matches!(
            prove_class_signature_erasure(&class, b"java/lang/Object", &[], &mut limited),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let cancellation = CancellationToken::new();
        let mut cancelled = Budget::with_cancellation_token(
            Limits {
                analysis_steps: u64::MAX,
                elapsed_millis: u64::MAX,
                ..Limits::default()
            },
            cancellation.clone(),
        );
        cancellation.cancel();
        assert!(matches!(
            prove_class_signature_erasure(&class, b"java/lang/Object", &[], &mut cancelled),
            Err(Error::Cancelled { .. })
        ));
    }

    #[test]
    fn method_signature_erasure_rejects_unbound_and_mismatched_facts() {
        let mut budget = test_budget(u64::MAX);
        let object_bound =
            parse_method_signature(b"<T:Ljava/lang/Object;>(TT;TT;Z)TT;", &mut budget).unwrap();
        assert!(matches!(
            prove_method_signature_erasure(
                &object_bound,
                b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;",
                &[],
                &mut budget,
            ),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));

        let return_mismatch = parse_method_signature(
            b"<T:Ljava/lang/Number;>(TT;TT;Z)Ljava/lang/Object;",
            &mut budget,
        )
        .unwrap();
        assert!(matches!(
            prove_method_signature_erasure(
                &return_mismatch,
                b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;",
                &[],
                &mut budget,
            ),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));

        let unbound =
            parse_method_signature(b"<T:Ljava/lang/Number;>(TU;TT;Z)TU;", &mut budget).unwrap();
        assert!(matches!(
            prove_method_signature_erasure(
                &unbound,
                b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;",
                &[],
                &mut budget,
            ),
            Err(Error::Unsupported { ref code, .. }) if code == "jvm_signature_scope_unproved"
        ));

        let nested_unbound = parse_method_signature(
            b"<T:Ljava/lang/Number;>(Ljava/util/List<TU;>;)TT;",
            &mut budget,
        )
        .unwrap();
        assert!(matches!(
            prove_method_signature_erasure(
                &nested_unbound,
                b"(Ljava/util/List;)Ljava/lang/Number;",
                &[],
                &mut budget,
            ),
            Err(Error::Unsupported { ref code, .. }) if code == "jvm_signature_scope_unproved"
        ));

        // A class-level `T` is not in the method-local scope this proof establishes.
        let class_variable = parse_method_signature(b"(TT;)TT;", &mut budget).unwrap();
        assert!(matches!(
            prove_method_signature_erasure(
                &class_variable,
                b"(Ljava/lang/Number;)Ljava/lang/Number;",
                &[],
                &mut budget,
            ),
            Err(Error::Unsupported { ref code, .. }) if code == "jvm_signature_scope_unproved"
        ));
    }

    #[test]
    fn erasure_proof_handles_complex_types_and_matches_exceptions_attribute() {
        let mut budget = test_budget(u64::MAX);
        let signature = parse_method_signature(
            b"<T:Ljava/lang/Number;>([Ljava/util/List<+TT;>;)[Ljava/util/List<+TT;>;^Ljava/io/IOException;",
            &mut budget,
        )
        .unwrap();
        let descriptor = b"([Ljava/util/List;)[Ljava/util/List;";
        let exception = vec![b"java/io/IOException".to_vec()];
        let proof = prove_method_signature_erasure(&signature, descriptor, &exception, &mut budget)
            .unwrap();
        assert_eq!(proof.parameters, vec![b"[Ljava/util/List;".to_vec()]);
        assert_eq!(proof.result, Some(b"[Ljava/util/List;".to_vec()));
        assert_eq!(proof.throws, exception);
        assert!(matches!(
            prove_method_signature_erasure(&signature, descriptor, &[], &mut budget),
            Err(Error::InvalidInput { ref code, .. }) if code == "jvm_signature_erasure_mismatch"
        ));
    }
}
