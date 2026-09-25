use jarde::*;
use std::{
    fs,
    ops::Deref,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Scratch(std::path::PathBuf);

impl Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_dir(label: &str) -> Scratch {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-static-local-generic-throws-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    Scratch(dir)
}

fn report_from_source(label: &str, debug: &str, source: &str) -> ClassSourceReport {
    let dir = temp_dir(label);
    let path = dir.join("StaticThrows.java");
    fs::write(&path, source).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", debug, "-d"])
        .arg(&*dir)
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let bytes = fs::read(dir.join("StaticThrows.class")).unwrap();
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("StaticThrows"),
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
            loader: LoaderId("app".to_owned()),
        },
    };
    match engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    }
}

#[test]
fn direct_static_return_with_local_throws_variable_is_projected_atomically() {
    const VALID: &str = r#"
public class StaticThrows {
    public static <T, X extends Exception> T echo(T value) throws X {
        return value;
    }
}
"#;
    for (label, debug) in [("debug", "-g"), ("no-debug", "-g:none")] {
        let report = report_from_source(label, debug, VALID);
        assert!(report.text.contains(
            "public static <T extends java.lang.Object, X extends java.lang.Exception> T echo(T "
        ));
        assert!(report.text.contains(") throws X {"), "{}", report.text);
        assert!(
            !report
                .text
                .contains("generic Signature projection refused for `echo"),
            "{}",
            report.text
        );
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"echo")
            .expect("physical method is retained");
        assert!(method.markers.iter().any(|marker| {
            marker.contains("direct parameter return") && marker.contains("Signature scope/erasure")
        }));
    }
}

#[test]
fn direct_return_type_can_also_be_the_local_throws_variable() {
    const VALID: &str = r#"
public class StaticThrows {
    public static <T extends Exception> T pass(T value) throws T {
        return value;
    }
}
"#;
    let report = report_from_source("same-variable", "-g:none", VALID);
    assert!(
        report
            .text
            .contains("public static <T extends java.lang.Exception> T pass(T arg0) throws T"),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("generic Signature projection refused for `pass"),
        "{}",
        report.text
    );
}

#[test]
fn a_non_jdk_throwable_bound_keeps_the_physical_generic_declaration() {
    const INVALID: &str = r#"
public class StaticThrows {
    public static <T, X extends java.io.IOException> T echo(T value) throws X {
        return value;
    }
}
"#;
    let report = report_from_source("custom-bound", "-g:none", INVALID);
    assert!(
        report
            .text
            .contains("generic Signature projection refused for `echo"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("throws variable bound is not a proved JDK throwable root"),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("<T extends java.lang.Object, X extends java.io.IOException>"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("java.lang.Object echo(java.lang.Object arg0) throws java.io.IOException")
    );
}
