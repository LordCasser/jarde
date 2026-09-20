//! Lists what an artifact holds and hands back the identity a later read is addressed by.
//!
//! The example walks the change's whole chain with the public library surface and nothing else:
//!
//! 1. **list classes** — the level-one listing partitions the physical scope by names alone, reads
//!    **no** header, and says so: every entry appears exactly once, as a class candidate or as the
//!    ordinary resource the class-name rule does not select;
//! 2. **confirm classes** — the level-two listing really reads each candidate's bytes and publishes
//!    the identity those bytes are (location, class bytes digest and length, syntactic variant)
//!    beside the facts the class declares (`this_class`, class flags, super/interfaces);
//! 3. **list members** — one bounded read of one *definition*, addressed by the identity the listing
//!    handed back, states the class declaration, its fields and its methods. A method item carries
//!    its raw name, descriptor, access flags and a `PhysicalMethodId` a method request consumes
//!    directly, so nothing about the member is reassembled by the caller;
//! 4. **find by name** — the friendly lookup answers a dotted or internal class name (and an optional
//!    member filter) with every physical candidate it bound, never a silent first.
//!
//! Two things the run prints on purpose: the **evidence level** of each answer (how many candidate
//! entries the confirmed listing did *not* read) and the **usage** each request charged —
//! `class_headers` for the reads, and zero `method_bodies` and zero IR construction for every
//! listing, because a listing reads no body and starts no analysis.
//!
//! It reads a standalone `CLASS` file or a `ZIP`/`JAR`/`WAR`. Nothing is executed, loaded, resolved,
//! parsed into Java or recovered, no listing claims that a listed class is loadable, linkable or
//! verified, and no class-path root or container layout is inferred: reading one of these definitions
//! is a later request under an environment the caller declares.
//!
//! Usage: `cargo run --example navigate_artifact [path]`; without an argument the historical v52
//! fixture is used.

use jarde::{
    ArtifactInput, Budget, ClassListingItem, CountedBudgetDimension, Engine, ExecutionReport,
    JvmBytes, Limits, MemberBodyEvidence, MemberQuery, MemberQueryKind, NavigationQuery,
    PhysicalScope, UsageSnapshot,
};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn limits() -> Limits {
    Limits {
        input_bytes: 64 * 1024 * 1024,
        archive_entries: 100_000,
        entry_bytes: 64 * 1024 * 1024,
        read_bytes: 64 * 1024 * 1024,
        class_bytes: 64 * 1024 * 1024,
        attribute_bytes: 16 * 1024 * 1024,
        code_bytes: 4 * 1024 * 1024,
        result_items: 1_000_000,
        output_bytes: 64 * 1024 * 1024,
        class_headers: 10_000,
        method_bodies: 0,
        nested_depth: 8,
        elapsed_millis: 60_000,
        ..Limits::default()
    }
}

/// The counted dimensions of one request, so a listing's own accounting is visible.
fn usage_line(label: &str, usage: &UsageSnapshot) {
    let counted = CountedBudgetDimension::ALL
        .iter()
        .map(|dimension| format!("{dimension:?}={}", usage.counted_usage(*dimension)))
        .filter(|entry| !entry.ends_with("=0"))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{label}.usage [{counted}]");
}

fn name(value: &JvmBytes) -> String {
    String::from_utf8_lossy(&value.0).into_owned()
}

fn status(execution: &ExecutionReport) -> String {
    match execution {
        ExecutionReport::Complete { .. } => "complete".to_owned(),
        ExecutionReport::Partial { reason, .. } => format!("partial({reason:?})"),
        ExecutionReport::Cancelled { .. } => "cancelled".to_owned(),
        ExecutionReport::Failed { reason, .. } => format!("failed({reason:?})"),
    }
}

fn run(path: PathBuf) -> jarde::Result<()> {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine.open(ArtifactInput::Path(path), &mut budget)?;
    println!(
        "artifact: snapshot={} kind={:?} bytes={}",
        snapshot.id().0,
        snapshot.kind(),
        snapshot.len()
    );
    // A snapshot is a shared, immutable handle: every request below reads it under its own budget,
    // and none of them changes the artifact or carries state into the next one.
    let scope = PhysicalScope::SnapshotAll;

    // 1. Names only. Every entry of the scope appears exactly once, and no class byte is read.
    let mut budget = Budget::new(limits());
    let candidates = engine.list_class_candidates(&snapshot, &scope, &mut budget)?;
    let candidate_count = candidates.candidates().count();
    let resource_count = candidates.resources().count();
    println!(
        "list_class_candidates: candidates={candidate_count} resources={resource_count} execution={}",
        status(&candidates.execution)
    );
    assert_eq!(
        budget.usage().class_headers,
        0,
        "a candidate listing reads no class header at all"
    );
    for item in &candidates.items {
        match item {
            ClassListingItem::ClassCandidate { location } => match location.entry() {
                Some(entry) => println!(
                    "  candidate {} ordinal={} container={}",
                    String::from_utf8_lossy(&entry.raw_name.0),
                    entry.ordinal,
                    entry.origin.current_container().0
                ),
                None => println!("  candidate <the snapshot's own standalone class>"),
            },
            ClassListingItem::Resource { entry } => println!(
                "  resource  {} ordinal={}",
                String::from_utf8_lossy(&entry.raw_name.0),
                entry.ordinal
            ),
        }
    }
    usage_line("list_class_candidates", &budget.usage());
    if candidate_count == 0 {
        println!("nothing looks like a class here; a listing stops where its evidence stops");
        return Ok(());
    }

    // 2. The confirmed read: one header read per candidate, each carrying its own identity.
    let mut budget = Budget::new(limits());
    let confirmed = engine.list_class_declarations(&snapshot, &scope, &mut budget)?;
    println!(
        "list_class_declarations: candidates={} confirmed={} unconfirmed={} execution={}",
        confirmed.candidates,
        confirmed.items.len(),
        confirmed.unconfirmed.len(),
        status(&confirmed.execution)
    );
    for item in &confirmed.items {
        println!(
            "  class {} flags={:#06x} binding={:?} bytes={} digest={}",
            item.declaration.this_class.escaped(),
            item.declaration.access_flags,
            item.binding,
            item.definition.class_bytes.length,
            item.definition.class_bytes.digest.0
        );
        match item.definition.entry() {
            Some(entry) => println!(
                "    definition: container={} ordinal={} raw_name={} variant={:?}",
                entry.origin.current_container().0,
                entry.ordinal,
                String::from_utf8_lossy(&entry.raw_name.0),
                item.definition.variant
            ),
            None => println!("    definition: the snapshot's own standalone class"),
        }
        if let Some(stop) = &item.member_table {
            println!(
                "    member table stopped at {:?}[{}] ({}): the class is confirmed, its members are a prefix",
                stop.phase, stop.index, stop.code
            );
        }
    }
    for diagnostic in &confirmed.diagnostics {
        println!("  diagnostic {}: {}", diagnostic.code, diagnostic.message);
    }
    usage_line("list_class_declarations", &budget.usage());
    assert!(
        budget.usage().method_bodies == 0,
        "confirming classes reads no method body"
    );
    let Some(first) = confirmed.items.first() else {
        println!("no class was confirmed, so no identity is handed on");
        return Ok(());
    };
    let definition = first.definition.clone();

    // 3. Members of exactly that definition, from the identity alone.
    let mut budget = Budget::new(limits());
    let members = engine.list_members(&snapshot, &definition, &mut budget)?;
    assert_eq!(
        members.definition, definition,
        "the member listing read the very definition it was handed"
    );
    match members.declaration() {
        Some(class) => println!(
            "list_members: class={} flags={:#06x} super={:?} interfaces={:?} execution={}",
            class.declaration.this_class.escaped(),
            class.declaration.access_flags,
            class
                .declaration
                .super_class
                .as_ref()
                .map(|name| name.escaped()),
            class
                .declaration
                .interfaces
                .iter()
                .map(|name| name.escaped())
                .collect::<Vec<_>>(),
            status(&members.execution)
        ),
        None => println!("list_members: the class-level item itself was not published"),
    }
    for field in members.fields() {
        println!(
            "  field  [{}] {}:{} flags={:#06x}",
            field.index,
            field.name.escaped(),
            field.descriptor.escaped(),
            field.access_flags
        );
    }
    for method in members.methods() {
        let body = match &method.body {
            MemberBodyEvidence::CodeAttribute { content_span } => format!(
                "Code@{}..{} (not read)",
                content_span.start,
                content_span.start + content_span.length
            ),
            MemberBodyEvidence::NoCodeAttribute => "no Code attribute".to_owned(),
        };
        println!(
            "  method [{}] {}:{} flags={:#06x} {body}",
            method.index,
            method.name.escaped(),
            method.descriptor.escaped(),
            method.access_flags
        );
    }
    if let Some(stop) = &members.stopped_at {
        println!(
            "  the member table stopped at {:?}[{}]: the members above are the reliable prefix",
            stop.phase, stop.index
        );
    }
    usage_line("list_members", &budget.usage());
    let usage = budget.usage();
    assert!(
        usage.method_bodies == 0 && usage.code_bytes == 0,
        "listing members reads no body"
    );
    assert!(
        usage.ir_items == 0 && usage.ir_edges == 0 && usage.analysis_steps == 0,
        "listing members builds no CFG, no SSA and no IR"
    );

    // The handoff itself: this is the identity a method request consumes, with nothing retyped.
    let Some(method) = members.methods().next() else {
        println!("this class declares no method, so there is no method identity to hand on");
        return Ok(());
    };
    println!(
        "handoff: PhysicalMethodId {{ owner: {}#{} , name: {} , descriptor: {} }}",
        method.identity.owner.location.snapshot().0,
        method.identity.owner.class_bytes.digest.0,
        name(&method.identity.name),
        name(&method.identity.descriptor)
    );

    // 4. The friendly lookup: both spellings of one class name bind the same definition, and a
    //    member filter answers with every overload rather than electing one.
    let internal = members
        .declaration()
        .expect("the class-level item was published")
        .declaration
        .this_class
        .raw()
        .0
        .clone();
    let dotted = internal
        .iter()
        .map(|byte| {
            if *byte == b'/' {
                '.'
            } else {
                char::from(*byte)
            }
        })
        .collect::<String>();
    let ask = |class: jarde::ClassNameQuery, member: Option<MemberQuery>| {
        let mut budget = Budget::new(limits());
        let report = engine.find_targets(
            &snapshot,
            &scope,
            &NavigationQuery { class, member },
            &mut budget,
        )?;
        Ok::<_, jarde::Error>((report, budget.usage()))
    };
    let (by_dotted, _) = ask(jarde::ClassNameQuery::dotted(dotted.clone()), None)?;
    let (by_internal, _) = ask(
        jarde::ClassNameQuery::internal(String::from_utf8_lossy(&internal).into_owned()),
        None,
    )?;
    println!(
        "find_targets: `{dotted}` -> {} class candidate(s), `{}` -> {} class candidate(s)",
        by_dotted.classes().count(),
        String::from_utf8_lossy(&internal),
        by_internal.classes().count()
    );
    let dotted_definition = by_dotted
        .classes()
        .next()
        .map(|class| class.definition.clone());
    let internal_definition = by_internal
        .classes()
        .next()
        .map(|class| class.definition.clone());
    assert_eq!(
        dotted_definition, internal_definition,
        "the two spellings denote one physical definition, because the spelling is not the identity"
    );
    assert_eq!(
        dotted_definition.as_ref(),
        Some(&definition),
        "and that definition is the one the confirmed listing handed over"
    );
    let (overloads, _) = ask(
        jarde::ClassNameQuery::internal(String::from_utf8_lossy(&internal).into_owned()),
        Some(MemberQuery {
            name: method.identity.name.clone(),
            descriptor: None,
            kind: MemberQueryKind::Methods,
        }),
    )?;
    println!(
        "find_targets(member): `{}` -> {} method candidate(s) — every overload, never a silent first",
        name(&method.identity.name),
        overloads.methods().count()
    );
    for candidate in overloads.methods() {
        println!(
            "  member candidate [{}]: {}:{} flags={:#06x}",
            candidate.index,
            candidate.name.escaped(),
            candidate.descriptor.escaped(),
            candidate.access_flags
        );
    }
    println!(
        "note: every listing above read headers only — no body was decoded, no analysis was started, \
         and no candidate is claimed to be loadable, linkable or verified"
    );
    Ok(())
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let path = match args.next() {
        Some(path) => PathBuf::from(path),
        None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
    };
    if args.next().is_some() {
        eprintln!("usage: {} [path]", PathBuf::from(program).display());
        return ExitCode::from(2);
    }
    match run(path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("navigate_artifact: {error}");
            ExitCode::FAILURE
        }
    }
}
