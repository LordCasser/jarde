//! The supertype-return form of the anonymous superclass projection
//! (`recover-anonymous-supertype-return`): the allocation sits at the root method's direct
//! return and the root method's declared return type is a proved one-layer supertype of the
//! direct superclass — the superclass itself, its direct superclass, or one of its directly
//! implemented interfaces — so the projection spells the declaration at the supertype and the
//! allocation at the superclass: `Renderer create() { return new Base(...) { ... }; }`.
//!
//! The frozen anchor `anonymous-top-level` closes the 5.3 anonymous-inline chain's last open
//! fixture: `create` declares `Renderer` while the child's direct superclass is `Base`, so the
//! exact-`Lparent;` gate refused it before this slice. The contrast positive
//! (`supertype-return`, formerly ring 3's boundary probe) composes the widened return segment
//! with ring 3's capture parameter — `(P)LT;` against `()LT;` — proving the return segment and
//! the parameter table compose independently. The refusal fixtures pin the one-layer boundary
//! (`indirect-supertype-return`), the spellability boundary (`nested-supertype-return`), and
//! the census containment (`interface-self-invocation`: the interface path's owner census must
//! keep refusing child-body self-invocations even though the direct path now admits them).
//! Two shapes are not expressible in javac output — an unrelated return type and a non-reference
//! (array) return — because a compiled declaration is always assignable from its allocation;
//! they are asserted by same-length descriptor patches of the anchor root class, the honest
//! non-javac inputs the gate must refuse.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

// The frozen anchor: one source file declares the interface, the superclass and the root class,
// so the recompile legs derive the support source from it rather than restating it.
const ANCHOR_SOURCE: &str =
    include_str!("fixtures/proved-java-structure/anonymous-top-level/AnonymousTopLevel.java");

const ANCHOR_ROOT: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-top-level/AnonymousTopLevel.class");
const ANCHOR_CHILD: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-top-level/AnonymousTopLevel$1.class");
const ANCHOR_BASE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-top-level/Base.class");
const ANCHOR_RENDERER: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-top-level/Renderer.class");

// The contrast positive: ring 3's frozen `(P)LT;` boundary probe — one capture parameter plus a
// one-layer supertype return, the fourth descriptor combination this slice's gate admits.
const CONTRAST_SOURCE: &str = include_str!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/ParameterizedSupertypeReturn.java"
);
const CONTRAST_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/ParameterizedSupertypeReturn.class"
);
const CONTRAST_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/ParameterizedSupertypeReturn$1.class"
);
const CONTRAST_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/Base.class"
);
const CONTRAST_RENDERER: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/Renderer.class"
);

// The one-layer boundary: `Top` is reachable from `Base` only through `Mid extends Top`.
const INDIRECT_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/indirect-supertype-return/SupertypeReturnIndirect.class"
);
const INDIRECT_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/indirect-supertype-return/SupertypeReturnIndirect$1.class"
);
const INDIRECT_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/indirect-supertype-return/Base.class"
);
const INDIRECT_MID: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/indirect-supertype-return/Mid.class"
);
const INDIRECT_TOP: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/indirect-supertype-return/Top.class"
);

// The spellability boundary: `Holder$Marker` sits in `Base`'s own interfaces (one layer) but a
// `$` name is not directly spellable source text — the pre-existing refusal keeps firing.
const NESTED_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/nested-supertype-return/SupertypeReturnNested.class"
);
const NESTED_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/nested-supertype-return/SupertypeReturnNested$1.class"
);
const NESTED_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/nested-supertype-return/Base.class"
);
const NESTED_HOLDER: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/nested-supertype-return/Holder.class"
);
const NESTED_MARKER: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/nested-supertype-return/Holder$Marker.class"
);

// The census containment: an interface-path anonymous body calling its own method must keep the
// shared owner census's refusal even though the direct path now admits the same shape.
const PROBE_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/interface-self-invocation/InterfaceSelfInvocation.class"
);
const PROBE_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/interface-self-invocation/InterfaceSelfInvocation$1.class"
);
const PROBE_RENDERER: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-supertype-return-refusals/interface-self-invocation/Renderer.class"
);

/// The frozen source's support part: everything before the root class declaration (package
/// private interface and superclass declarations only, so any file name carries them).
fn support_source(source: &'static str) -> &'static str {
    let index = source
        .find("public final class")
        .expect("the frozen source declares the root class");
    &source[..index]
}

#[test]
fn the_supertype_return_anchor_projects_the_interface_declaration() {
    let snapshot = open(zip_of(&[
        (b"AnonymousTopLevel.class", ANCHOR_ROOT),
        (b"AnonymousTopLevel$1.class", ANCHOR_CHILD),
        (b"Base.class", ANCHOR_BASE),
        (b"Renderer.class", ANCHOR_RENDERER),
    ]));
    let root = class_source_of(&snapshot, "AnonymousTopLevel");
    // The declaration spells the interface (the root method's own declared type); the
    // allocation spells the direct superclass the child extends — never the anonymous name.
    assert!(
        root.text
            .contains("static Renderer create() {\n        java.lang.String captured = captureLocal();\n        return new Base(choose()) {"),
        "{}",
        root.text
    );
    assert!(!root.text.contains("AnonymousTopLevel$1"), "{}", root.text);
    assert!(
        !matches!(
            root.anonymous_interface_projection,
            class_source::ClassSourceAnonymousInterfaceProjection::Refused { .. }
        ),
        "{:?}",
        root.anonymous_interface_projection
    );
}

#[test]
fn the_supertype_return_anchor_recompiles_and_runs_as_the_original() {
    let entries: [(&[u8], &[u8]); 4] = [
        (b"AnonymousTopLevel.class", ANCHOR_ROOT),
        (b"AnonymousTopLevel$1.class", ANCHOR_CHILD),
        (b"Base.class", ANCHOR_BASE),
        (b"Renderer.class", ANCHOR_RENDERER),
    ];
    let snapshot = open(zip_of(&entries));
    let root = class_source_of(&snapshot, "AnonymousTopLevel");
    let scratch = Scratch::new("supertype-return-anchor");
    write_originals(&scratch, &entries);
    let original = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.path())
        .arg("AnonymousTopLevel")
        .output()
        .expect("JDK java is available for the original fixture run");
    let original_out = String::from_utf8_lossy(&original.stdout).into_owned();
    assert!(
        original.status.success()
            && original_out.contains("value=")
            && original_out.contains("events="),
        "the original fixture run is the behavior baseline:\n{original_out}{}",
        String::from_utf8_lossy(&original.stderr)
    );
    // The rendered source set: the projected root beside the frozen support source — the anchor
    // file declares `Renderer` and `Base` in the same unit, and the render claims only the root
    // class, so all three compiled units must come from the render plus that support part.
    let rendered = scratch.path().join("AnonymousTopLevel.java");
    fs::write(&rendered, projected_source(&root.text)).expect("write the recovered source");
    fs::write(
        scratch.path().join("Base.java"),
        support_source(ANCHOR_SOURCE),
    )
    .expect("write the support source");
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("-d")
        .arg(scratch.path().join("out"))
        .arg(&rendered)
        .arg(scratch.path().join("Base.java"))
        .output()
        .expect("JDK javac is available for the recompile check");
    assert!(
        compile.status.success(),
        "the rendered source set must compile (the baseline refused to):\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.path().join("out"))
        .arg("AnonymousTopLevel")
        .output()
        .expect("JDK java is available for the recompiled run");
    assert!(
        run.status.success(),
        "the recompiled program did not verify:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        original_out,
        "a text that compiles must run as the original ran — never a silent change"
    );
}

#[test]
fn the_parameterized_supertype_return_contrast_positive_projects_and_runs() {
    // `(P)LT;`: ring 3's capture parameter composed with this slice's widened return segment.
    // The parameter table and the return segment were proved independently, so this
    // former refusal fixture must now project exactly like the `()LT;` anchor.
    let entries: [(&[u8], &[u8]); 4] = [
        (b"ParameterizedSupertypeReturn.class", CONTRAST_ROOT),
        (b"ParameterizedSupertypeReturn$1.class", CONTRAST_CHILD),
        (b"Base.class", CONTRAST_BASE),
        (b"Renderer.class", CONTRAST_RENDERER),
    ];
    let snapshot = open(zip_of(&entries));
    let root = class_source_of(&snapshot, "ParameterizedSupertypeReturn");
    assert!(
        root.text
            .contains("private static Renderer create(java.lang.String"),
        "{}",
        root.text
    );
    assert!(root.text.contains("return new Base() {"), "{}", root.text);
    assert!(
        !root.text.contains("ParameterizedSupertypeReturn$1"),
        "{}",
        root.text
    );
    let scratch = Scratch::new("supertype-return-contrast");
    write_originals(&scratch, &entries);
    let original = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.path())
        .arg("ParameterizedSupertypeReturn")
        .output()
        .expect("JDK java is available for the original fixture run");
    let original_out = String::from_utf8_lossy(&original.stdout).into_owned();
    assert!(
        original.status.success() && original_out.contains("r:captured-value"),
        "the original fixture run is the behavior baseline:\n{original_out}{}",
        String::from_utf8_lossy(&original.stderr)
    );
    let rendered = scratch.path().join("ParameterizedSupertypeReturn.java");
    fs::write(&rendered, projected_source(&root.text)).expect("write the recovered source");
    fs::write(
        scratch.path().join("Base.java"),
        support_source(CONTRAST_SOURCE),
    )
    .expect("write the support source");
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("-d")
        .arg(scratch.path().join("out"))
        .arg(&rendered)
        .arg(scratch.path().join("Base.java"))
        .output()
        .expect("JDK javac is available for the recompile check");
    assert!(
        compile.status.success(),
        "the contrast positive's rendered source set must compile:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.path().join("out"))
        .arg("ParameterizedSupertypeReturn")
        .output()
        .expect("JDK java is available for the recompiled run");
    assert!(
        run.status.success(),
        "the recompiled program did not verify:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        original_out,
        "a text that compiles must run as the original ran — never a silent change"
    );
}

#[test]
fn the_gate_keeps_the_one_layer_and_spelling_boundaries() {
    // Two layers (`Base -> Mid -> Top`) need two hierarchy walks: refused. A `$` name in the
    // proved set (`Holder$Marker` is `Base`'s own interface) is not spellable source text:
    // the pre-existing refusal keeps firing. Both keep the physical class text.
    let indirect = class_source_of(
        &open(zip_of(&[
            (b"SupertypeReturnIndirect.class", INDIRECT_ROOT),
            (b"SupertypeReturnIndirect$1.class", INDIRECT_CHILD),
            (b"Base.class", INDIRECT_BASE),
            (b"Mid.class", INDIRECT_MID),
            (b"Top.class", INDIRECT_TOP),
        ])),
        "SupertypeReturnIndirect",
    );
    assert!(
        indirect.text.contains("new SupertypeReturnIndirect$1("),
        "{}",
        indirect.text
    );
    assert!(
        indirect
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "anonymous_super_return_type_unproved"),
        "{:?}",
        indirect.diagnostics
    );
    let nested = class_source_of(
        &open(zip_of(&[
            (b"SupertypeReturnNested.class", NESTED_ROOT),
            (b"SupertypeReturnNested$1.class", NESTED_CHILD),
            (b"Base.class", NESTED_BASE),
            (b"Holder.class", NESTED_HOLDER),
            (b"Holder$Marker.class", NESTED_MARKER),
        ])),
        "SupertypeReturnNested",
    );
    assert!(
        nested.text.contains("new SupertypeReturnNested$1("),
        "{}",
        nested.text
    );
    assert!(
        nested
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "anonymous_super_source_type_unproved"),
        "{:?}",
        nested.diagnostics
    );
}

#[test]
fn the_interface_path_keeps_refusing_child_self_invocations() {
    // The census widening is gated on the census path discriminant; the interface path passes
    // the restricted value, so its own child-body self-invocation keeps the exact refusal and
    // the physical text (the interface path's acceptance set cannot open as a side effect).
    let probe = class_source_of(
        &open(zip_of(&[
            (b"InterfaceSelfInvocation.class", PROBE_ROOT),
            (b"InterfaceSelfInvocation$1.class", PROBE_CHILD),
            (b"Renderer.class", PROBE_RENDERER),
        ])),
        "InterfaceSelfInvocation",
    );
    assert!(
        probe.text.contains("new InterfaceSelfInvocation$1("),
        "{}",
        probe.text
    );
    assert!(
        probe
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "anonymous_interface_child_additional_use"),
        "{:?}",
        probe.diagnostics
    );
}

#[test]
fn non_javac_return_descriptors_keep_their_refusals() {
    // A compiled declaration is always assignable from its allocation, so an unrelated return
    // type and a non-reference (array) return cannot come from javac. The honest inputs are
    // same-length descriptor patches of the anchor root class (the constant pool stays valid;
    // javap reads both patched descriptors). The gate refuses both loudly.
    let target: &[u8] = b"()LRenderer;";
    let cases: [(&[u8], &str); 2] = [
        (b"()LWrongOne;", "the unrelated return type"),
        (b"()[LWrongOn;", "the array return type"),
    ];
    for (replacement, description) in cases {
        assert_eq!(target.len(), replacement.len());
        let occurrences = ANCHOR_ROOT
            .windows(target.len())
            .filter(|window| **window == *target)
            .count();
        assert_eq!(occurrences, 1, "the anchor names its descriptor once");
        let start = ANCHOR_ROOT
            .windows(target.len())
            .position(|window| *window == *target)
            .expect("the anchor's descriptor is present");
        let mut patched = ANCHOR_ROOT.to_vec();
        patched[start..start + target.len()].copy_from_slice(replacement);
        let snapshot = open(zip_of(&[
            (b"AnonymousTopLevel.class", &patched),
            (b"AnonymousTopLevel$1.class", ANCHOR_CHILD),
            (b"Base.class", ANCHOR_BASE),
            (b"Renderer.class", ANCHOR_RENDERER),
        ]));
        let root = class_source_of(&snapshot, "AnonymousTopLevel");
        assert!(
            root.text.contains("new AnonymousTopLevel$1("),
            "{description} keeps the physical class text:\n{}",
            root.text
        );
        assert!(
            root.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "anonymous_super_return_type_unproved"),
            "{description} refuses with the return gate: {:?}",
            root.diagnostics
        );
    }
}

/// The report's text minus the `// jarde:` bookkeeping preamble: the compilable source region.
fn projected_source(text: &str) -> String {
    text.lines()
        .filter(|line| !line.starts_with("// jarde:"))
        .map(|line| format!("{line}\n"))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// The library side of every case
// ---------------------------------------------------------------------------------------------

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("the fixture snapshot opens")
}

fn performed<T>(outcome: OperationOutcome<T>) -> T {
    match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "expected one bound class, got {} candidate(s) and no execution",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "expected one bound class, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    performed(
        Engine::new()
            .class_source(
                slice::from_ref(snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal(name),
                    },
                    environment: EnvironmentRequest {
                        snapshot: snapshot.id().clone(),
                        scope: PhysicalScope::SnapshotAll,
                        policy: EnvironmentPolicy::PlainJar,
                        profile: RuntimeProfile {
                            java_release: 8,
                            multi_release: MultiReleasePolicy::Disabled,
                            layout: LayoutMode::Generic,
                        },
                        loader: LoaderId("app".to_owned()),
                    },
                },
                &mut budget(),
            )
            .expect("a legal class-source request is answered"),
    )
}

/// A stored-only archive, built with the repository's own `rawzip` dev-dependency.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is written");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// The original fixture classes on disk, for the javac classpath and the baseline run.
fn write_originals(scratch: &Scratch, entries: &[(&[u8], &[u8])]) {
    for (name, data) in entries {
        let path = scratch.path().join(String::from_utf8_lossy(name).as_ref());
        fs::write(path, data).expect("write the original fixture class");
    }
}

/// One throwaway directory per compile-and-run case, removed with the test.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-supertype-return-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the supertype-return scratch directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
