//! Transcribes one standalone `.class` file with the class-source presentation — the patrol
//! entry that records what the assembly layer actually writes for a frozen fixture.
//!
//! Usage: `cargo run --release --example class_source_file -- path/to/Fixture.class`

use jarde::*;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn budget() -> Budget {
    jarde::task_budget(&[]).expect("the task defaults are bounded")
}

fn run(path: PathBuf) -> jarde::Result<()> {
    // A standalone class file's own name is its internal class name; a nested class keeps the
    // `$` spelling its class file also states verbatim.
    let internal = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_string();
    let engine = Engine::new();
    let mut open_budget = budget();
    let snapshot = engine.open(ArtifactInput::Path(path), &mut open_budget)?;
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(internal),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_string()),
        },
    };
    let report = engine.class_source(std::slice::from_ref(&snapshot), &request, &mut budget())?;
    match report {
        OperationOutcome::Performed(report) => {
            print!("{}", report.text);
            for diagnostic in &report.diagnostics {
                eprintln!("diagnostic: {} {}", diagnostic.code, diagnostic.message);
            }
            Ok(())
        }
        outcome => {
            eprintln!("outcome: {outcome:?}");
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let Some(path) = args.next() else {
        eprintln!(
            "usage: {} <standalone.class>",
            PathBuf::from(program).display()
        );
        return ExitCode::from(2);
    };
    match run(PathBuf::from(path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:?}");
            ExitCode::FAILURE
        }
    }
}
