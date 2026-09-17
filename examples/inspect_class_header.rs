use jarde::{
    ArtifactInput, Budget, ClassTarget, Engine, InspectionMode, Limits, VerificationStatus,
};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn limits() -> Limits {
    Limits {
        input_bytes: 16 * 1024 * 1024,
        archive_entries: 1_000,
        entry_bytes: 16 * 1024 * 1024,
        read_bytes: 32 * 1024 * 1024,
        class_bytes: 16 * 1024 * 1024,
        attribute_bytes: 8 * 1024 * 1024,
        code_bytes: 4 * 1024 * 1024,
        result_items: 100_000,
        output_bytes: 32 * 1024 * 1024,
        nested_depth: 8,
        elapsed_millis: 30_000,
        ..Limits::default()
    }
}

fn run(path: PathBuf) -> jarde::Result<()> {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine.open(ArtifactInput::Path(path), &mut budget)?;
    let report = engine.inspect_header(
        &snapshot,
        ClassTarget::Root,
        &mut budget,
        InspectionMode::Strict,
    )?;
    let header = &report.inspection.header;

    println!("class={}", header.this_class.escaped());
    println!("version={}.{}", header.major_version, header.minor_version);
    println!("fields={}", header.fields.len());
    println!("methods={}", header.methods.len());
    println!("verification={:?}", report.inspection.verification);
    println!("usage={:?}", budget.usage());

    debug_assert_eq!(
        report.inspection.verification,
        VerificationStatus::NotPerformed
    );
    Ok(())
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
    if args.next().is_some() {
        eprintln!(
            "error: expected exactly one standalone CLASS path; ZIP/JAR/WAR input is not supported by this example"
        );
        return ExitCode::from(2);
    }

    match run(PathBuf::from(path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("inspect_class_header: {error}");
            ExitCode::FAILURE
        }
    }
}
