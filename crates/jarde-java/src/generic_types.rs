//! Declared source types bound to the definitions selected by this recovery request.
//!
//! Signature syntax alone is not a type identity: substituted arguments retain the loader of
//! their declaration, whereas a parent's own references resolve from that parent's loader.
//! This arena is the source-type domain consumed by variable planning; JVM erasures stay intact.

use crate::report::{DeclaredGenericMemberTypes, DeclaredGenericType as Ty, DeclaredGenericTypes};
use crate::source_proof::{DependencyResult, RecoveryHeaders, ReferenceClassHeader};
use jarde_jvm::resolver::ResolvedMemberRef;
use jarde_reader::{
    budget::{Budget, CountedBudgetDimension as Cost},
    error::Result,
    model::{JvmBytes, SymbolRef},
    signature::{ClassType, SignatureType, TypeArgument, TypeParameter, TypeParameterErasure},
    view::LoaderId,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn intern(types: &mut DeclaredGenericTypes, ty: Ty, budget: &mut Budget) -> Result<usize> {
    for (index, existing) in types.types.iter().enumerate() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if existing == &ty {
            return Ok(index);
        }
    }
    budget.charge(Cost::IrItems, 1)?;
    let index = types.types.len();
    types.types.push(ty);
    Ok(index)
}

fn charge_syntax(ty: &SignatureType, budget: &mut Budget) -> Result<()> {
    budget.poll()?;
    budget.charge(Cost::IrItems, 1)?;
    match ty {
        SignatureType::Array(element) => charge_syntax(element, budget)?,
        SignatureType::Class(class) => {
            for segment in &class.segments {
                budget.charge(Cost::IrItems, 1 + segment.arguments.len() as u64)?;
                for argument in &segment.arguments {
                    match argument {
                        TypeArgument::Exact(ty)
                        | TypeArgument::Extends(ty)
                        | TypeArgument::Super(ty) => charge_syntax(ty, budget)?,
                        TypeArgument::Any => budget.poll()?,
                    }
                }
            }
        }
        SignatureType::Base(_) | SignatureType::TypeVariable(_) => {}
    }
    Ok(())
}

/// Bind one declared type. The environment maps declaration-local variable names to already
/// scoped nodes; substituting an argument copies its node index, never its unbound syntax.
pub fn bind_declared_type<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    syntax: &SignatureType,
    loader: &LoaderId,
    variables: &[(Vec<u8>, usize)],
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<usize>, H::Error> {
    bind_type(types, syntax, loader, variables, headers, budget, 0)
}

fn bind_type<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    syntax: &SignatureType,
    loader: &LoaderId,
    variables: &[(Vec<u8>, usize)],
    headers: &mut H,
    budget: &mut Budget,
    depth: u64,
) -> std::result::Result<DependencyResult<usize>, H::Error> {
    budget.poll()?;
    budget.observe_dependency_depth(depth)?;
    budget.charge(Cost::AnalysisSteps, 1)?;
    let ty = match syntax {
        SignatureType::Base(base) => Ty::Base(*base),
        SignatureType::TypeVariable(name) => {
            for (declared, index) in variables.iter().rev() {
                budget.charge(Cost::AnalysisSteps, 1)?;
                if declared == name {
                    return Ok(DependencyResult::Found(*index));
                }
            }
            return Ok(DependencyResult::Unknown);
        }
        SignatureType::Array(element) => {
            let element = match bind_type(
                types,
                element,
                loader,
                variables,
                headers,
                budget,
                depth + 1,
            )? {
                DependencyResult::Found(element) => element,
                DependencyResult::Missing => return Ok(DependencyResult::Missing),
                DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
                DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
            };
            budget.charge(Cost::IrEdges, 1)?;
            Ty::Array(element)
        }
        SignatureType::Class(class) => {
            let Some(segment) = class.segments.last() else {
                return Ok(DependencyResult::Unknown);
            };
            // Parameterized enclosing instances require a separate enclosing receiver. They are
            // preserved by Reader, but this finite source-type domain does not infer that receiver.
            if class.segments[..class.segments.len() - 1]
                .iter()
                .any(|segment| !segment.arguments.is_empty())
            {
                return Ok(DependencyResult::Unknown);
            }
            let header =
                match headers.selected_generic_from(loader, &segment.binary_name, budget)? {
                    DependencyResult::Found(header) => header,
                    DependencyResult::Missing => return Ok(DependencyResult::Missing),
                    DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
                    DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
                };
            let Some(signatures) = &header.generic_signatures else {
                return Ok(DependencyResult::Unknown);
            };
            let arity = signatures
                .class
                .as_ref()
                .map_or(0, |(signature, _)| signature.type_parameters.len());
            // Empty arguments represent a physical raw use. It may be retained, but cannot
            // supply bindings when traversing a parameterized parent's declaration below.
            if !segment.arguments.is_empty() && arity != segment.arguments.len() {
                return Ok(DependencyResult::Unknown);
            }
            let mut arguments = Vec::new();
            for argument in &segment.arguments {
                budget.poll()?;
                let TypeArgument::Exact(argument) = argument else {
                    return Ok(DependencyResult::Unknown);
                };
                let argument = match bind_type(
                    types,
                    argument,
                    loader,
                    variables,
                    headers,
                    budget,
                    depth + 1,
                )? {
                    DependencyResult::Found(argument) => argument,
                    DependencyResult::Missing => return Ok(DependencyResult::Missing),
                    DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
                    DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
                };
                budget.charge(Cost::IrItems, 1)?;
                budget.charge(Cost::IrEdges, 1)?;
                arguments.push(argument);
            }
            Ty::Reference {
                declaration: ResolvedMemberRef {
                    loader: header.loader,
                    definition: header.definition,
                    member: SymbolRef::Class {
                        owner: JvmBytes(segment.binary_name.clone()),
                    },
                },
                arguments,
            }
        }
    };
    Ok(DependencyResult::Found(intern(types, ty, budget)?))
}

/// Bind an actual class/method's own declared variables before their bounds, which permits
/// recursive bounds. Method variables shadow class names by environment order, not by erasure.
pub fn bind_declared_parameters<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    header: &ReferenceClassHeader,
    method: Option<(&[u8], &[u8])>,
    parameters: &[TypeParameter],
    erasures: &[TypeParameterErasure],
    enclosing: &[(Vec<u8>, usize)],
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<Vec<(Vec<u8>, usize)>>, H::Error> {
    if parameters.len() != erasures.len() {
        return Ok(DependencyResult::Unknown);
    }
    let mut variables = Vec::new();
    budget.charge(Cost::IrItems, enclosing.len() as u64)?;
    variables.extend_from_slice(enclosing);
    for (ordinal, (parameter, erasure)) in parameters.iter().zip(erasures).enumerate() {
        budget.poll()?;
        if parameter.name != erasure.name {
            return Ok(DependencyResult::Unknown);
        }
        let node = intern(
            types,
            Ty::Variable {
                loader: header.loader.clone(),
                owner: header.definition.clone(),
                method: method.map(|(name, descriptor)| {
                    (JvmBytes(name.to_vec()), JvmBytes(descriptor.to_vec()))
                }),
                ordinal,
                name: JvmBytes(parameter.name.clone()),
                erasure: JvmBytes(erasure.descriptor.clone()),
            },
            budget,
        )?;
        budget.charge(Cost::IrItems, 1)?;
        variables.push((parameter.name.clone(), node));
    }
    for (parameter, (_, variable)) in parameters.iter().zip(&variables[enclosing.len()..]) {
        let mut bounds = Vec::new();
        for bound in parameter
            .class_bound
            .iter()
            .chain(&parameter.interface_bounds)
        {
            let bound = match bind_declared_type(
                types,
                bound,
                &header.loader,
                &variables,
                headers,
                budget,
            )? {
                DependencyResult::Found(bound) => bound,
                DependencyResult::Missing => return Ok(DependencyResult::Missing),
                DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
                DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
            };
            budget.charge(Cost::IrItems, 1)?;
            budget.charge(Cost::IrEdges, 1)?;
            bounds.push(bound);
        }
        if !types
            .bounds
            .iter()
            .any(|(existing, _)| existing == variable)
        {
            budget.charge(Cost::IrItems, 1)?;
            types.bounds.push((*variable, bounds));
        }
    }
    Ok(DependencyResult::Found(variables))
}

/// Bind a selected class declaration's own Signature without constructing a method IR.
/// This establishes source-type facts only; publishing the declaration still requires the
/// caller's class-level transaction to admit every affected use and write.
pub fn declared_class_types<H: RecoveryHeaders>(
    actual_class: &ResolvedMemberRef,
    header: &ReferenceClassHeader,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<(DeclaredGenericTypes, usize, Vec<usize>)>, H::Error> {
    let SymbolRef::Class { owner } = &actual_class.member else {
        return Ok(DependencyResult::Unknown);
    };
    if actual_class.loader != header.loader || actual_class.definition != header.definition {
        return Ok(DependencyResult::Unknown);
    }
    let selected = match headers.selected_generic_from(&actual_class.loader, &owner.0, budget)? {
        DependencyResult::Found(selected)
            if selected.loader == actual_class.loader
                && selected.definition == actual_class.definition
                && selected.generic_signatures == header.generic_signatures =>
        {
            selected
        }
        DependencyResult::Found(_) | DependencyResult::Unknown => {
            return Ok(DependencyResult::Unknown);
        }
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
    };
    let Some(signatures) = &selected.generic_signatures else {
        return Ok(DependencyResult::Unknown);
    };
    let erased;
    let (signature, erasures) = if let Some((signature, proof)) = &signatures.class {
        (signature, proof.type_parameters.as_slice())
    } else {
        let Some(parent) = &selected.super_name else {
            return Ok(DependencyResult::Unknown);
        };
        let raw = |name: &[u8]| ClassType {
            segments: vec![jarde_reader::signature::ClassTypeSegment {
                binary_name: name.to_vec(),
                arguments: Vec::new(),
            }],
        };
        budget.charge(Cost::IrItems, 2 + selected.interfaces.len() as u64)?;
        erased = jarde_reader::signature::ClassSignature {
            type_parameters: Vec::new(),
            superclass: raw(parent),
            interfaces: selected.interfaces.iter().map(|name| raw(name)).collect(),
        };
        (&erased, &[][..])
    };

    let mut types = DeclaredGenericTypes::default();
    types.declaration = Some(actual_class.clone());
    let variables = match bind_declared_parameters(
        &mut types,
        &selected,
        None,
        &signature.type_parameters,
        erasures,
        &[],
        headers,
        budget,
    )? {
        DependencyResult::Found(variables) => variables,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    budget.charge(Cost::IrItems, variables.len() as u64)?;
    types.class_parameters = variables.iter().map(|(_, node)| *node).collect();

    let superclass = match bind_declared_type(
        &mut types,
        &SignatureType::Class(signature.superclass.clone()),
        &selected.loader,
        &variables,
        headers,
        budget,
    )? {
        DependencyResult::Found(node) => node,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    let mut interfaces = Vec::new();
    budget.charge(Cost::IrItems, signature.interfaces.len() as u64)?;
    for interface in &signature.interfaces {
        match bind_declared_type(
            &mut types,
            &SignatureType::Class(interface.clone()),
            &selected.loader,
            &variables,
            headers,
            budget,
        )? {
            DependencyResult::Found(node) => interfaces.push(node),
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        }
    }

    // The same walkers used by method declarations establish bounds, selected parent edges,
    // and which referenced types can be named from this class's actual package/loader.
    prove_declared_relations(&mut types, headers, budget)?;
    for index in 0..types.class_parameters.len() {
        let node = types.class_parameters[index];
        match declared_type_bounds(&mut types, node, headers, budget)? {
            DependencyResult::Found(true) => {}
            DependencyResult::Found(false) | DependencyResult::Unknown => {
                return Ok(DependencyResult::Unknown);
            }
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        }
    }
    for (node, interface) in std::iter::once((superclass, false))
        .chain(interfaces.iter().copied().map(|node| (node, true)))
    {
        let parent = match selected_node(&types, node, headers, budget)? {
            DependencyResult::Found(parent) => parent,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
        if (parent.flags & crate::facts::ACC_INTERFACE != 0) != interface {
            return Ok(DependencyResult::Unknown);
        }
        match declared_type_bounds(&mut types, node, headers, budget)? {
            DependencyResult::Found(true) => {}
            DependencyResult::Found(false) | DependencyResult::Unknown => {
                return Ok(DependencyResult::Unknown);
            }
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        }
    }
    Ok(DependencyResult::Found((types, superclass, interfaces)))
}

fn selected_node<H: RecoveryHeaders>(
    types: &DeclaredGenericTypes,
    node: usize,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<ReferenceClassHeader>, H::Error> {
    let Some(Ty::Reference { declaration, .. }) = types.types.get(node) else {
        return Ok(DependencyResult::Unknown);
    };
    let SymbolRef::Class { owner } = &declaration.member else {
        return Ok(DependencyResult::Unknown);
    };
    Ok(
        match headers.selected_generic_from(&declaration.loader, &owner.0, budget)? {
            DependencyResult::Found(header)
                if header.loader == declaration.loader
                    && header.definition == declaration.definition =>
            {
                DependencyResult::Found(header)
            }
            DependencyResult::Found(_) => DependencyResult::Unknown,
            other => other,
        },
    )
}

/// Compose declared parent arguments using bound nodes. A target is the original resolver's
/// physical declaration; a same-named parent in another loader cannot satisfy it. All paths are
/// checked, so a conflicting diamond is Ambiguous and an incomplete branch remains Unknown.
pub fn declared_supertype<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    source: usize,
    target: &ResolvedMemberRef,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<usize>, H::Error> {
    Ok(match walk_hierarchy(types, source, Some(target), headers, budget)? {
        DependencyResult::Found(nodes) => nodes.into_iter().find(|node| {
            matches!(&types.types[*node], Ty::Reference { declaration, .. }
                if declaration.loader == target.loader && declaration.definition == target.definition)
        }).map_or(DependencyResult::Missing, DependencyResult::Found),
        DependencyResult::Missing => DependencyResult::Missing,
        DependencyResult::Ambiguous => DependencyResult::Ambiguous,
        DependencyResult::Unknown => DependencyResult::Unknown,
    })
}

fn walk_hierarchy<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    source: usize,
    target: Option<&ResolvedMemberRef>,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<Vec<usize>>, H::Error> {
    let mut pending = VecDeque::new();
    let mut variables = vec![(source, Vec::new())];
    budget.charge(Cost::IrItems, 1)?;
    while let Some((node, mut ancestors)) = variables.pop() {
        budget.poll()?;
        budget.observe_dependency_depth(ancestors.len() as u64)?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        match types.types.get(node) {
            Some(Ty::Reference { .. }) => {
                budget.charge(Cost::IrItems, 1)?;
                pending.push_back((node, Vec::new()));
            }
            Some(Ty::Variable { .. }) => {
                if ancestors.contains(&node) {
                    return Ok(DependencyResult::Unknown);
                }
                let Some((_, bounds)) = types.bounds.iter().find(|(variable, _)| *variable == node)
                else {
                    return Ok(DependencyResult::Unknown);
                };
                if bounds.is_empty() {
                    return Ok(DependencyResult::Unknown);
                }
                budget.charge(Cost::IrItems, 1)?;
                ancestors.push(node);
                for bound in bounds {
                    budget.charge(Cost::IrItems, 1 + ancestors.len() as u64)?;
                    budget.charge(Cost::IrEdges, 1)?;
                    variables.push((*bound, ancestors.clone()));
                }
            }
            _ => return Ok(DependencyResult::Unknown),
        }
    }
    let mut seen = Vec::new();
    let mut incomplete = false;
    while let Some((node, mut ancestors)) = pending.pop_front() {
        budget.poll()?;
        budget.observe_dependency_depth(ancestors.len() as u64)?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        let header = match selected_node(types, node, headers, budget)? {
            DependencyResult::Found(header) => header,
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Missing | DependencyResult::Unknown => {
                incomplete = true;
                continue;
            }
        };
        let key = (header.loader.clone(), header.definition.clone());
        if ancestors.contains(&key) {
            return Ok(DependencyResult::Unknown);
        }
        let mut repeated = false;
        for previous in &seen {
            budget.charge(Cost::AnalysisSteps, 1)?;
            let Ty::Reference { declaration, .. } = &types.types[*previous] else {
                unreachable!()
            };
            if declaration.loader == header.loader && declaration.definition == header.definition {
                if *previous != node {
                    return Ok(DependencyResult::Ambiguous);
                }
                repeated = true;
                break;
            }
        }
        if repeated {
            continue;
        }
        budget.charge(Cost::IrItems, 1)?;
        seen.push(node);
        if target.is_some_and(|target| {
            header.loader == target.loader && header.definition == target.definition
        }) {
            continue;
        }
        let Some(signatures) = &header.generic_signatures else {
            incomplete = true;
            continue;
        };
        let Ty::Reference { arguments, .. } = &types.types[node] else {
            unreachable!()
        };
        let parameters = signatures.class.as_ref().map_or(&[][..], |(signature, _)| {
            signature.type_parameters.as_slice()
        });
        if !arguments.is_empty() && parameters.len() != arguments.len() {
            incomplete = true;
            continue;
        }
        budget.charge(Cost::IrItems, parameters.len() as u64)?;
        let variables: Vec<_> = parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| (parameter.name.clone(), *argument))
            .collect();
        let parents: Vec<_> = if let Some((signature, _)) = &signatures.class
            && parameters.len() == arguments.len()
        {
            budget.charge(Cost::IrItems, 1 + signature.interfaces.len() as u64)?;
            for parent in std::iter::once(&signature.superclass).chain(&signature.interfaces) {
                // The temporary syntax is charged before its recursive allocation below.
                for segment in &parent.segments {
                    budget.charge(Cost::IrItems, 1 + segment.arguments.len() as u64)?;
                    for argument in &segment.arguments {
                        if let TypeArgument::Exact(ty)
                        | TypeArgument::Extends(ty)
                        | TypeArgument::Super(ty) = argument
                        {
                            charge_syntax(ty, budget)?;
                        }
                    }
                }
            }
            std::iter::once(&signature.superclass)
                .chain(&signature.interfaces)
                .cloned()
                .collect()
        } else {
            budget.charge(
                Cost::IrItems,
                u64::from(header.super_name.is_some()) + header.interfaces.len() as u64,
            )?;
            header
                .super_name
                .iter()
                .chain(&header.interfaces)
                .map(|name| ClassType {
                    segments: vec![jarde_reader::signature::ClassTypeSegment {
                        binary_name: name.clone(),
                        arguments: Vec::new(),
                    }],
                })
                .collect()
        };
        budget.charge(Cost::IrItems, 1)?;
        ancestors.push(key);
        for parent in parents {
            let parent = match bind_declared_type(
                types,
                &SignatureType::Class(parent),
                &header.loader,
                &variables,
                headers,
                budget,
            )? {
                DependencyResult::Found(parent) => parent,
                DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
                DependencyResult::Missing | DependencyResult::Unknown => {
                    incomplete = true;
                    continue;
                }
            };
            budget.charge(Cost::IrItems, 1 + ancestors.len() as u64)?;
            budget.charge(Cost::IrEdges, 1)?;
            pending.push_back((parent, ancestors.clone()));
        }
    }
    Ok(if incomplete {
        DependencyResult::Unknown
    } else {
        DependencyResult::Found(seen)
    })
}

/// Assignment in the finite declared-type domain: exact generic arguments, selected ancestor
/// applications, declared variable bounds, and reference-array covariance. It performs neither
/// unchecked conversion nor invocation inference. An incomplete hierarchy is not incompatibility.
pub fn declared_assignable<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    source: usize,
    target: usize,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<bool>, H::Error> {
    assignable(types, source, target, headers, budget, &mut Vec::new())
}

/// Check the declaration's actual type-parameter bounds, including substitutions in bounds.
/// Reader's erasure proof deliberately does not establish these source-language constraints.
pub fn declared_type_bounds<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    node: usize,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<bool>, H::Error> {
    let mut pending = vec![node];
    let mut seen = BTreeSet::new();
    budget.charge(Cost::IrItems, 1)?;
    while let Some(node) = pending.pop() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if !seen.insert(node) {
            continue;
        }
        budget.charge(Cost::IrItems, 1)?;
        match types.types.get(node) {
            Some(Ty::Base(_)) => continue,
            Some(Ty::Variable { .. }) => {
                let Some((_, bounds)) = types.bounds.iter().find(|(variable, _)| *variable == node)
                else {
                    return Ok(DependencyResult::Unknown);
                };
                budget.charge(Cost::IrItems, bounds.len() as u64)?;
                pending.extend(bounds.iter().copied());
                continue;
            }
            Some(Ty::Array(component)) => {
                budget.charge(Cost::IrItems, 1)?;
                pending.push(*component);
                continue;
            }
            Some(Ty::Reference { .. }) => {}
            None => return Ok(DependencyResult::Unknown),
        }
        let header = match selected_node(types, node, headers, budget)? {
            DependencyResult::Found(header) => header,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
        let Some(signatures) = &header.generic_signatures else {
            return Ok(DependencyResult::Unknown);
        };
        let parameters = signatures.class.as_ref().map_or(&[][..], |(signature, _)| {
            signature.type_parameters.as_slice()
        });
        let Ty::Reference { arguments, .. } = &types.types[node] else {
            unreachable!()
        };
        if arguments.is_empty() {
            continue;
        }
        if arguments.len() != parameters.len() {
            return Ok(DependencyResult::Found(false));
        }
        budget.charge(Cost::IrItems, 2 * arguments.len() as u64)?;
        let variables: Vec<_> = parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| (parameter.name.clone(), *argument))
            .collect();
        pending.extend(arguments.iter().copied());
        for (parameter, (_, argument)) in parameters.iter().zip(&variables) {
            for bound in parameter
                .class_bound
                .iter()
                .chain(&parameter.interface_bounds)
            {
                let bound = match bind_declared_type(
                    types,
                    bound,
                    &header.loader,
                    &variables,
                    headers,
                    budget,
                )? {
                    DependencyResult::Found(bound) => bound,
                    DependencyResult::Missing => return Ok(DependencyResult::Missing),
                    DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
                    DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
                };
                match declared_assignable(types, *argument, bound, headers, budget)? {
                    DependencyResult::Found(true) => {}
                    other => return Ok(other),
                }
            }
        }
    }
    Ok(DependencyResult::Found(true))
}

/// Complete only the type relations and bounds that the bound arena contains. These are
/// request-owned proofs, shared by the final local/call planner; an incomplete source contributes
/// no positive relation and does not erase another independent source's proved relations.
pub fn prove_declared_relations<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<(), H::Error> {
    let mut named = BTreeSet::new();
    let mut index = 0;
    while index < types.types.len() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if matches!(types.types[index], Ty::Reference { .. })
            && let DependencyResult::Found(ancestors) =
                walk_hierarchy(types, index, None, headers, budget)?
        {
            for ancestor in ancestors {
                budget.charge(Cost::IrItems, 1)?;
                budget.charge(Cost::IrEdges, 1)?;
                types.relations.push((index, ancestor));
            }
        }
        if matches!(
            declared_type_bounds(types, index, headers, budget)?,
            DependencyResult::Found(true)
        ) {
            budget.charge(Cost::IrItems, 1)?;
            types.admissible.push(index);
        }
        if let (Some(context), Ty::Reference { declaration, .. }) =
            (&types.declaration, &types.types[index])
            && let SymbolRef::Class { owner } = &declaration.member
            && named.insert(owner.0.clone())
        {
            budget.charge(Cost::IrItems, 1)?;
            if let DependencyResult::Found(header) =
                headers.selected_from(&context.loader, &owner.0, budget)?
            {
                let context_owner = match &context.member {
                    SymbolRef::Method { owner, .. } | SymbolRef::Class { owner } => &owner.0,
                    _ => continue,
                };
                let accessible = header.flags & crate::facts::ACC_PUBLIC != 0
                    || (header.loader == context.loader
                        && crate::source_proof::internal_package(&owner.0)
                            == crate::source_proof::internal_package(context_owner));
                if accessible {
                    budget.charge(Cost::IrItems, 1)?;
                    types.spellings.push((
                        owner.clone(),
                        ResolvedMemberRef {
                            loader: header.loader,
                            definition: header.definition,
                            member: SymbolRef::Class {
                                owner: owner.clone(),
                            },
                        },
                    ));
                }
            }
        }
        index += 1;
    }
    Ok(())
}

/// Consume only completed relations from the same bound graph. `false` means this input did not
/// prove assignment, so the caller must retain its baseline or refuse the affected projection.
pub fn proved_assignment(
    types: &DeclaredGenericTypes,
    source: usize,
    target: usize,
    budget: &mut Budget,
) -> Result<bool> {
    fn visit(
        types: &DeclaredGenericTypes,
        source: usize,
        target: usize,
        seen: &mut Vec<(usize, usize)>,
        budget: &mut Budget,
    ) -> Result<bool> {
        budget.poll()?;
        budget.observe_dependency_depth(seen.len() as u64)?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        let (Some(from), Some(to)) = (types.types.get(source), types.types.get(target)) else {
            return Ok(false);
        };
        if source == target {
            return Ok(true);
        }
        if seen.contains(&(source, target)) {
            return Ok(false);
        }
        budget.charge(Cost::IrItems, 1)?;
        seen.push((source, target));
        let result = match (from, to) {
            (Ty::Array(from), Ty::Array(to)) => {
                if matches!(types.types.get(*from), Some(Ty::Base(_)))
                    || matches!(types.types.get(*to), Some(Ty::Base(_)))
                {
                    from == to
                } else {
                    visit(types, *from, *to, seen, budget)?
                }
            }
            (Ty::Variable { .. }, _) => {
                let mut found = false;
                if let Some((_, bounds)) = types
                    .bounds
                    .iter()
                    .find(|(variable, _)| *variable == source)
                {
                    for bound in bounds {
                        if visit(types, *bound, target, seen, budget)? {
                            found = true;
                            break;
                        }
                    }
                }
                found
            }
            (
                Ty::Reference {
                    declaration: source_declaration,
                    arguments: source_arguments,
                },
                Ty::Reference {
                    declaration,
                    arguments,
                },
            ) => {
                if source_declaration == declaration {
                    seen.pop();
                    return Ok(arguments.is_empty() || source_arguments == arguments);
                }
                let mut found = false;
                for (from, ancestor) in &types.relations {
                    budget.charge(Cost::AnalysisSteps, 1)?;
                    if *from != source {
                        continue;
                    }
                    if matches!(types.types.get(*ancestor), Some(Ty::Reference { declaration: actual, arguments: args })
                        if actual == declaration && (arguments.is_empty() || args == arguments))
                    {
                        found = true;
                        break;
                    }
                }
                found
            }
            _ => false,
        };
        seen.pop();
        Ok(result)
    }
    visit(types, source, target, &mut Vec::new(), budget)
}

/// Source spelling uses the same grammar/speller as other Signature consumers, after checking
/// that a variable denotes the visible declaring binder. A shadowed class T cannot be printed
/// as an unrelated method T merely because its raw name is equal.
pub fn spell_declared_type(
    types: &DeclaredGenericTypes,
    node: usize,
    budget: &mut Budget,
) -> Result<Option<String>> {
    fn syntax(
        types: &DeclaredGenericTypes,
        node: usize,
        scope: &[(Vec<u8>, usize)],
        budget: &mut Budget,
        depth: u64,
    ) -> Result<Option<SignatureType>> {
        budget.poll()?;
        budget.observe_dependency_depth(depth)?;
        budget.charge(Cost::IrItems, 1)?;
        Ok(Some(match types.types.get(node) {
            Some(Ty::Base(base)) => SignatureType::Base(*base),
            Some(Ty::Array(element)) => {
                let Some(element) = syntax(types, *element, scope, budget, depth + 1)? else {
                    return Ok(None);
                };
                SignatureType::Array(Box::new(element))
            }
            Some(Ty::Variable { name, .. }) => {
                if scope
                    .iter()
                    .rev()
                    .find(|(visible, _)| visible == &name.0)
                    .map(|(_, variable)| *variable)
                    != Some(node)
                {
                    return Ok(None);
                }
                SignatureType::TypeVariable(name.0.clone())
            }
            Some(Ty::Reference {
                declaration,
                arguments,
            }) => {
                let SymbolRef::Class { owner } = &declaration.member else {
                    return Ok(None);
                };
                let mut bound = false;
                for (name, selected) in &types.spellings {
                    budget.charge(Cost::AnalysisSteps, 1)?;
                    if name == owner {
                        bound = selected == declaration;
                        break;
                    }
                }
                if !bound {
                    return Ok(None);
                }
                // A JVM-selected class is not necessarily nameable by its qualified Java name.
                // Type binders capture the leading identifier, including in static contexts
                // where a class binder itself cannot be used as a source type.
                let Some(qualified) = crate::names::source_class_name(owner) else {
                    return Ok(None);
                };
                let first = qualified
                    .split('.')
                    .next()
                    .expect("a source class name is nonempty");
                for parameter in types
                    .class_parameters
                    .iter()
                    .chain(&types.method_parameters)
                {
                    budget.poll()?;
                    budget.charge(Cost::AnalysisSteps, 1)?;
                    if let Some(Ty::Variable { name, .. }) = types.types.get(*parameter)
                        && crate::names::source_type_parameter_name(&name.0).as_deref()
                            == Some(first)
                    {
                        return Ok(None);
                    }
                }
                let mut args = Vec::new();
                for argument in arguments {
                    let Some(argument) = syntax(types, *argument, scope, budget, depth + 1)? else {
                        return Ok(None);
                    };
                    budget.charge(Cost::IrItems, 1)?;
                    args.push(TypeArgument::Exact(argument));
                }
                SignatureType::Class(ClassType {
                    segments: vec![jarde_reader::signature::ClassTypeSegment {
                        binary_name: owner.0.clone(),
                        arguments: args,
                    }],
                })
            }
            None => return Ok(None),
        }))
    }
    let mut scope = Vec::new();
    let mut erasures = Vec::new();
    // Class binders are visible in a class header and an instance body, not in a static
    // method. A method's own binders remain visible and may shadow a class binder.
    let class_parameters = if types.receiver.is_none()
        && types
            .declaration
            .as_ref()
            .is_some_and(|declaration| matches!(&declaration.member, SymbolRef::Method { .. }))
    {
        &[][..]
    } else {
        types.class_parameters.as_slice()
    };
    for parameter in class_parameters.iter().chain(&types.method_parameters) {
        budget.poll()?;
        let Some(Ty::Variable { name, erasure, .. }) = types.types.get(*parameter) else {
            return Ok(None);
        };
        budget.charge(Cost::IrItems, 2)?;
        scope.push((name.0.clone(), *parameter));
        erasures.push(TypeParameterErasure {
            name: name.0.clone(),
            descriptor: erasure.0.clone(),
        });
    }
    let Some(ty) = syntax(types, node, &scope, budget, 0)? else {
        return Ok(None);
    };
    match crate::generic_source_proof::spell_ordinary_signature_type(&ty, &erasures, budget, 0) {
        Ok(spelling) => Ok(Some(spelling)),
        Err(
            error @ (jarde_reader::error::Error::BudgetExceeded { .. }
            | jarde_reader::error::Error::Cancelled { .. }),
        ) => Err(error),
        Err(_) => Ok(None),
    }
}

/// Spell a method declaration from its already-bound declared type arena.
///
/// This is a header candidate only: it proves no body behavior and publishes nothing. The
/// declaration and method Signature must be the actual selected method retained by the arena;
/// formal names must cover its physical descriptor slots exactly. Unsupported source forms keep
/// the caller's existing descriptor declaration by returning `None`.
pub fn declared_method_declaration(
    types: &DeclaredGenericTypes,
    formal_names: &[(u16, String)],
    raw_name: &[u8],
    access_flags: u16,
    physical_throws: &[String],
    budget: &mut Budget,
) -> Result<Option<String>> {
    use jarde_reader::classfile::{DescriptorKind, descriptor_facts};
    use jarde_reader::signature::TypeParameterErasure;

    const ACC_STATIC: u16 = 0x0008;
    const ACC_VARARGS: u16 = 0x0080;
    const SUPPORTED_FLAGS: u16 =
        0x0001 | 0x0002 | 0x0004 | ACC_STATIC | 0x0010 | 0x0020 | 0x0100 | 0x0400 | 0x0800;

    fn admissible_tree(
        types: &DeclaredGenericTypes,
        node: usize,
        budget: &mut Budget,
        depth: u64,
    ) -> Result<bool> {
        budget.poll()?;
        budget.observe_dependency_depth(depth)?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if !types.admissible.contains(&node) {
            return Ok(false);
        }
        match types.types.get(node) {
            Some(Ty::Array(component)) => admissible_tree(types, *component, budget, depth + 1),
            Some(Ty::Reference { arguments, .. }) => {
                for argument in arguments {
                    if !admissible_tree(types, *argument, budget, depth + 1)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Some(Ty::Base(_) | Ty::Variable { .. }) => Ok(true),
            None => Ok(false),
        }
    }

    budget.poll()?;
    if access_flags & !SUPPORTED_FLAGS != 0 || access_flags & ACC_VARARGS != 0 {
        return Ok(None);
    }
    let raw_name = JvmBytes(raw_name.to_vec());
    let Some(source_name) = crate::names::source_method_name(&raw_name) else {
        return Ok(None);
    };
    let Some(declaration) = types.declaration.as_ref() else {
        return Ok(None);
    };
    let jarde_reader::model::SymbolRef::Method {
        name: declared_name,
        descriptor,
        ..
    } = &declaration.member
    else {
        return Ok(None);
    };
    if declared_name != &raw_name {
        return Ok(None);
    }
    let Some(signature) = types.method_signature.as_ref() else {
        return Ok(None);
    };
    let descriptor_facts = descriptor_facts(&descriptor.0, DescriptorKind::Method)?;
    let Some(expected_slots) = jarde_jvm::method_ir::parameter_positions(
        &descriptor_facts,
        access_flags & ACC_STATIC != 0,
    ) else {
        return Ok(None);
    };
    if formal_names.len() != expected_slots.len()
        || types.parameters.len() != expected_slots.len()
        || signature.parameters.len() != expected_slots.len()
        || types.result.is_some() != signature.result.is_some()
        || signature.type_parameters.len() != types.method_parameters.len()
    {
        return Ok(None);
    }
    let mut seen_slots = BTreeSet::new();
    let mut seen_names = BTreeSet::new();
    let mut arguments = Vec::with_capacity(formal_names.len());
    let mut erasures =
        Vec::with_capacity(types.class_parameters.len() + types.method_parameters.len());
    for index in 0..types.class_parameters.len() + types.method_parameters.len() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        let node = if index < types.class_parameters.len() {
            types.class_parameters[index]
        } else {
            types.method_parameters[index - types.class_parameters.len()]
        };
        let Some(Ty::Variable {
            loader,
            owner: variable_owner,
            method,
            ordinal,
            name,
            erasure,
        }) = types.types.get(node)
        else {
            return Ok(None);
        };
        let expected_ordinal = if index < types.class_parameters.len() {
            index
        } else {
            index - types.class_parameters.len()
        };
        if loader != &declaration.loader
            || variable_owner != &declaration.definition
            || method
                .as_ref()
                .map(|(method_name, _)| method_name.0.as_slice())
                != if index < types.class_parameters.len() {
                    None
                } else {
                    Some(declared_name.0.as_slice())
                }
            || method
                .as_ref()
                .is_some_and(|(_, method_descriptor)| method_descriptor != descriptor)
            || *ordinal != expected_ordinal
        {
            return Ok(None);
        }
        erasures.push(TypeParameterErasure {
            name: name.0.clone(),
            descriptor: erasure.0.clone(),
        });
    }
    for ((slot, name), (expected_slot, (parameter_slot, node))) in formal_names
        .iter()
        .zip(expected_slots.iter().zip(&types.parameters))
    {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if slot != expected_slot
            || parameter_slot != expected_slot
            || !seen_slots.insert(*slot)
            || !crate::is_java_identifier(name)
            || !seen_names.insert(name.as_str())
            || !admissible_tree(types, *node, budget, 0)?
        {
            return Ok(None);
        }
        let Some(spelling) = spell_declared_type(types, *node, budget)? else {
            return Ok(None);
        };
        let signature_spelling = crate::generic_source_proof::spell_ordinary_signature_type(
            &signature.parameters[arguments.len()],
            &erasures,
            budget,
            0,
        )?;
        if spelling != signature_spelling {
            return Ok(None);
        }
        arguments.push(format!("{spelling} {name}"));
    }
    let result = match (types.result, signature.result.as_ref()) {
        (None, None) => "void".to_owned(),
        (Some(result_node), Some(signature_result)) => {
            if !admissible_tree(types, result_node, budget, 0)? {
                return Ok(None);
            }
            let Some(result) = spell_declared_type(types, result_node, budget)? else {
                return Ok(None);
            };
            if result
                != crate::generic_source_proof::spell_ordinary_signature_type(
                    signature_result,
                    &erasures,
                    budget,
                    0,
                )?
            {
                return Ok(None);
            }
            result
        }
        _ => return Ok(None),
    };

    let mut type_parameters = Vec::with_capacity(signature.type_parameters.len());
    for (index, parameter) in signature.type_parameters.iter().enumerate() {
        budget.poll()?;
        let node = types.method_parameters[index];
        let Some(Ty::Variable {
            name,
            method: Some((method_name, method_descriptor)),
            ordinal,
            ..
        }) = types.types.get(node)
        else {
            return Ok(None);
        };
        let Some(parameter_name) = crate::names::source_type_parameter_name(&parameter.name) else {
            return Ok(None);
        };
        if name.0 != parameter.name
            || method_name != declared_name
            || method_descriptor != descriptor
            || *ordinal != index
        {
            return Ok(None);
        }
        let mut bounds = None;
        for (variable, nodes) in &types.bounds {
            budget.charge(Cost::AnalysisSteps, 1)?;
            if *variable == node {
                bounds = Some(nodes);
                break;
            }
        }
        let Some(bounds) = bounds else {
            return Ok(None);
        };
        if bounds.is_empty() {
            type_parameters.push(parameter_name.to_owned());
            continue;
        }
        let mut spelled_bounds = Vec::with_capacity(bounds.len());
        for bound in bounds {
            if !admissible_tree(types, *bound, budget, 0)? {
                return Ok(None);
            }
            let Some(bound) = spell_declared_type(types, *bound, budget)? else {
                return Ok(None);
            };
            spelled_bounds.push(bound);
        }
        type_parameters.push(format!(
            "{} extends {}",
            parameter_name,
            spelled_bounds.join(" & ")
        ));
    }

    let throws = if signature.throws.len() != physical_throws.len() {
        return Ok(None);
    } else {
        let mut spelled = Vec::with_capacity(signature.throws.len());
        for (ty, physical) in signature.throws.iter().zip(physical_throws) {
            let SignatureType::Class(class) = ty else {
                return Ok(None);
            };
            if class.segments.len() != 1 || !class.segments[0].arguments.is_empty() {
                return Ok(None);
            }
            let spelling = crate::generic_source_proof::spell_ordinary_signature_type(
                ty, &erasures, budget, 0,
            )?;
            if &spelling != physical {
                return Ok(None);
            }
            spelled.push(spelling);
        }
        spelled
    };
    Ok(Some(
        crate::generic_source_proof::format_generic_method_header(
            access_flags,
            &type_parameters,
            &result,
            &source_name,
            &arguments,
            &throws,
        ),
    ))
}

/// Spell a class header from the already-bound class Signature facts. This is only a declaration
/// candidate: the caller still owns nesting, class attributes, and joint publication eligibility.
#[allow(clippy::too_many_arguments)]
pub fn declared_class_declaration(
    types: &DeclaredGenericTypes,
    superclass: usize,
    interfaces: &[usize],
    name: &str,
    this_class: &jarde_reader::model::JvmString,
    flags: u16,
    physical_super: &jarde_reader::model::JvmString,
    physical_interfaces: &[jarde_reader::model::JvmString],
    budget: &mut Budget,
) -> Result<Option<(String, Option<String>)>> {
    budget.poll()?;
    let Some(ResolvedMemberRef {
        loader,
        definition,
        member: SymbolRef::Class { owner },
    }) = types.declaration.as_ref()
    else {
        return Ok(None);
    };
    if owner.0.as_slice() != this_class.raw().0.as_slice()
        || crate::names::class_simple_name(&this_class.raw().0).as_deref() != Some(name)
        || !types.method_parameters.is_empty()
        || interfaces.len() != physical_interfaces.len()
    {
        return Ok(None);
    }

    let variable_identity = |node: usize, ordinal: usize| {
        matches!(types.types.get(node), Some(Ty::Variable {
            loader: variable_loader,
            owner: variable_owner,
            method: None,
            ordinal: variable_ordinal,
            ..
        }) if variable_loader == loader && variable_owner == definition && *variable_ordinal == ordinal)
    };
    let mut type_parameters = Vec::with_capacity(types.class_parameters.len());
    for (ordinal, node) in types.class_parameters.iter().copied().enumerate() {
        budget.poll()?;
        if !variable_identity(node, ordinal) || !types.admissible.contains(&node) {
            return Ok(None);
        }
        let Some(Ty::Variable { name: raw_name, .. }) = types.types.get(node) else {
            return Ok(None);
        };
        let Some(parameter_name) = crate::names::source_type_parameter_name(&raw_name.0) else {
            return Ok(None);
        };
        let Some((_, bounds)) = types.bounds.iter().find(|(variable, _)| *variable == node) else {
            return Ok(None);
        };
        if bounds.is_empty() {
            type_parameters.push(parameter_name.to_owned());
            continue;
        }
        let mut spelled_bounds = Vec::with_capacity(bounds.len());
        for bound in bounds {
            budget.poll()?;
            if !types.admissible.contains(bound) {
                return Ok(None);
            }
            let Some(spelling) = spell_declared_type(types, *bound, budget)? else {
                return Ok(None);
            };
            spelled_bounds.push(spelling);
        }
        type_parameters.push(format!(
            "{} extends {}",
            parameter_name,
            spelled_bounds.join(" & ")
        ));
    }
    let parameters = (!type_parameters.is_empty()).then(|| type_parameters.join(", "));

    let parent_spelling = |node: usize,
                           physical: &jarde_reader::model::JvmString,
                           budget: &mut Budget|
     -> Result<Option<String>> {
        if !types.admissible.contains(&node) {
            return Ok(None);
        }
        let Some(Ty::Reference { declaration, .. }) = types.types.get(node) else {
            return Ok(None);
        };
        let SymbolRef::Class { owner } = &declaration.member else {
            return Ok(None);
        };
        if owner.0.as_slice() != physical.raw().0.as_slice() {
            return Ok(None);
        }
        spell_declared_type(types, node, budget)
    };

    let Some(generic_superclass) = parent_spelling(superclass, physical_super, budget)? else {
        return Ok(None);
    };
    let mut generic_interfaces = Vec::with_capacity(interfaces.len());
    for (node, physical) in interfaces.iter().copied().zip(physical_interfaces) {
        let Some(spelling) = parent_spelling(node, physical, budget)? else {
            return Ok(None);
        };
        generic_interfaces.push(spelling);
    }

    let declaration = crate::declaration::class_declaration_with_types(
        name,
        this_class,
        flags,
        Some(physical_super),
        physical_interfaces,
        parameters.as_deref(),
        Some(&generic_superclass),
        Some(&generic_interfaces),
    );
    Ok(declaration.map(|declaration| (declaration, parameters)))
}

/// The unchanged JVM-level type beside a source declaration. Variables use Reader's checked
/// first-bound erasure; no source name is parsed backwards to recover this fact.
pub fn erased_type(
    types: &DeclaredGenericTypes,
    mut node: usize,
    budget: &mut Budget,
) -> Result<Option<crate::ast::Type>> {
    use crate::ast::Type;
    let mut dimensions = 0;
    while let Some(Ty::Array(component)) = types.types.get(node) {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        dimensions += 1;
        budget.observe_dependency_depth(dimensions)?;
        node = *component;
    }
    budget.poll()?;
    budget.charge(Cost::AnalysisSteps, 1)?;
    let result = match types.types.get(node) {
        Some(Ty::Base(base)) => Some(match base {
            b'Z' => Type::Boolean,
            b'B' => Type::Byte,
            b'C' => Type::Char,
            b'S' => Type::Short,
            b'I' => Type::Int,
            b'J' => Type::Long,
            b'F' => Type::Float,
            b'D' => Type::Double,
            _ => return Ok(None),
        }),
        Some(Ty::Reference { declaration, .. }) => match &declaration.member {
            SymbolRef::Class { owner } => {
                crate::build::spell_class_operand(owner).map(Type::Reference)
            }
            _ => None,
        },
        Some(Ty::Variable { erasure, .. }) => jarde_reader::classfile::descriptor_facts(
            &erasure.0,
            jarde_reader::classfile::DescriptorKind::Field,
        )?
        .single()
        .and_then(crate::lambda::type_of_component),
        Some(Ty::Array(_)) => unreachable!(),
        None => None,
    };
    Ok(result.map(|ty| {
        if dimensions == 0 {
            ty
        } else {
            Type::Reference(format!(
                "{}{}",
                ty.spell(),
                "[]".repeat(dimensions as usize)
            ))
        }
    }))
}

pub(crate) fn has_source_arguments(
    types: &DeclaredGenericTypes,
    mut node: usize,
    budget: &mut Budget,
) -> Result<bool> {
    loop {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        match types.types.get(node) {
            Some(Ty::Variable { .. }) => return Ok(true),
            Some(Ty::Reference { arguments, .. }) => return Ok(!arguments.is_empty()),
            Some(Ty::Array(component)) => node = *component,
            Some(Ty::Base(_)) | None => return Ok(false),
        }
    }
}

/// Qualifying a member by its declaring type must not invalidate protected/private access.
/// The original resolver checked the use; this checks the source qualifier added by recovery.
pub(crate) fn field_qualifier_accessible(
    types: &DeclaredGenericTypes,
    member: &DeclaredGenericMemberTypes,
) -> bool {
    let DeclaredGenericMemberTypes::Field {
        declaration,
        access_flags,
        ..
    } = member
    else {
        return false;
    };
    if access_flags & crate::facts::ACC_PUBLIC != 0 {
        return true;
    }
    let Some(method) = &types.declaration else {
        return false;
    };
    if method.loader == declaration.loader && method.definition == declaration.definition {
        return true;
    }
    match (&method.member, &declaration.member) {
        (SymbolRef::Method { owner: caller, .. }, SymbolRef::Field { owner, .. }) => {
            access_flags & crate::facts::ACC_PRIVATE == 0
                && method.loader == declaration.loader
                && crate::source_proof::internal_package(&caller.0)
                    == crate::source_proof::internal_package(&owner.0)
        }
        _ => false,
    }
}

fn assignable<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    source: usize,
    target: usize,
    headers: &mut H,
    budget: &mut Budget,
    ancestors: &mut Vec<(usize, usize)>,
) -> std::result::Result<DependencyResult<bool>, H::Error> {
    budget.poll()?;
    budget.observe_dependency_depth(ancestors.len() as u64)?;
    budget.charge(Cost::AnalysisSteps, 1)?;
    let (Some(source_type), Some(target_type)) = (types.types.get(source), types.types.get(target))
    else {
        return Ok(DependencyResult::Unknown);
    };
    if source == target {
        return Ok(DependencyResult::Found(true));
    }
    if ancestors.contains(&(source, target)) {
        return Ok(DependencyResult::Unknown);
    }
    budget.charge(Cost::IrItems, 1)?;
    ancestors.push((source, target));
    let result = match (source_type, target_type) {
        (Ty::Array(source_component), Ty::Array(target_component)) => {
            let (source_component, target_component) = (*source_component, *target_component);
            if matches!(types.types.get(source_component), Some(Ty::Base(_)))
                || matches!(types.types.get(target_component), Some(Ty::Base(_)))
            {
                DependencyResult::Found(source_component == target_component)
            } else {
                assignable(
                    types,
                    source_component,
                    target_component,
                    headers,
                    budget,
                    ancestors,
                )?
            }
        }
        (Ty::Variable { .. }, _) => {
            let Some((_, bounds)) = types
                .bounds
                .iter()
                .find(|(variable, _)| *variable == source)
            else {
                ancestors.pop();
                return Ok(DependencyResult::Unknown);
            };
            budget.charge(Cost::IrItems, bounds.len() as u64)?;
            let bounds = bounds.clone();
            let mut unknown = false;
            let mut matched = false;
            for bound in bounds {
                match assignable(types, bound, target, headers, budget, ancestors)? {
                    DependencyResult::Found(true) => {
                        matched = true;
                        break;
                    }
                    DependencyResult::Found(false) => {}
                    DependencyResult::Missing
                    | DependencyResult::Unknown
                    | DependencyResult::Ambiguous => unknown = true,
                }
            }
            if matched {
                DependencyResult::Found(true)
            } else if unknown {
                DependencyResult::Unknown
            } else {
                DependencyResult::Found(false)
            }
        }
        (
            Ty::Reference { .. },
            Ty::Reference {
                declaration,
                arguments,
            },
        ) => {
            let declaration = declaration.clone();
            budget.charge(Cost::IrItems, arguments.len() as u64)?;
            let arguments = arguments.clone();
            match declared_supertype(types, source, &declaration, headers, budget)? {
                DependencyResult::Found(application) => {
                    let Ty::Reference {
                        arguments: actual, ..
                    } = &types.types[application]
                    else {
                        unreachable!()
                    };
                    DependencyResult::Found(arguments.is_empty() || *actual == arguments)
                }
                DependencyResult::Missing => DependencyResult::Found(false),
                DependencyResult::Ambiguous => DependencyResult::Ambiguous,
                DependencyResult::Unknown => DependencyResult::Unknown,
            }
        }
        // A concrete type cannot be assigned to an unrelated declaration variable. Primitive
        // conversions retain the existing descriptor planner's separate rules.
        _ => DependencyResult::Found(false),
    };
    ancestors.pop();
    Ok(result)
}

/// Fields that consume their declaring class's variable scope are part of a new class Header
/// transaction even when no Code reads them. Reader has checked these variables against the
/// actual class scope; unavailable Signature metadata is not an empty dependency set.
pub fn class_scope_field_dependencies(
    header: &ReferenceClassHeader,
    budget: &mut Budget,
) -> Result<Option<Vec<usize>>> {
    fn uses_variable(ty: &SignatureType, budget: &mut Budget, depth: u64) -> Result<bool> {
        budget.poll()?;
        budget.observe_dependency_depth(depth)?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        match ty {
            SignatureType::TypeVariable(_) => Ok(true),
            SignatureType::Array(component) => uses_variable(component, budget, depth + 1),
            SignatureType::Class(class) => {
                for segment in &class.segments {
                    for argument in &segment.arguments {
                        match argument {
                            TypeArgument::Exact(ty)
                            | TypeArgument::Extends(ty)
                            | TypeArgument::Super(ty)
                                if uses_variable(ty, budget, depth + 1)? =>
                            {
                                return Ok(true);
                            }
                            _ => budget.poll()?,
                        }
                    }
                }
                Ok(false)
            }
            SignatureType::Base(_) => Ok(false),
        }
    }
    budget.poll()?;
    let Some(signatures) = &header.generic_signatures else {
        return Ok(None);
    };
    if signatures
        .class
        .as_ref()
        .is_none_or(|(signature, _)| signature.type_parameters.is_empty())
    {
        return Ok(Some(Vec::new()));
    }
    let mut fields = Vec::new();
    for (index, signature) in signatures.fields.iter().enumerate() {
        budget.charge(Cost::AnalysisSteps, 1)?;
        if let Some(signature) = signature
            && uses_variable(&signature.ty, budget, 0)?
        {
            budget.charge(Cost::IrItems, 1)?;
            fields.push(index);
        }
    }
    Ok(Some(fields))
}

/// Bind one field from the selected class declaration without inventing a method/BCI context.
/// Reader has checked erasure. The caller proves relations/bounds and spelling after binding
/// the entire field cohort, before publishing any declaration.
pub fn bind_declared_field_type<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    header: &ReferenceClassHeader,
    index: usize,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<usize>, H::Error> {
    let Some(actual) = types.declaration.as_ref() else {
        return Ok(DependencyResult::Unknown);
    };
    let SymbolRef::Class { owner } = &actual.member else {
        return Ok(DependencyResult::Unknown);
    };
    if actual.loader != header.loader || actual.definition != header.definition {
        return Ok(DependencyResult::Unknown);
    }
    let selected = match headers.selected_generic_from(&actual.loader, &owner.0, budget)? {
        DependencyResult::Found(selected)
            if selected.loader == header.loader
                && selected.definition == header.definition
                && selected.generic_signatures == header.generic_signatures
                && selected.fields == header.fields =>
        {
            selected
        }
        DependencyResult::Found(_) | DependencyResult::Unknown => {
            return Ok(DependencyResult::Unknown);
        }
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
    };

    let Some(field) = selected.fields.get(index) else {
        return Ok(DependencyResult::Unknown);
    };
    if selected
        .fields
        .iter()
        .enumerate()
        .any(|(other, candidate)| {
            other != index
                && candidate.name.raw() == field.name.raw()
                && candidate.descriptor.raw() == field.descriptor.raw()
        })
    {
        return Ok(DependencyResult::Ambiguous);
    }
    let Some(signatures) = &selected.generic_signatures else {
        return Ok(DependencyResult::Unknown);
    };
    if signatures.fields.len() != selected.fields.len() {
        return Ok(DependencyResult::Unknown);
    }
    let Some(signature) = signatures.fields.get(index).and_then(Option::as_ref) else {
        return Ok(DependencyResult::Unknown);
    };

    let class_formals = signatures.class.as_ref().map_or(&[][..], |(signature, _)| {
        signature.type_parameters.as_slice()
    });
    if class_formals.len() != types.class_parameters.len() {
        return Ok(DependencyResult::Unknown);
    }
    let mut variables = Vec::with_capacity(class_formals.len());
    for (formal, node) in class_formals.iter().zip(&types.class_parameters) {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        let Some(Ty::Variable { name, .. }) = types.types.get(*node) else {
            return Ok(DependencyResult::Unknown);
        };
        if name.0 != formal.name {
            return Ok(DependencyResult::Unknown);
        }
        variables.push((formal.name.clone(), *node));
    }
    if field.access_flags & crate::facts::ACC_STATIC != 0 {
        variables.clear();
    }

    let node = match bind_declared_type(
        types,
        &signature.ty,
        &header.loader,
        &variables,
        headers,
        budget,
    )? {
        DependencyResult::Found(node) => node,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    Ok(DependencyResult::Found(node))
}

/// Instantiate the original resolved member in a proved receiver application. Class variables
/// are substituted by arena index; method inference/capture has no answer in this declared-only
/// domain. The physical descriptor is retained in `declaration` beside the source type.
pub fn declared_member_types<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    source: usize,
    declaration: &ResolvedMemberRef,
    bci: u32,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<DeclaredGenericMemberTypes>, H::Error> {
    use jarde_reader::signature::parse_method_signature;
    let application = match declared_supertype(types, source, declaration, headers, budget)? {
        DependencyResult::Found(application) => application,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    let header = match selected_node(types, application, headers, budget)? {
        DependencyResult::Found(header) => header,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    let Some(signatures) = &header.generic_signatures else {
        return Ok(DependencyResult::Unknown);
    };
    let Ty::Reference { arguments, .. } = &types.types[application] else {
        unreachable!()
    };
    let parameters = signatures.class.as_ref().map_or(&[][..], |(signature, _)| {
        signature.type_parameters.as_slice()
    });
    let raw_application = !parameters.is_empty() && arguments.is_empty();
    if !raw_application && arguments.len() != parameters.len() {
        return Ok(DependencyResult::Unknown);
    }
    budget.charge(Cost::IrItems, parameters.len() as u64)?;
    let variables: Vec<_> = parameters
        .iter()
        .zip(arguments)
        .map(|(parameter, argument)| (parameter.name.clone(), *argument))
        .collect();
    let (name, descriptor, fields) = match &declaration.member {
        SymbolRef::Field {
            name, descriptor, ..
        } => (name, descriptor, true),
        SymbolRef::Method {
            name, descriptor, ..
        } => (name, descriptor, false),
        _ => return Ok(DependencyResult::Unknown),
    };
    let table = if fields {
        &header.fields
    } else {
        &header.methods
    };
    let mut selected = None;
    for (index, member) in table.iter().enumerate() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if member.name.raw() == name && member.descriptor.raw() == descriptor {
            if selected.is_some() {
                return Ok(DependencyResult::Ambiguous);
            }
            selected = Some(index);
        }
    }
    let Some(index) = selected else {
        return Ok(DependencyResult::Unknown);
    };
    let is_static = table[index].access_flags & crate::facts::ACC_STATIC != 0;
    // JLS 4.8 erases non-static members declared by the raw class, including a method's
    // own type parameters. Static declarations retain their declaration Signature.
    // The hierarchy walk has already found the actual declaring class application.
    let erased_member = raw_application && !is_static;
    let variables = if is_static || erased_member {
        &[][..]
    } else {
        variables.as_slice()
    };
    let syntax = if erased_member {
        if fields {
            budget.charge(Cost::IrItems, descriptor.0.len() as u64 + 2)?;
            let mut wrapped = b"()".to_vec();
            wrapped.extend_from_slice(&descriptor.0);
            parse_method_signature(&wrapped, budget)?
        } else {
            parse_method_signature(&descriptor.0, budget)?
        }
    } else if fields {
        if let Some(signature) = &signatures.fields[index] {
            charge_syntax(&signature.ty, budget)?;
            jarde_reader::signature::MethodSignature {
                type_parameters: Vec::new(),
                parameters: Vec::new(),
                result: Some(signature.ty.clone()),
                throws: Vec::new(),
            }
        } else {
            budget.charge(Cost::IrItems, descriptor.0.len() as u64 + 2)?;
            let mut wrapped = b"()".to_vec();
            wrapped.extend_from_slice(&descriptor.0);
            parse_method_signature(&wrapped, budget)?
        }
    } else if let Some((signature, _)) = &signatures.methods[index] {
        if !signature.type_parameters.is_empty() {
            return Ok(DependencyResult::Unknown);
        }
        budget.charge(
            Cost::IrItems,
            1 + signature.parameters.len() as u64 + signature.throws.len() as u64,
        )?;
        for ty in signature
            .parameters
            .iter()
            .chain(&signature.result)
            .chain(&signature.throws)
        {
            charge_syntax(ty, budget)?;
        }
        signature.clone()
    } else {
        parse_method_signature(&descriptor.0, budget)?
    };
    let mut parameters = Vec::new();
    let mut result = None;
    let mut throws = Vec::new();
    for (position, ty) in syntax
        .parameters
        .iter()
        .enumerate()
        .chain(syntax.result.iter().map(|ty| (syntax.parameters.len(), ty)))
        .chain(
            syntax
                .throws
                .iter()
                .enumerate()
                .map(|(i, ty)| (syntax.parameters.len() + 1 + i, ty)),
        )
    {
        let ty = match bind_declared_type(types, ty, &header.loader, variables, headers, budget)? {
            DependencyResult::Found(ty) => ty,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
        budget.charge(Cost::IrItems, 1)?;
        budget.charge(Cost::IrEdges, 1)?;
        if position < syntax.parameters.len() {
            parameters.push(ty);
        } else if position == syntax.parameters.len() {
            result = Some(ty);
        } else {
            throws.push(ty);
        }
    }
    let (erased_parameters, erased_result) = match bind_erased_member_types(
        types,
        &descriptor.0,
        fields,
        &header.loader,
        headers,
        budget,
    )? {
        DependencyResult::Found(bound) => bound,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    Ok(DependencyResult::Found(if fields {
        let Some(ty) = result else {
            return Ok(DependencyResult::Unknown);
        };
        DeclaredGenericMemberTypes::Field {
            bci,
            declaration: declaration.clone(),
            access_flags: table[index].access_flags,
            application,
            ty,
            erased_ty: erased_result.expect("a parsed field descriptor has its field type"),
        }
    } else {
        DeclaredGenericMemberTypes::Method {
            bci,
            declaration: declaration.clone(),
            application,
            parameters,
            erased_parameters,
            result,
            erased_result,
            throws,
        }
    }))
}

fn bind_erased_member_types<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    descriptor: &[u8],
    field: bool,
    loader: &LoaderId,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<(Vec<usize>, Option<usize>)>, H::Error> {
    let wrapped;
    let descriptor = if field {
        budget.charge(Cost::IrItems, descriptor.len() as u64 + 2)?;
        wrapped = [b"()".as_slice(), descriptor].concat();
        wrapped.as_slice()
    } else {
        descriptor
    };
    let syntax = jarde_reader::signature::parse_method_signature(descriptor, budget)?;
    let mut parameters = Vec::new();
    let mut result = None;
    for (position, ty) in syntax
        .parameters
        .iter()
        .enumerate()
        .chain(syntax.result.iter().map(|ty| (syntax.parameters.len(), ty)))
    {
        let node = match bind_declared_type(types, ty, loader, &[], headers, budget)? {
            DependencyResult::Found(node) => node,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
        budget.charge(Cost::IrItems, 1)?;
        if position < syntax.parameters.len() {
            parameters.push(node);
        } else {
            result = Some(node);
        }
    }
    Ok(DependencyResult::Found((parameters, result)))
}

/// Prove the source lookup after qualifying a call by its actual declaring application.
/// This finite domain accepts a single substituted fixed-arity signature. Competing overloads,
/// method inference and source-hidden bridges need a broader class transaction and remain
/// unproved here. Walking the same bound hierarchy preserves loader identities and diamonds.
pub fn declared_call_binding<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    member: &DeclaredGenericMemberTypes,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<bool>, H::Error> {
    const ACC_VARARGS: u16 = 0x0080;
    let DeclaredGenericMemberTypes::Method {
        declaration,
        application,
        parameters,
        bci,
        ..
    } = member
    else {
        return Ok(DependencyResult::Unknown);
    };
    let SymbolRef::Method {
        name, descriptor, ..
    } = &declaration.member
    else {
        return Ok(DependencyResult::Unknown);
    };
    budget.charge(Cost::IrItems, descriptor.0.len() as u64)?;
    let original_descriptor = jarde_reader::classfile::descriptor_facts(
        &descriptor.0,
        jarde_reader::classfile::DescriptorKind::Method,
    )?;
    let applications = match walk_hierarchy(types, *application, None, headers, budget)? {
        DependencyResult::Found(applications) => applications,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    let mut original = false;
    for application in applications {
        let Ty::Reference {
            declaration: class, ..
        } = &types.types[application]
        else {
            unreachable!()
        };
        let SymbolRef::Class { owner } = &class.member else {
            return Ok(DependencyResult::Unknown);
        };
        let owner = owner.clone();
        let header = match selected_node(types, application, headers, budget)? {
            DependencyResult::Found(header) => header,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
        for candidate in &header.methods {
            budget.poll()?;
            budget.charge(Cost::AnalysisSteps, 1)?;
            if candidate.name.raw() != name {
                continue;
            }
            let actual = header.loader == declaration.loader
                && header.definition == declaration.definition
                && candidate.descriptor.raw() == descriptor;
            // Non-public declarations can affect lookup, while bridge hiding belongs to the
            // joint class projection. Neither is silently treated as an absent overload.
            if candidate.access_flags & (crate::facts::ACC_BRIDGE | crate::facts::ACC_SYNTHETIC)
                != 0
                || (actual && candidate.access_flags & crate::facts::ACC_PUBLIC == 0)
            {
                return Ok(DependencyResult::Found(false));
            }
            budget.charge(Cost::IrItems, candidate.descriptor.raw().0.len() as u64)?;
            let facts = jarde_reader::classfile::descriptor_facts(
                &candidate.descriptor.raw().0,
                jarde_reader::classfile::DescriptorKind::Method,
            )?;
            if facts.parameters().len() != parameters.len() {
                if candidate.access_flags & ACC_VARARGS != 0 {
                    return Ok(DependencyResult::Found(false));
                }
                continue;
            }
            // Substitution can make two originally different overloads present identical
            // source formals (C<T>.set(T) / set(String), at T=String). Equality of those
            // substituted types cannot prove which physical descriptor javac will choose.
            if facts.parameters() != original_descriptor.parameters() {
                return Ok(DependencyResult::Found(false));
            }
            let candidate_ref = ResolvedMemberRef {
                loader: header.loader.clone(),
                definition: header.definition.clone(),
                member: SymbolRef::Method {
                    owner: owner.clone(),
                    name: name.clone(),
                    descriptor: candidate.descriptor.raw().clone(),
                },
            };
            match declared_member_types(types, application, &candidate_ref, *bci, headers, budget)?
            {
                DependencyResult::Found(DeclaredGenericMemberTypes::Method {
                    parameters: candidate_parameters,
                    ..
                }) if candidate_parameters == *parameters => {}
                DependencyResult::Found(_) => return Ok(DependencyResult::Found(false)),
                DependencyResult::Missing => return Ok(DependencyResult::Missing),
                DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
                DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
            }
            original |= actual;
        }
    }
    Ok(DependencyResult::Found(original))
}

/// Establish this method's own declaration in the bound domain. This does not publish its
/// Signature: the class transaction still has to admit every affected write and use.
pub fn declared_method_types<H: RecoveryHeaders>(
    ir: &jarde_jvm::method_ir::MethodIr,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<DeclaredGenericTypes>, H::Error> {
    use jarde_reader::{
        classfile::{DescriptorKind, descriptor_facts},
        signature::parse_method_signature,
    };
    let Some(declaration) = ir.declaration() else {
        return Ok(DependencyResult::Unknown);
    };
    let loader = headers.caller_loader().clone();
    let header =
        match headers.selected_generic_from(&loader, &declaration.class_name().0, budget)? {
            DependencyResult::Found(header)
                if header.definition == declaration.identity().owner =>
            {
                header
            }
            DependencyResult::Found(_) | DependencyResult::Unknown => {
                return Ok(DependencyResult::Unknown);
            }
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        };
    let Some(signatures) = &header.generic_signatures else {
        return Ok(DependencyResult::Unknown);
    };
    let mut types = DeclaredGenericTypes::default();
    let class_variables = if let Some((signature, proof)) = &signatures.class {
        match bind_declared_parameters(
            &mut types,
            &header,
            None,
            &signature.type_parameters,
            &proof.type_parameters,
            &[],
            headers,
            budget,
        )? {
            DependencyResult::Found(variables) => variables,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        }
    } else {
        Vec::new()
    };
    budget.charge(Cost::IrItems, class_variables.len() as u64)?;
    types.class_parameters = class_variables.iter().map(|(_, node)| *node).collect();
    let identity = declaration.identity();
    let mut selected = None;
    for (index, member) in header.methods.iter().enumerate() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if member.name.raw() == &identity.name && member.descriptor.raw() == &identity.descriptor {
            if selected.is_some() {
                return Ok(DependencyResult::Ambiguous);
            }
            selected = Some(index);
        }
    }
    let Some(index) = selected else {
        return Ok(DependencyResult::Unknown);
    };
    let erased;
    types.declaration = Some(ResolvedMemberRef {
        loader: header.loader.clone(),
        definition: header.definition.clone(),
        member: SymbolRef::Method {
            owner: declaration.class_name().clone(),
            name: identity.name.clone(),
            descriptor: identity.descriptor.clone(),
        },
    });
    let (syntax, variables) = if let Some((signature, proof)) = &signatures.methods[index] {
        let variables = match bind_declared_parameters(
            &mut types,
            &header,
            Some((&identity.name.0, &identity.descriptor.0)),
            &signature.type_parameters,
            &proof.type_parameters,
            &class_variables,
            headers,
            budget,
        )? {
            DependencyResult::Found(variables) => variables,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
        budget.charge(Cost::IrItems, signature.type_parameters.len() as u64)?;
        types.method_parameters = variables[class_variables.len()..]
            .iter()
            .map(|(_, node)| *node)
            .collect();
        for ty in signature
            .parameters
            .iter()
            .chain(&signature.result)
            .chain(&signature.throws)
        {
            charge_syntax(ty, budget)?;
        }
        for parameter in &signature.type_parameters {
            for bound in parameter
                .class_bound
                .iter()
                .chain(&parameter.interface_bounds)
            {
                charge_syntax(bound, budget)?;
            }
        }
        types.method_signature = Some(signature.clone());
        (signature, variables)
    } else {
        erased = parse_method_signature(&identity.descriptor.0, budget)?;
        (&erased, class_variables)
    };
    let is_static = header.methods[index].access_flags & crate::facts::ACC_STATIC != 0;
    if !is_static {
        let owner = Ty::Reference {
            declaration: ResolvedMemberRef {
                loader: header.loader.clone(),
                definition: header.definition.clone(),
                member: SymbolRef::Class {
                    owner: declaration.class_name().clone(),
                },
            },
            arguments: types.class_parameters.clone(),
        };
        budget.charge(Cost::IrItems, types.class_parameters.len() as u64)?;
        budget.charge(Cost::IrEdges, types.class_parameters.len() as u64)?;
        types.receiver = Some(intern(&mut types, owner, budget)?);
    }
    let descriptor = descriptor_facts(&identity.descriptor.0, DescriptorKind::Method)?;
    let Some(slots) = jarde_jvm::method_ir::parameter_positions(&descriptor, is_static) else {
        return Ok(DependencyResult::Unknown);
    };
    if slots.len() != syntax.parameters.len() {
        return Ok(DependencyResult::Unknown);
    }
    for (slot, ty) in slots.into_iter().zip(&syntax.parameters) {
        let node = match bind_declared_type(
            &mut types,
            ty,
            &header.loader,
            &variables,
            headers,
            budget,
        )? {
            DependencyResult::Found(node) => node,
            DependencyResult::Missing => return Ok(DependencyResult::Missing),
            DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
            DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
        };
        budget.charge(Cost::IrItems, 1)?;
        types.parameters.push((slot, node));
    }
    if let Some(ty) = &syntax.result {
        types.result = Some(
            match bind_declared_type(&mut types, ty, &header.loader, &variables, headers, budget)? {
                DependencyResult::Found(node) => node,
                DependencyResult::Missing => return Ok(DependencyResult::Missing),
                DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
                DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
            },
        );
    }
    types.erased_result = match bind_erased_member_types(
        &mut types,
        &identity.descriptor.0,
        false,
        &header.loader,
        headers,
        budget,
    )? {
        DependencyResult::Found((_, result)) => result,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    Ok(DependencyResult::Found(types))
}

fn propagate_values(
    types: &DeclaredGenericTypes,
    ssa: &jarde_jvm::method_ir::SsaTable,
    flows: &crate::assignability::ReferenceBounds,
    seeds: &BTreeMap<jarde_jvm::method_ir::ValueId, BTreeSet<usize>>,
    budget: &mut Budget,
) -> Result<BTreeMap<jarde_jvm::method_ir::ValueId, BTreeSet<usize>>> {
    use jarde_jvm::method_ir::{Value, ValueId};
    let mut values = BTreeMap::<ValueId, BTreeSet<usize>>::new();
    let mut rooted = BTreeSet::new();
    let mut blocked = BTreeSet::new();
    let mut readers = BTreeMap::<ValueId, Vec<(ValueId, u8)>>::new();
    for (id, value) in ssa.values_with_ids() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if !matches!(value.ty(), Value::Ref(_) | Value::Null) {
            continue;
        }
        budget.charge(Cost::IrItems, 1)?;
        if let Some(seed) = seeds.get(&id) {
            budget.charge(Cost::IrItems, seed.len() as u64)?;
            values.insert(id, seed.clone());
            rooted.insert(id);
        } else {
            values.insert(id, BTreeSet::new());
            if flows.parents_of(id).is_empty() {
                blocked.insert(id);
            }
        }
        for (parent, projection) in flows.parents_of(id) {
            budget.charge(Cost::IrEdges, 1)?;
            budget.charge(Cost::IrItems, 1)?;
            readers.entry(*parent).or_default().push((id, *projection));
        }
    }
    for (parent, children) in &readers {
        if !values.contains_key(parent) {
            for (child, _) in children {
                blocked.insert(*child);
            }
        }
    }
    budget.charge(Cost::IrItems, 2 * values.len() as u64)?;
    let mut pending: VecDeque<_> = values.keys().copied().collect();
    let mut queued: BTreeSet<_> = values.keys().copied().collect();
    while let Some(value) = pending.pop_front() {
        queued.remove(&value);
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        budget.charge(Cost::IrItems, values[&value].len() as u64)?;
        let source = values[&value].clone();
        for (child, projection) in readers.get(&value).into_iter().flatten() {
            budget.poll()?;
            budget.charge(Cost::AnalysisSteps, 1)?;
            let mut changed = false;
            if rooted.contains(&value) {
                changed |= rooted.insert(*child);
            }
            if blocked.contains(&value) {
                changed |= blocked.insert(*child);
            }
            for node in &source {
                budget.charge(Cost::AnalysisSteps, 1)?;
                let projected = if *projection == 0 {
                    Some(*node)
                } else {
                    match types.types.get(*node) {
                        Some(Ty::Array(component)) if *projection == 1 => Some(*component),
                        _ => None,
                    }
                };
                let Some(node) = projected else {
                    changed |= blocked.insert(*child);
                    continue;
                };
                let target = values
                    .get_mut(child)
                    .expect("SSA reference dependency exists");
                if !target.contains(&node) {
                    budget.charge(Cost::IrItems, 1)?;
                    target.insert(node);
                    changed = true;
                }
            }
            if changed && queued.insert(*child) {
                budget.charge(Cost::IrItems, 1)?;
                pending.push_back(*child);
            }
        }
    }
    values.retain(|value, _| rooted.contains(value) && !blocked.contains(value));
    Ok(values)
}

fn runtime_type<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    raw: &[u8],
    loader: &LoaderId,
    class_operand: bool,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<usize>, H::Error> {
    let syntax = if class_operand && raw.first() != Some(&b'[') {
        budget.charge(Cost::IrItems, 2)?;
        SignatureType::Class(ClassType {
            segments: vec![jarde_reader::signature::ClassTypeSegment {
                binary_name: raw.to_vec(),
                arguments: Vec::new(),
            }],
        })
    } else {
        match jarde_reader::signature::parse_field_signature(raw, budget) {
            Ok(signature) => signature.ty,
            Err(
                error @ (jarde_reader::error::Error::BudgetExceeded { .. }
                | jarde_reader::error::Error::Cancelled { .. }),
            ) => return Err(error.into()),
            Err(_) => return Ok(DependencyResult::Unknown),
        }
    };
    bind_declared_type(types, &syntax, loader, &[], headers, budget)
}

fn runtime_erasure<H: RecoveryHeaders>(
    types: &mut DeclaredGenericTypes,
    node: usize,
    headers: &mut H,
    budget: &mut Budget,
    depth: u64,
) -> std::result::Result<DependencyResult<usize>, H::Error> {
    budget.poll()?;
    budget.observe_dependency_depth(depth)?;
    budget.charge(Cost::AnalysisSteps, 1)?;
    match types.types.get(node) {
        Some(Ty::Base(_)) => Ok(DependencyResult::Found(node)),
        Some(Ty::Reference {
            declaration,
            arguments,
        }) => {
            if arguments.is_empty() {
                return Ok(DependencyResult::Found(node));
            }
            let ty = Ty::Reference {
                declaration: declaration.clone(),
                arguments: Vec::new(),
            };
            Ok(DependencyResult::Found(intern(types, ty, budget)?))
        }
        Some(Ty::Array(component)) => {
            let component = *component;
            let erased = match runtime_erasure(types, component, headers, budget, depth + 1)? {
                DependencyResult::Found(erased) => erased,
                _ => return Ok(DependencyResult::Unknown),
            };
            budget.charge(Cost::IrEdges, 1)?;
            Ok(DependencyResult::Found(intern(
                types,
                Ty::Array(erased),
                budget,
            )?))
        }
        Some(Ty::Variable {
            loader, erasure, ..
        }) => {
            let (loader, erasure) = (loader.clone(), erasure.clone());
            runtime_type(types, &erasure.0, &loader, false, headers, budget)
        }
        None => Ok(DependencyResult::Unknown),
    }
}

fn runtime_widens<H: RecoveryHeaders>(
    types: &DeclaredGenericTypes,
    source: usize,
    target: usize,
    headers: &mut H,
    budget: &mut Budget,
    depth: u64,
) -> std::result::Result<bool, H::Error> {
    budget.poll()?;
    budget.observe_dependency_depth(depth)?;
    budget.charge(Cost::AnalysisSteps, 1)?;
    if source == target {
        return Ok(true);
    }
    match (types.types.get(source), types.types.get(target)) {
        (
            Some(Ty::Reference {
                declaration: from, ..
            }),
            Some(Ty::Reference {
                declaration: to, ..
            }),
        ) => crate::source_proof::selected_header_widens(from, to, headers, budget),
        (Some(Ty::Array(from)), Some(Ty::Array(to))) => {
            if matches!(types.types.get(*from), Some(Ty::Base(_)))
                || matches!(types.types.get(*to), Some(Ty::Base(_)))
            {
                return Ok(from == to);
            }
            runtime_widens(types, *from, *to, headers, budget, depth + 1)
        }
        (Some(Ty::Array(_)), Some(Ty::Reference { .. })) => {
            let (Some(from), Some(to)) = (
                erased_type(types, source, budget)?,
                erased_type(types, target, budget)?,
            ) else {
                return Ok(false);
            };
            Ok(crate::assignability::reference_intrinsically_assignable(
                from.spell(),
                to.spell(),
            ))
        }
        _ => Ok(false),
    }
}

/// A source cast may be added only when every original runtime producer already fits its
/// erasure. Signature substitution alone never supplies this proof.
pub fn proved_runtime_upcast(
    types: &DeclaredGenericTypes,
    value_index: usize,
    target: usize,
    budget: &mut Budget,
) -> Result<bool> {
    budget.poll()?;
    let Some(Some(bounds)) = types.runtime_values.get(value_index) else {
        return Ok(false);
    };
    for bound in bounds {
        let mut proved = false;
        for pair in &types.runtime_upcasts {
            budget.charge(Cost::AnalysisSteps, 1)?;
            if pair == &(*bound, target) {
                proved = true;
                break;
            }
        }
        if !proved {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Source conversion may express an unchecked raw application only at an already-declared
/// consumer. The selected hierarchy must actually contain the raw target application; a
/// different nonempty argument list never becomes raw merely to make the source compile.
pub(crate) fn proved_source_conversion(
    types: &DeclaredGenericTypes,
    mut source: usize,
    mut target: usize,
    budget: &mut Budget,
) -> Result<bool> {
    if proved_assignment(types, source, target, budget)? {
        return Ok(true);
    }
    let mut depth = 0;
    loop {
        budget.poll()?;
        budget.observe_dependency_depth(depth)?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        match (types.types.get(source), types.types.get(target)) {
            (Some(Ty::Array(from)), Some(Ty::Array(to))) => {
                source = *from;
                target = *to;
                depth += 1;
            }
            (
                Some(Ty::Reference {
                    declaration: from,
                    arguments: args,
                }),
                Some(Ty::Reference {
                    declaration: to,
                    arguments,
                }),
            ) if !arguments.is_empty() => {
                if from == to && args.is_empty() {
                    return Ok(true);
                }
                for (start, ancestor) in &types.relations {
                    budget.charge(Cost::AnalysisSteps, 1)?;
                    if *start == source
                        && matches!(types.types.get(*ancestor), Some(Ty::Reference { declaration, arguments }) if declaration == to && arguments.is_empty())
                    {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            _ => return Ok(false),
        }
    }
}

/// Adding a source cast needs a declared assignment/unchecked raw conversion and absence of
/// any new runtime type check. Unknown producers cannot manufacture a generic application.
pub(crate) fn proved_declared_conversion(
    types: &DeclaredGenericTypes,
    value_index: usize,
    target: usize,
    budget: &mut Budget,
) -> Result<bool> {
    let Some(Some(bounds)) = types.values.get(value_index) else {
        return Ok(false);
    };
    for source in bounds {
        if !proved_source_conversion(types, *source, target, budget)? {
            return Ok(false);
        }
    }
    proved_runtime_upcast(types, value_index, target, budget)
}

/// Decide whether this method has a declared generic producer or a live reference member
/// position to resolve. Primitive-only bodies and bare no-argument constructor prologues have
/// neither; requesting their generic hierarchy would spend Header quota for no source decision.
#[doc(hidden)]
pub fn needs_declared_value_types<'a>(
    ir: &jarde_jvm::method_ir::MethodIr,
    mut known_header: impl FnMut(&[u8]) -> Option<&'a ReferenceClassHeader>,
    budget: &mut Budget,
) -> Result<bool> {
    use jarde_reader::classfile::{CpEntryKind, DescriptorKind, cp_entry, descriptor_facts};
    budget.poll()?;
    let Some(declaration) = ir.declaration() else {
        return Ok(false);
    };
    let Some(methods) = ir.class_methods() else {
        return Ok(true);
    };
    for method in methods {
        budget.charge(Cost::AnalysisSteps, 1)?;
        if method.name.raw() == declaration.name()
            && method.descriptor.raw() == declaration.descriptor()
            && method
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Signature")
        {
            return Ok(true);
        }
    }
    if declaration.access_flags() & crate::facts::ACC_STATIC == 0
        && ir.class_attributes().is_some_and(|attributes| {
            attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Signature")
        })
    {
        return Ok(true);
    }
    let Some(code) = ir.code() else {
        return Ok(false);
    };
    for instruction in &code.instructions {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = cp_entry(ir.constant_pool(), index) else {
            return Ok(true);
        };
        let (owner, name, descriptor, kind) = match &entry.kind {
            CpEntryKind::FieldRef {
                owner,
                name,
                descriptor,
                ..
            } => (owner, name, descriptor, DescriptorKind::Field),
            CpEntryKind::MethodRef {
                owner,
                name,
                descriptor,
                ..
            }
            | CpEntryKind::InterfaceMethodRef {
                owner,
                name,
                descriptor,
                ..
            } => (owner, name, descriptor, DescriptorKind::Method),
            _ => continue,
        };
        if let Some(header) = known_header(&owner.0)
            && let Some(signatures) = &header.generic_signatures
        {
            let members = if kind == DescriptorKind::Field {
                &header.fields
            } else {
                &header.methods
            };
            let mut direct = members.iter().enumerate().filter(|(_, member)| {
                member.name.raw() == name && member.descriptor.raw() == descriptor
            });
            if let Some((index, _)) = direct.next()
                && direct.next().is_none()
            {
                let absent = if kind == DescriptorKind::Field {
                    signatures.fields.get(index).is_some_and(Option::is_none)
                } else {
                    signatures.methods.get(index).is_some_and(Option::is_none)
                };
                // This suppresses only an optional generic proof. It grants no member binding:
                // existing raw recovery keeps its own accessibility and resolver decisions.
                if absent {
                    continue;
                }
            }
        }
        budget.charge(Cost::AnalysisSteps, descriptor.0.len() as u64)?;
        let Ok(facts) = descriptor_facts(&descriptor.0, kind) else {
            return Ok(true);
        };
        if facts
            .components()
            .any(|component| component.is_array() || component.object_name().is_some())
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Propagate declared producers over the existing SSA copy/phi/array graph. A producer is added
/// only after every possible receiver is known and instantiates the same resolved declaration.
/// Unknown leaves and rootless cycles remain unknown; null never invents a type application.
pub fn declared_value_types<H: RecoveryHeaders>(
    ir: &jarde_jvm::method_ir::MethodIr,
    headers: &mut H,
    budget: &mut Budget,
) -> std::result::Result<DependencyResult<DeclaredGenericTypes>, H::Error> {
    use crate::facts::{FieldAccess, InvokeKind, Operation};
    use jarde_jvm::{
        method_ir::{Definition, RefType, Slot, Value},
        resolver::ReferenceUse,
    };
    let mut types = match declared_method_types(ir, headers, budget)? {
        DependencyResult::Found(types) => types,
        DependencyResult::Missing => return Ok(DependencyResult::Missing),
        DependencyResult::Ambiguous => return Ok(DependencyResult::Ambiguous),
        DependencyResult::Unknown => return Ok(DependencyResult::Unknown),
    };
    let (Some(ssa), Some(code)) = (ir.ssa(), ir.code()) else {
        return Ok(DependencyResult::Unknown);
    };
    let operations = crate::decode::Operations::of(code, ir.constant_pool());
    let has_receiver = types.receiver.is_some();
    let flows = crate::assignability::reference_bounds(ssa, &operations, has_receiver, budget)?;
    let loader = types
        .declaration
        .as_ref()
        .expect("the bound method has its selected declaration")
        .loader
        .clone();
    let mut seeds = BTreeMap::new();
    for (id, value) in ssa.values_with_ids() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if !flows.parents_of(id).is_empty() {
            continue;
        }
        let mut seed = None;
        match value.def() {
            Definition::Entry {
                slot: Slot::Local(slot),
                ..
            } => {
                seed = if *slot == 0 && has_receiver {
                    types.receiver
                } else {
                    types
                        .parameters
                        .iter()
                        .find_map(|(position, ty)| (*position == *slot).then_some(*ty))
                };
            }
            Definition::Instruction { bci, .. } => {
                let raw = match operations.get(*bci) {
                    Some(Operation::CheckCast { ty }) => Some(ty.0.clone()),
                    Some(Operation::Invoke(target)) if target.raw_name().0 == b"<init>" => {
                        match value.ty() {
                            Value::Ref(RefType::Named { name, .. }) => Some(name.clone()),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Some(raw) = raw {
                    let descriptor = if raw.first() == Some(&b'[') {
                        raw
                    } else {
                        let mut descriptor = vec![b'L'];
                        descriptor.extend_from_slice(&raw);
                        descriptor.push(b';');
                        descriptor
                    };
                    let syntax =
                        jarde_reader::signature::parse_field_signature(&descriptor, budget)?;
                    if let DependencyResult::Found(ty) =
                        bind_declared_type(&mut types, &syntax.ty, &loader, &[], headers, budget)?
                    {
                        seed = Some(ty);
                    }
                }
                if matches!(value.ty(), Value::Null) {
                    budget.charge(Cost::IrItems, 1)?;
                    seeds.insert(id, BTreeSet::new());
                }
            }
            _ => {}
        }
        if let Some(seed) = seed {
            budget.charge(Cost::IrItems, 2)?;
            seeds.insert(id, BTreeSet::from([seed]));
        }
    }
    let mut resolved = BTreeMap::<_, ResolvedMemberRef>::new();
    budget.charge(Cost::IrItems, seeds.len() as u64 * 2)?;
    let base_seeds = seeds.clone();
    let values = loop {
        let values = propagate_values(&types, ssa, &flows, &seeds, budget)?;
        budget.charge(Cost::IrItems, base_seeds.len() as u64 * 2)?;
        let mut next_seeds = base_seeds.clone();
        // A raw receiver can acquire proved arguments after an upstream physical cast or
        // member result is bound. Recompute consumers from that iteration's complete inputs;
        // retaining the first successful raw instantiation would freeze a stale result type.
        types.members.clear();
        types.unique_calls.clear();
        types.casts.clear();
        for block in ssa.blocks() {
            for instruction in block.instructions() {
                budget.poll()?;
                budget.charge(Cost::AnalysisSteps, 1)?;
                let key = (block.block().clone(), instruction.bci());
                let bci = instruction.bci();
                if matches!(operations.get(bci), Some(Operation::CheckCast { .. })) {
                    let output = instruction.writes().iter().find_map(|(slot, output)| {
                        matches!(slot, Slot::Stack(_)).then_some(*output)
                    });
                    let target =
                        output
                            .and_then(|output| base_seeds.get(&output))
                            .and_then(|nodes| {
                                if nodes.len() != 1 {
                                    return None;
                                }
                                match &types.types[*nodes.first()?] {
                                    Ty::Reference { declaration, .. } => Some(declaration.clone()),
                                    _ => None,
                                }
                            });
                    let input = instruction
                        .reads()
                        .iter()
                        .find_map(|(slot, input)| matches!(slot, Slot::Stack(_)).then_some(*input));
                    if let (Some(output), Some(target), Some(inputs)) =
                        (output, target, input.and_then(|input| values.get(&input)))
                    {
                        let mut application = None;
                        let mut complete = !inputs.is_empty();
                        for input in inputs {
                            let next = match declared_supertype(
                                &mut types, *input, &target, headers, budget,
                            )? {
                                DependencyResult::Found(next) => next,
                                _ => {
                                    complete = false;
                                    break;
                                }
                            };
                            if application.is_some_and(|previous| previous != next) {
                                complete = false;
                                break;
                            }
                            application = Some(next);
                        }
                        if complete
                            && let Some(application) = application
                            && matches!(&types.types[application], Ty::Reference { arguments, .. } if !arguments.is_empty())
                        {
                            budget.charge(Cost::IrItems, 3 + block.block().path().len() as u64)?;
                            next_seeds.insert(output, BTreeSet::from([application]));
                            types.casts.push((block.block().clone(), bci, application));
                        }
                    }
                    continue;
                }
                let (symbol, use_kind, is_static) = match operations.get(bci) {
                    Some(Operation::Field {
                        access,
                        is_static,
                        owner,
                        name,
                        descriptor,
                    }) => (
                        SymbolRef::Field {
                            owner: owner.clone(),
                            name: name.clone(),
                            descriptor: descriptor.clone(),
                        },
                        if *access == FieldAccess::Read {
                            ReferenceUse::FieldRead
                        } else {
                            ReferenceUse::FieldWrite
                        },
                        *is_static,
                    ),
                    Some(Operation::Invoke(target)) if target.raw_name().0 != b"<init>" => (
                        SymbolRef::Method {
                            owner: target.raw_owner().clone(),
                            name: target.raw_name().clone(),
                            descriptor: target.raw_descriptor().clone(),
                        },
                        match target.kind() {
                            InvokeKind::Static => ReferenceUse::InvokeStatic,
                            InvokeKind::Special => ReferenceUse::InvokeSpecial,
                            InvokeKind::Virtual => ReferenceUse::InvokeVirtual,
                            InvokeKind::Interface => ReferenceUse::InvokeInterface,
                        },
                        target.kind() == InvokeKind::Static,
                    ),
                    _ => continue,
                };
                let receiver = if is_static {
                    None
                } else {
                    let value = instruction
                        .reads()
                        .iter()
                        .filter_map(|(slot, value)| match slot {
                            Slot::Stack(depth) => Some((*depth, *value)),
                            _ => None,
                        })
                        .min_by_key(|(depth, _)| *depth)
                        .map(|(_, value)| value);
                    let Some(nodes) = value
                        .and_then(|value| values.get(&value))
                        .filter(|nodes| !nodes.is_empty())
                    else {
                        continue;
                    };
                    Some(nodes)
                };
                let declaration = if let Some(declaration) = resolved.get(&key) {
                    declaration.clone()
                } else {
                    let declaration = match headers.resolve_member(&symbol, use_kind, budget)? {
                        DependencyResult::Found(declaration) => declaration,
                        DependencyResult::Missing
                        | DependencyResult::Ambiguous
                        | DependencyResult::Unknown => continue,
                    };
                    budget.charge(Cost::IrItems, 1)?;
                    resolved.insert(key.clone(), declaration.clone());
                    declaration
                };
                let mut member = None;
                let mut complete = true;
                let static_application;
                let applications = if let Some(receiver) = receiver {
                    receiver
                } else {
                    // A static member uses its actual declaring definition, with no receiver
                    // substitutions. Its own method binders remain outside this finite domain.
                    let owner = match &declaration.member {
                        SymbolRef::Field { owner, .. } | SymbolRef::Method { owner, .. } => {
                            owner.clone()
                        }
                        _ => continue,
                    };
                    let application = intern(
                        &mut types,
                        Ty::Reference {
                            declaration: ResolvedMemberRef {
                                loader: declaration.loader.clone(),
                                definition: declaration.definition.clone(),
                                member: SymbolRef::Class { owner },
                            },
                            arguments: Vec::new(),
                        },
                        budget,
                    )?;
                    static_application = BTreeSet::from([application]);
                    &static_application
                };
                for receiver in applications {
                    let candidate = match declared_member_types(
                        &mut types,
                        *receiver,
                        &declaration,
                        bci,
                        headers,
                        budget,
                    )? {
                        DependencyResult::Found(candidate) => candidate,
                        DependencyResult::Missing
                        | DependencyResult::Ambiguous
                        | DependencyResult::Unknown => {
                            complete = false;
                            break;
                        }
                    };
                    if member
                        .as_ref()
                        .is_some_and(|previous| previous != &candidate)
                    {
                        complete = false;
                        break;
                    }
                    member = Some(candidate);
                }
                if !complete {
                    continue;
                }
                let Some(member) = member else {
                    continue;
                };
                if matches!(&member, DeclaredGenericMemberTypes::Method { .. })
                    && matches!(
                        declared_call_binding(&mut types, &member, headers, budget)?,
                        DependencyResult::Found(true)
                    )
                {
                    budget.charge(Cost::IrItems, 1 + block.block().path().len() as u64)?;
                    types.unique_calls.push((block.block().clone(), bci));
                }
                let result = match &member {
                    DeclaredGenericMemberTypes::Field { ty, .. }
                        if use_kind == ReferenceUse::FieldRead =>
                    {
                        Some(*ty)
                    }
                    DeclaredGenericMemberTypes::Method { result, .. } => *result,
                    _ => None,
                };
                if let Some(result) = result {
                    for (slot, output) in instruction.writes() {
                        if matches!(slot, Slot::Stack(_))
                            && matches!(ssa.value(*output).ty(), Value::Ref(_))
                        {
                            budget.charge(Cost::IrItems, 2)?;
                            next_seeds.insert(*output, BTreeSet::from([result]));
                        }
                    }
                }
                budget.charge(Cost::IrItems, 2)?;
                budget.charge(Cost::IrItems, block.block().path().len() as u64)?;
                types.members.push((block.block().clone(), member));
            }
        }
        if next_seeds == seeds {
            break values;
        }
        seeds = next_seeds;
    };
    budget.charge(Cost::IrItems, ssa.values().len() as u64)?;
    for (id, _) in ssa.values_with_ids() {
        let value = values
            .get(&id)
            .map(|types| types.iter().copied().collect::<Vec<_>>());
        budget.charge(
            Cost::IrItems,
            value.as_ref().map_or(0, |types| types.len()) as u64,
        )?;
        types.values.push(value);
    }
    let mut runtime_seeds = BTreeMap::new();
    for (id, value) in ssa.values_with_ids() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if !flows.parents_of(id).is_empty() {
            continue;
        }
        if matches!(value.ty(), Value::Null) {
            budget.charge(Cost::IrItems, 1)?;
            runtime_seeds.insert(id, BTreeSet::new());
        } else if let Some((raw, loader, class_operand)) =
            crate::assignability::named_reference_type(value, &operations, has_receiver)
            && let DependencyResult::Found(node) =
                runtime_type(&mut types, raw, loader, class_operand, headers, budget)?
        {
            budget.charge(Cost::IrItems, 2)?;
            runtime_seeds.insert(id, BTreeSet::from([node]));
        }
    }
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            for (owner, member) in &types.members {
                budget.poll()?;
                budget.charge(Cost::AnalysisSteps, 1)?;
                if owner != block.block() {
                    continue;
                }
                let (at, result) = match member {
                    DeclaredGenericMemberTypes::Field { bci, erased_ty, .. } => {
                        (*bci, Some(*erased_ty))
                    }
                    DeclaredGenericMemberTypes::Method {
                        bci, erased_result, ..
                    } => (*bci, *erased_result),
                };
                if at != instruction.bci() {
                    continue;
                }
                if let Some(result) = result {
                    for (slot, output) in instruction.writes() {
                        if matches!(slot, Slot::Stack(_))
                            && matches!(ssa.value(*output).ty(), Value::Ref(_))
                        {
                            budget.charge(Cost::IrItems, 2)?;
                            runtime_seeds.insert(*output, BTreeSet::from([result]));
                        }
                    }
                }
                break;
            }
        }
    }
    let runtime_values = propagate_values(&types, ssa, &flows, &runtime_seeds, budget)?;
    budget.charge(Cost::IrItems, ssa.values().len() as u64)?;
    for (id, _) in ssa.values_with_ids() {
        let value = runtime_values
            .get(&id)
            .map(|nodes| nodes.iter().copied().collect::<Vec<_>>());
        budget.charge(Cost::IrItems, value.as_ref().map_or(0, Vec::len) as u64)?;
        types.runtime_values.push(value);
    }
    let mut wanted = BTreeSet::new();
    for (_, member) in &types.members {
        match member {
            DeclaredGenericMemberTypes::Field {
                application,
                ty,
                erased_ty,
                ..
            } => {
                wanted.insert(*application);
                wanted.insert(*ty);
                wanted.insert(*erased_ty);
            }
            DeclaredGenericMemberTypes::Method {
                application,
                parameters,
                erased_parameters,
                ..
            } => {
                wanted.insert(*application);
                wanted.extend(parameters.iter().copied());
                wanted.extend(erased_parameters.iter().copied());
            }
        }
    }
    wanted.extend(types.result);
    wanted.extend(types.erased_result);
    wanted.extend(types.parameters.iter().map(|(_, node)| *node));
    budget.charge(Cost::IrItems, wanted.len() as u64)?;
    let mut array_components: Vec<_> = wanted.iter().copied().collect();
    while let Some(node) = array_components.pop() {
        budget.poll()?;
        budget.charge(Cost::AnalysisSteps, 1)?;
        if let Ty::Array(component) = types.types[node]
            && wanted.insert(component)
        {
            budget.charge(Cost::IrItems, 2)?;
            array_components.push(component);
        }
    }
    budget.charge(Cost::IrItems, wanted.len() as u64)?;
    let mut erasures = Vec::new();
    for target in wanted {
        if let DependencyResult::Found(erasure) =
            runtime_erasure(&mut types, target, headers, budget, 0)?
        {
            budget.charge(Cost::IrItems, 1)?;
            erasures.push((target, erasure));
        }
    }
    let mut sources = BTreeSet::new();
    for nodes in runtime_values.values() {
        sources.extend(nodes.iter().copied());
    }
    budget.charge(Cost::IrItems, sources.len() as u64)?;
    for source in sources {
        for (target, erasure) in &erasures {
            if runtime_widens(&types, source, *erasure, headers, budget, 0)? {
                budget.charge(Cost::IrItems, 1)?;
                budget.charge(Cost::IrEdges, 1)?;
                types.runtime_upcasts.push((source, *target));
            }
        }
    }
    prove_declared_relations(&mut types, headers, budget)?;
    Ok(DependencyResult::Found(types))
}
