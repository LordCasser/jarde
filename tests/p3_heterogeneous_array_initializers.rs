//! EM-18 heterogeneous reference-array initializer proof and constructor-composition boundary.
//!
//! `factory` is the complete positive type-proof family: the array initializer's actual element
//! is a typed call result. `direct` is a verifier-valid family with recovered constructor/store
//! methods and nested covariant child-array initializers. Code-derived negative controls stay
//! separate and are never executed. Earlier direct reference-element
//! shapes have a separate seven-class generated-source integration fixture; the newly supported
//! primitive-conversion wrapper arguments are pinned here and in the numeric conversion family
//! test `p3_constructor_primitive_conversion_arguments.rs`.

use jarde::*;
use jarde_java::DeclaringClass;
use jarde_java::pass::JAVA_8;
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::budget::Budget;
use jarde_reader::classfile::{CpEntryKind, class_facts, method_code_facts};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const STORE: u16 = 0;
static NEXT: AtomicU64 = AtomicU64::new(0);

const EXPECTED_STDOUT: &[u8] = include_bytes!(
    "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac8/logs/factory_javac8_original-main.stdout"
);
const EXPECTED_STDERR: &[u8] = include_bytes!(
    "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac8/logs/factory_javac8_original-main.stderr"
);

const FACTORY_JAVAC8: &[(&str, &[u8])] = &[
    (
        "Main.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac8/classes/Main.class"
        ),
    ),
    (
        "Base.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac8/classes/Base.class"
        ),
    ),
    (
        "Mid.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac8/classes/Mid.class"
        ),
    ),
    (
        "DerivedA.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac8/classes/DerivedA.class"
        ),
    ),
    (
        "DerivedB.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac8/classes/DerivedB.class"
        ),
    ),
    (
        "LocalInterface.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac8/classes/LocalInterface.class"
        ),
    ),
];
const FACTORY_JAVAC23: &[(&str, &[u8])] = &[
    (
        "Main.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac23/classes/Main.class"
        ),
    ),
    (
        "Base.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac23/classes/Base.class"
        ),
    ),
    (
        "Mid.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac23/classes/Mid.class"
        ),
    ),
    (
        "DerivedA.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac23/classes/DerivedA.class"
        ),
    ),
    (
        "DerivedB.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac23/classes/DerivedB.class"
        ),
    ),
    (
        "LocalInterface.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/factory/javac23/classes/LocalInterface.class"
        ),
    ),
];
const DIRECT_JAVAC8: &[(&str, &[u8])] = &[
    (
        "Main.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac8/classes/Main.class"
        ),
    ),
    (
        "Base.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac8/classes/Base.class"
        ),
    ),
    (
        "Mid.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac8/classes/Mid.class"
        ),
    ),
    (
        "DerivedA.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac8/classes/DerivedA.class"
        ),
    ),
    (
        "DerivedB.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac8/classes/DerivedB.class"
        ),
    ),
    (
        "LocalInterface.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac8/classes/LocalInterface.class"
        ),
    ),
];
const DIRECT_JAVAC23: &[(&str, &[u8])] = &[
    (
        "Main.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac23/classes/Main.class"
        ),
    ),
    (
        "Base.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac23/classes/Base.class"
        ),
    ),
    (
        "DerivedA.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac23/classes/DerivedA.class"
        ),
    ),
    (
        "DerivedB.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac23/classes/DerivedB.class"
        ),
    ),
    (
        "Mid.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac23/classes/Mid.class"
        ),
    ),
    (
        "LocalInterface.class",
        include_bytes!(
            "fixtures/p3-heterogeneous-array-initializers-v3/direct/javac23/classes/LocalInterface.class"
        ),
    ),
];

struct Leg {
    name: &'static str,
    family: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}

const LEGS: &[Leg] = &[
    Leg {
        name: "javac8",
        family: "factory",
        files: FACTORY_JAVAC8,
    },
    Leg {
        name: "javac23",
        family: "factory",
        files: FACTORY_JAVAC23,
    },
    Leg {
        name: "javac8",
        family: "direct",
        files: DIRECT_JAVAC8,
    },
    Leg {
        name: "javac23",
        family: "direct",
        files: DIRECT_JAVAC23,
    },
];

const FACTORY_STORES: &[(&str, &[u32])] = &[
    ("boxedFactory", &[14, 24, 34, 44, 54, 65]),
    ("sequenceFactory", &[13, 23]),
    ("collectionFactory", &[10, 17]),
    ("throwableFactory", &[10, 17]),
    ("numberGridFactory", &[13, 23]),
    ("collectionGridFactory", &[13, 23]),
    ("ownTwoHopFactory", &[13, 23]),
    ("ownInterfaceFactory", &[13, 23]),
    ("ownGridFactory", &[13, 23]),
    ("exactNumber", &[13]),
    ("objectElement", &[13]),
    ("nullElement", &[7]),
];

#[derive(Clone, Copy)]
struct DirectPresentedSite {
    class: &'static str,
    head: u32,
    dup: u32,
    constructor: u32,
    argument: u32,
    store: u32,
}

const DIRECT_SITE_PRESENTATIONS: &[(&str, &[DirectPresentedSite])] = &[
    (
        "boxedDirect",
        &[
            DirectPresentedSite {
                class: "java/lang/Byte",
                head: 7,
                dup: 10,
                constructor: 16,
                argument: 15,
                store: 19,
            },
            DirectPresentedSite {
                class: "java/lang/Short",
                head: 22,
                dup: 25,
                constructor: 31,
                argument: 30,
                store: 34,
            },
            DirectPresentedSite {
                class: "java/lang/Integer",
                head: 37,
                dup: 40,
                constructor: 45,
                argument: 42,
                store: 48,
            },
            DirectPresentedSite {
                class: "java/lang/Long",
                head: 51,
                dup: 54,
                constructor: 60,
                argument: 59,
                store: 63,
            },
            DirectPresentedSite {
                class: "java/lang/Float",
                head: 66,
                dup: 69,
                constructor: 75,
                argument: 74,
                store: 78,
            },
            DirectPresentedSite {
                class: "java/lang/Double",
                head: 81,
                dup: 84,
                constructor: 91,
                argument: 90,
                store: 94,
            },
        ],
    ),
    (
        "sequenceDirect",
        &[
            DirectPresentedSite {
                class: "java/lang/String",
                head: 6,
                dup: 9,
                constructor: 17,
                argument: 14,
                store: 20,
            },
            DirectPresentedSite {
                class: "java/lang/StringBuilder",
                head: 23,
                dup: 26,
                constructor: 34,
                argument: 31,
                store: 37,
            },
        ],
    ),
    (
        "collectionDirect",
        &[
            DirectPresentedSite {
                class: "java/util/ArrayList",
                head: 6,
                dup: 9,
                constructor: 20,
                argument: 17,
                store: 23,
            },
            DirectPresentedSite {
                class: "java/util/HashSet",
                head: 26,
                dup: 29,
                constructor: 40,
                argument: 37,
                store: 43,
            },
        ],
    ),
    (
        "throwableDirect",
        &[
            DirectPresentedSite {
                class: "java/lang/IllegalStateException",
                head: 6,
                dup: 9,
                constructor: 17,
                argument: 14,
                store: 20,
            },
            DirectPresentedSite {
                class: "java/lang/IllegalArgumentException",
                head: 23,
                dup: 26,
                constructor: 34,
                argument: 31,
                store: 37,
            },
        ],
    ),
    (
        "ownTwoHopDirect",
        &[
            DirectPresentedSite {
                class: "DerivedA",
                head: 6,
                dup: 9,
                constructor: 14,
                argument: 11,
                store: 17,
            },
            DirectPresentedSite {
                class: "DerivedB",
                head: 20,
                dup: 23,
                constructor: 28,
                argument: 25,
                store: 31,
            },
        ],
    ),
    (
        "ownInterfaceDirect",
        &[
            DirectPresentedSite {
                class: "DerivedA",
                head: 6,
                dup: 9,
                constructor: 14,
                argument: 11,
                store: 17,
            },
            DirectPresentedSite {
                class: "DerivedB",
                head: 20,
                dup: 23,
                constructor: 28,
                argument: 25,
                store: 31,
            },
        ],
    ),
    (
        "ownGridDirect",
        &[
            DirectPresentedSite {
                class: "DerivedA",
                head: 12,
                dup: 15,
                constructor: 20,
                argument: 17,
                store: 23,
            },
            DirectPresentedSite {
                class: "DerivedB",
                head: 33,
                dup: 36,
                constructor: 41,
                argument: 38,
                store: 44,
            },
        ],
    ),
];

const DIRECT_GRID_FACTS: &[(&str, &[u32], &[u32], &[u32], &[&str])] = &[
    (
        "numberGridDirect",
        &[1, 7, 24],
        &[19, 20, 37, 38],
        &[13, 16, 30, 33, 34],
        &[
            "new java.lang.Number[][]",
            "new java.lang.Integer[]",
            "new java.lang.Long[]",
            "(long) mark(2)",
        ],
    ),
    (
        "collectionGridDirect",
        &[1, 7, 24],
        &[19, 20, 36, 37],
        &[13, 16, 30, 33],
        &[
            "new java.util.Collection[][]",
            "new java.util.ArrayList[]",
            "new java.util.HashSet[]",
            "listValue(",
            "setValue(",
        ],
    ),
    (
        "ownGridDirect",
        &[1, 7, 28],
        &[23, 24, 44, 45],
        &[12, 15, 17, 20, 33, 36, 38, 41],
        &["new Base[][]", "new DerivedA[]", "new DerivedB[]"],
    ),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn archive(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in files {
            let (mut entry, config) = writer
                .new_file(EntryPath::verbatim(name.as_bytes().to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("class entry starts");
            let mut stream = config.wrap(&mut entry);
            stream.write_all(bytes).expect("class entry is written");
            let (_, descriptor) = stream.finish().expect("class entry closes");
            entry.finish(descriptor).expect("class entry finishes");
        }
        writer
            .finish()
            .expect("one complete fixture archive closes");
    }
    output.into_inner()
}

fn opened(bytes: &[u8], class: &str) -> (artifact::ArtifactSnapshot, ClassSourceRequest) {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("one same-archive class family opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
    };
    (snapshot, request)
}

fn class_source(snapshot: &artifact::ArtifactSnapshot, class: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("valid same-archive class-source request answers")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("{class} class source did not complete: {other:?}"),
    }
}

struct OwnGridCodeLayout {
    code_start: usize,
    code_length: usize,
    code_length_offset: usize,
    attribute_length_offset: usize,
    attribute_content_length: usize,
    max_stack: u16,
}

fn own_grid_code_layout(class: &[u8]) -> OwnGridCodeLayout {
    let mut read_budget = budget();
    let facts = class_facts(class, &mut read_budget).expect("direct Main class facts");
    let method = facts
        .methods
        .iter()
        .find(|method| {
            method.name.raw().0 == b"ownGridDirect" && method.descriptor.raw().0 == b"()[[LBase;"
        })
        .expect("ownGridDirect descriptor anchor");
    let code_attributes: Vec<_> = method
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0 == b"Code")
        .collect();
    let [code_attribute] = code_attributes.as_slice() else {
        panic!("ownGridDirect has exactly one Code attribute");
    };
    let mut code_budget = budget();
    let code =
        method_code_facts(class, method, &mut code_budget).expect("ownGridDirect code facts");
    let content_start =
        usize::try_from(code_attribute.content_span.start).expect("Code content offset fits usize");
    assert_eq!(
        usize::try_from(code.code_span.start).expect("bytecode offset fits usize"),
        content_start + 8,
        "the Code header precedes its first opcode"
    );
    assert_eq!(
        code.code_span.length, 47,
        "frozen ownGridDirect Code length"
    );
    let code_start = content_start + 8;
    let code_length_offset = content_start + 4;
    let attribute_length_offset =
        usize::try_from(code_attribute.span.start).expect("attribute offset fits usize") + 2;
    let code_length = read_u32(class, code_length_offset) as usize;
    assert_eq!(code_length, code.code_span.length as usize);
    let code_end = code_start + code_length;
    assert_eq!(
        read_u16(class, code_end),
        0,
        "ownGridDirect has no handlers"
    );
    assert_eq!(
        read_u16(class, code_end + 2),
        0,
        "ownGridDirect has no Code subattributes"
    );
    assert_eq!(
        code_end + 4,
        content_start + usize::try_from(code_attribute.content_span.length).unwrap(),
        "Code content ends after its terminal counts"
    );
    let max_stack = read_u16(class, content_start);
    assert_eq!(max_stack, 9, "frozen direct class max_stack");
    OwnGridCodeLayout {
        code_start,
        code_length,
        code_length_offset,
        attribute_length_offset,
        attribute_content_length: usize::try_from(code_attribute.content_span.length)
            .expect("Code content length fits usize"),
        max_stack,
    }
}

fn own_grid_constant(class: &[u8], wanted: impl Fn(&CpEntryKind) -> bool) -> u16 {
    let mut read_budget = budget();
    class_facts(class, &mut read_budget)
        .expect("direct Main class facts")
        .constant_pool
        .iter()
        .find(|entry| wanted(&entry.kind))
        .expect("ownGridDirect references the required constant")
        .index
}

fn read_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().expect("u2 is in range"))
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().expect("u4 is in range"))
}

fn write_u16(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

fn write_u32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_be_bytes());
}

fn own_grid_code_variant(class: &[u8], kind: OwnGridVariant) -> Vec<u8> {
    let layout = own_grid_code_layout(class);
    let mut code = class[layout.code_start..layout.code_start + layout.code_length].to_vec();
    match kind {
        OwnGridVariant::Reordered => {
            assert_eq!(
                (code[5], code[26]),
                (0x03, 0x04),
                "parent indices are 0 then 1"
            );
            code[5] = 0x04;
            code[26] = 0x03;
        }
        OwnGridVariant::ExtraReader => {
            assert_eq!(code[23], 0x53, "child element store is at BCI 23");
            code.splice(24..24, [0x59, 0x57]);
        }
        OwnGridVariant::InterleavedEffect => {
            assert_eq!(code[23], 0x53, "child element store is at BCI 23");
            let method = own_grid_constant(class, |entry| {
                matches!(entry, CpEntryKind::MethodRef { owner, name, descriptor, .. }
                    if owner.0 == b"Main" && name.0 == b"mark" && descriptor.0 == b"(I)I")
            });
            let mut effect = vec![0x04, 0xb8];
            effect.extend_from_slice(&method.to_be_bytes());
            effect.push(0x57);
            assert_eq!(effect.len(), 5, "iconst_1; invokestatic; pop is five bytes");
            code.splice(24..24, effect);
        }
        OwnGridVariant::ObjectChild => {
            assert_eq!(code[7], 0xbd, "DerivedA child uses anewarray at BCI 7");
            let object = own_grid_constant(
                class,
                |entry| matches!(entry, CpEntryKind::Class { name, .. } if name.0 == b"java/lang/Object"),
            );
            write_u16(&mut code, 8, object);
        }
    }

    let mut variant = class.to_vec();
    let code_end = layout.code_start + layout.code_length;
    variant.splice(layout.code_start..code_end, code.iter().copied());
    let code_length = u32::try_from(code.len()).expect("patched Code length fits u4");
    write_u32(&mut variant, layout.code_length_offset, code_length);
    let content_length = layout
        .attribute_content_length
        .checked_sub(layout.code_length)
        .and_then(|length| length.checked_add(code.len()))
        .expect("patched Code attribute length fits usize");
    let old_attribute_length = read_u32(class, layout.attribute_length_offset);
    assert_eq!(
        old_attribute_length as usize,
        layout.attribute_content_length
    );
    write_u32(
        &mut variant,
        layout.attribute_length_offset,
        u32::try_from(content_length).expect("Code attribute length fits u4"),
    );
    assert_eq!(
        variant.len(),
        class.len() - layout.code_length + code.len(),
        "only ownGridDirect Code bytes and its length fields change"
    );
    if matches!(
        kind,
        OwnGridVariant::ExtraReader | OwnGridVariant::InterleavedEffect
    ) {
        assert!(
            layout.max_stack >= 4,
            "the existing stack bound covers the insertion"
        );
    }
    let expected_delta = match kind {
        OwnGridVariant::Reordered | OwnGridVariant::ObjectChild => 0,
        OwnGridVariant::ExtraReader => 2,
        OwnGridVariant::InterleavedEffect => 5,
    };
    assert_eq!(
        code.len(),
        47 + expected_delta,
        "the patched control has its expected exact Code length"
    );
    variant
}

#[derive(Clone, Copy)]
enum OwnGridVariant {
    Reordered,
    ExtraReader,
    InterleavedEffect,
    ObjectChild,
}

fn recovered_direct_grid_body(
    files: &[(&str, &[u8])],
    variant: Option<OwnGridVariant>,
) -> RecoveryReport {
    let patched_main = variant.map(|kind| {
        own_grid_code_variant(
            files
                .iter()
                .find(|(name, _)| *name == "Main.class")
                .expect("Main.class")
                .1,
            kind,
        )
    });
    let selected: Vec<_> = files
        .iter()
        .map(|(name, bytes)| {
            if *name == "Main.class" {
                (*name, patched_main.as_deref().unwrap_or(*bytes))
            } else {
                (*name, *bytes)
            }
        })
        .collect();
    let jar = archive(&selected);
    let (snapshot, _) = opened(&jar, "Main");
    let report = class_source(&snapshot, "Main");
    body(&report, "ownGridDirect").clone()
}

fn method_only_own_grid(class: &[u8]) -> RecoveryReport {
    let mut open_budget = budget();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut open_budget)
        .expect("standalone Main opens");
    let facts = class_facts(class, &mut open_budget).expect("standalone Main facts");
    let header = facts
        .methods
        .iter()
        .find(|method| {
            method.name.raw().0 == b"ownGridDirect" && method.descriptor.raw().0 == b"()[[LBase;"
        })
        .expect("ownGridDirect method-only anchor");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("class length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(b"ownGridDirect".to_vec()),
        descriptor: JvmBytes(b"()[[LBase;".to_vec()),
    };
    let request = MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        },
        method,
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut analysis_budget = budget();
    let analysis = analyze_method_ir(
        std::slice::from_ref(&snapshot),
        &request,
        &mut analysis_budget,
    )
    .expect("method-only SSA analysis completes");
    let recovery_facts = RecoveryFacts::new(
        MethodFacts::new("ownGridDirect".to_owned(), "()[[LBase;".to_owned(), 0)
            .with_access_flags(header.access_flags)
            .with_declaring_class(DeclaringClass::new(
                String::from_utf8_lossy(&facts.this_class.raw().0).into_owned(),
                facts.access_flags,
            )),
    );
    recover(
        &RecoveryRequest::new(analysis.ir(), &recovery_facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::essential()),
        &mut analysis_budget,
    )
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("class has no method `{name}`"))
}

fn body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &method(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("{name} has no complete recovery record: {other:?}"),
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-heterogeneous-array-{}-{label}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("private fixture scratch directory is created");
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

fn run_verified(classes: &Path, main: &str) -> Output {
    Command::new(java_tool())
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(classes)
        .arg(main)
        .output()
        .expect("java is available for verifier execution")
}

fn java_tool() -> PathBuf {
    std::env::var_os("JARDE_JAVA23")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("java"))
}

fn javac_tool() -> PathBuf {
    std::env::var_os("JARDE_JAVAC23")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("javac"))
}

fn compile_complete(dir: &Path, source_names: &[&str]) -> Output {
    let empty = dir.join("empty-classpath-sourcepath");
    let classes = dir.join("classes");
    fs::create_dir_all(&empty).expect("empty source/class path exists");
    fs::create_dir_all(&classes).expect("isolated output classes exist");
    let mut command = Command::new(javac_tool());
    command
        .args([
            "-source",
            "8",
            "-target",
            "8",
            "-g:none",
            "-Xlint:-options",
            "-classpath",
        ])
        .arg(&empty)
        .arg("-sourcepath")
        .arg(&empty)
        .arg("-d")
        .arg(&classes);
    for name in source_names {
        command.arg(dir.join(name));
    }
    command
        .current_dir(dir)
        .output()
        .expect("javac is available for Java 8 source compilation")
}

fn class_names() -> [&'static str; 6] {
    [
        "Main",
        "Base",
        "Mid",
        "DerivedA",
        "DerivedB",
        "LocalInterface",
    ]
}

fn save_original_classes(files: &[(&str, &[u8])], dir: &Path) {
    fs::create_dir_all(dir).expect("original class-only directory exists");
    for (name, bytes) in files {
        fs::write(dir.join(name), bytes).expect("frozen original class is written");
    }
}

#[test]
fn factory_families_are_complete_mapped_and_match_both_frozen_jdk_legs() {
    for leg in LEGS.iter().filter(|leg| leg.family == "factory") {
        let jar = archive(leg.files);
        let (snapshot, _) = opened(&jar, "Main");
        let generated = Scratch::new(&format!("{}-{}-generated", leg.family, leg.name));
        let recovered = generated.path().join("sources");
        fs::create_dir_all(&recovered).unwrap();

        // All classes are rendered from the one six-class snapshot. No support class comes from an
        // original classpath or a different class-source request.
        for class in class_names() {
            let report = class_source(&snapshot, class);
            assert!(report.declaration.is_some(), "{class} declaration missing");
            // The positive contract is scoped to the initializer methods below. Generic Signature
            // projection on the ArrayList/HashSet factory declarations remains a separate marker.
            assert!(
                !report.text.contains("@bytecode"),
                "{}/{class}: {}",
                leg.name,
                report.text
            );
            assert!(
                !report.text.contains("jarde_refused_body"),
                "{}/{class}: {}",
                leg.name,
                report.text
            );
            fs::write(recovered.join(format!("{class}.java")), &report.text)
                .expect("complete generated class source is written");
            if class == "Main" {
                for (method_name, stores) in FACTORY_STORES {
                    let recovered_body = body(&report, method_name);
                    assert!(
                        recovered_body.produced(),
                        "{method_name}: {recovered_body:?}"
                    );
                    assert_eq!(recovered_body.quality, Quality::Structured, "{method_name}");
                    assert_eq!(
                        recovered_body.representation,
                        Representation::Java,
                        "{method_name}"
                    );
                    assert!(
                        !recovered_body.text.contains("@bytecode"),
                        "{method_name}: {}",
                        recovered_body.text
                    );
                    for bci in *stores {
                        assert!(
                            !recovered_body.source_map.of_bci(*bci).is_empty(),
                            "{method_name} omitted actual javap aastore BCI {bci}: {:?}",
                            recovered_body.source_map.segments()
                        );
                    }
                }
            }
        }

        let names = [
            "Main.java",
            "Base.java",
            "Mid.java",
            "DerivedA.java",
            "DerivedB.java",
            "LocalInterface.java",
        ];
        let compile = compile_complete(&recovered, &names);
        assert!(
            compile.status.success(),
            "recovered {} {} family did not compile:\n{}\n{}",
            leg.family,
            leg.name,
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );

        let original = generated.path().join("original-classes");
        save_original_classes(leg.files, &original);
        let original_run = run_verified(&original, "Main");
        assert!(
            original_run.status.success(),
            "original {} verifier run failed: {}",
            leg.name,
            String::from_utf8_lossy(&original_run.stderr)
        );
        assert_eq!(
            original_run.stdout, EXPECTED_STDOUT,
            "{} original stdout differs from its frozen oracle",
            leg.name
        );
        assert_eq!(
            original_run.stderr, EXPECTED_STDERR,
            "{} original stderr differs from its frozen oracle",
            leg.name
        );

        let recovered_run = run_verified(&recovered.join("classes"), "Main");
        assert!(
            recovered_run.status.success(),
            "recovered {} verifier run failed:\n{}\n{}",
            leg.name,
            String::from_utf8_lossy(&recovered_run.stdout),
            String::from_utf8_lossy(&recovered_run.stderr)
        );
        assert_eq!(recovered_run.status.code(), original_run.status.code());
        assert_eq!(
            recovered_run.stdout, original_run.stdout,
            "{} recovered stdout differs",
            leg.name
        );
        assert_eq!(
            recovered_run.stderr, original_run.stderr,
            "{} recovered stderr differs",
            leg.name
        );
        assert_eq!(recovered_run.stdout, EXPECTED_STDOUT);
        assert_eq!(recovered_run.stderr, EXPECTED_STDERR);
    }
}

#[test]
fn direct_new_family_pins_recovered_constructor_methods_and_remaining_controls() {
    for leg in LEGS.iter().filter(|leg| leg.family == "direct") {
        let jar = archive(leg.files);
        let (snapshot, _) = opened(&jar, "Main");
        let report = class_source(&snapshot, "Main");
        assert!(
            !report.text.contains("@bytecode") && !report.text.contains("jarde_refused_body"),
            "{}/Main must be complete after all direct grids recover:\n{}",
            leg.name,
            report.text
        );
        let collection_declaration = method(&report, "collectionGridDirect")
            .declaration
            .as_deref();
        assert_eq!(
            collection_declaration,
            Some("public static java.util.Collection<?>[][] collectionGridDirect()"),
            "{}/collectionGridDirect must project the source Signature without changing its body",
            leg.name
        );
        assert!(
            !method(&report, "collectionGridDirect")
                .text
                .contains("generic Signature projection refused"),
            "{}/collectionGridDirect retained a Signature refusal:\n{}",
            leg.name,
            method(&report, "collectionGridDirect").text
        );
        assert!(
            body(&report, "collectionGridDirect")
                .text
                .contains("new java.util.Collection[][]"),
            "{}/collectionGridDirect must keep the raw array creation text",
            leg.name
        );
        let original = Scratch::new(&format!("direct-{}-oracle", leg.name));
        let classes = original.path().join("original-classes");
        save_original_classes(leg.files, &classes);
        let oracle = run_verified(&classes, "Main");
        assert!(
            oracle.status.success(),
            "direct-new original {} failed: {}",
            leg.name,
            String::from_utf8_lossy(&oracle.stderr)
        );
        assert_eq!(
            oracle.stdout, EXPECTED_STDOUT,
            "direct-new original {} changed",
            leg.name
        );
        assert_eq!(
            oracle.stderr, EXPECTED_STDERR,
            "direct-new original {} wrote stderr",
            leg.name
        );

        for (method_name, expected_sites) in DIRECT_SITE_PRESENTATIONS {
            let recovered = body(&report, method_name);
            assert_eq!(
                recovered.quality,
                Quality::Structured,
                "{}/{method_name} quality",
                leg.name
            );
            assert_eq!(
                recovered.representation,
                Representation::Java,
                "{}/{method_name} representation",
                leg.name
            );
            assert!(
                !recovered.text.contains("@bytecode")
                    && !recovered.text.contains("jarde_refused_body"),
                "{}/{method_name} retained a refusal marker:\n{}",
                leg.name,
                recovered.text
            );
            assert_eq!(
                recovered.news.len(),
                expected_sites.len(),
                "{}/{method_name} construction records: {:?}",
                leg.name,
                recovered.news
            );
            for expected in *expected_sites {
                let matching = recovered
                    .news
                    .iter()
                    .filter(|site| site.head == expected.head)
                    .collect::<Vec<_>>();
                assert_eq!(
                    matching.len(),
                    1,
                    "{}/{method_name} new@{} record",
                    leg.name,
                    expected.head
                );
                let record = matching[0];
                assert_eq!(record.class, expected.class);
                assert_eq!(record.dup, Some(expected.dup));
                assert_eq!(record.constructor, Some(expected.constructor));
                assert_eq!(record.arguments.as_slice(), &[expected.argument]);
                assert!(record.presented, "{}/{method_name}: {record:?}", leg.name);
                assert!(
                    record.refusal.is_none(),
                    "{}/{method_name}: {record:?}",
                    leg.name
                );
                for bci in [
                    expected.head,
                    expected.dup,
                    expected.argument,
                    expected.constructor,
                    expected.store,
                ] {
                    assert!(
                        !recovered.source_map.of_bci(bci).is_empty(),
                        "{}/{method_name} omitted new/dup/argument/init/aastore BCI {bci}",
                        leg.name
                    );
                }
            }
        }

        let boxed = body(&report, "boxedDirect");
        for (cast, call) in [
            ("(byte) mark(1)", "new java.lang.Byte("),
            ("(short) mark(2)", "new java.lang.Short("),
            ("mark(3)", "new java.lang.Integer("),
            ("(long) mark(4)", "new java.lang.Long("),
            ("(float) mark(5)", "new java.lang.Float("),
            ("(double) mark(6)", "new java.lang.Double("),
        ] {
            assert_eq!(
                boxed.text.matches(cast).count(),
                1,
                "{}/boxedDirect must render `{cast}` once:\n{}",
                leg.name,
                boxed.text
            );
            assert_eq!(
                boxed.text.matches(call).count(),
                1,
                "{}/boxedDirect must render `{call}` once:\n{}",
                leg.name,
                boxed.text
            );
        }
        assert_eq!(
            boxed.text.matches("mark(").count(),
            6,
            "{}/boxedDirect must invoke each side-effecting mark once:\n{}",
            leg.name,
            boxed.text
        );

        for (method_name, allocations, stores, producers, fragments) in DIRECT_GRID_FACTS {
            let recovered = body(&report, method_name);
            assert_eq!(
                recovered.quality,
                Quality::Structured,
                "{}/{method_name} quality: {:?}",
                leg.name,
                recovered
            );
            assert_eq!(
                recovered.representation,
                Representation::Java,
                "{}/{method_name} representation",
                leg.name
            );
            assert!(
                !recovered.text.contains("@bytecode")
                    && !recovered.text.contains("jarde_refused_body"),
                "{}/{method_name} retained a refusal:\n{}",
                leg.name,
                recovered.text
            );
            assert!(
                fragments
                    .iter()
                    .all(|fragment| recovered.text.contains(fragment)),
                "{}/{method_name} lost an expected typed initializer fragment:\n{}",
                leg.name,
                recovered.text
            );
            assert_eq!(
                recovered.text.matches("mark(").count(),
                2,
                "{}/{method_name} must evaluate each marked child element once:\n{}",
                leg.name,
                recovered.text
            );
            for bci in allocations
                .iter()
                .chain(stores.iter())
                .chain(producers.iter())
            {
                assert!(
                    !recovered.source_map.of_bci(*bci).is_empty(),
                    "{}/{method_name} omitted allocation/store/constructor/conversion source BCI {bci}: {:?}",
                    leg.name,
                    recovered.source_map.segments()
                );
            }
        }
        for name in ["numberGridDirect", "collectionGridDirect"] {
            assert!(
                body(&report, name).news.is_empty(),
                "{}/{name} has no object-constructor records: {:?}",
                leg.name,
                body(&report, name).news
            );
        }

        // Compile all six complete direct-family source outputs in an empty source/class path and
        // compare a verifier run with the frozen original. This is the complete-family contract,
        // not just three locally structured method bodies.
        let recovered_dir = original.path().join("recovered");
        fs::create_dir_all(&recovered_dir).expect("recovered source directory exists");
        for class in class_names() {
            if class == "Main" {
                fs::write(recovered_dir.join("Main.java"), &report.text)
                    .expect("complete Main source is saved");
            } else {
                let companion = class_source(&snapshot, class);
                assert!(
                    !companion.text.contains("@bytecode")
                        && !companion.text.contains("jarde_refused_body"),
                    "{}/{class} must be a complete family source:\n{}",
                    leg.name,
                    companion.text
                );
                fs::write(recovered_dir.join(format!("{class}.java")), companion.text)
                    .expect("complete companion source is saved");
            }
        }
        let compile = compile_complete(
            &recovered_dir,
            &[
                "Main.java",
                "Base.java",
                "Mid.java",
                "DerivedA.java",
                "DerivedB.java",
                "LocalInterface.java",
            ],
        );
        assert!(
            compile.status.success(),
            "direct {} complete family did not compile:\n{}\n{}",
            leg.name,
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let recovered_run = run_verified(&recovered_dir.join("classes"), "Main");
        assert!(
            recovered_run.status.success(),
            "direct {} recovered family failed -Xverify:all:\n{}\n{}",
            leg.name,
            String::from_utf8_lossy(&recovered_run.stdout),
            String::from_utf8_lossy(&recovered_run.stderr)
        );
        assert_eq!(recovered_run.status.code(), oracle.status.code());
        assert_eq!(recovered_run.stdout, oracle.stdout, "{} stdout", leg.name);
        assert_eq!(recovered_run.stderr, oracle.stderr, "{} stderr", leg.name);
    }
}

#[test]
fn direct_child_array_controls_keep_order_reader_effect_and_type_boundaries() {
    for leg in LEGS.iter().filter(|leg| leg.family == "direct") {
        for (variant, label) in [
            (OwnGridVariant::Reordered, "reordered parent indices"),
            (OwnGridVariant::ExtraReader, "extra child-array reader"),
            (
                OwnGridVariant::InterleavedEffect,
                "interleaved child-parent effect",
            ),
        ] {
            let body = recovered_direct_grid_body(leg.files, Some(variant));
            assert_eq!(
                body.quality,
                Quality::Fallback,
                "{}/{label} must reject the complete ownGridDirect body:\n{}",
                leg.name,
                body.text
            );
            assert!(
                body.text.contains("@bytecode") && !body.text.contains("new Base[][]"),
                "{}/{label} must not publish a partial nested initializer:\n{}",
                leg.name,
                body.text
            );
        }

        let object_array = recovered_direct_grid_body(leg.files, Some(OwnGridVariant::ObjectChild));
        assert_eq!(
            object_array.quality,
            Quality::Fallback,
            "{}/Object[] child must fail Java type presentation:\n{}",
            leg.name,
            object_array.text
        );
        assert!(
            object_array.text.contains("java.lang.Object[]")
                && object_array.text.contains("Base[]")
                && object_array.text.contains("@bytecode")
                && !object_array.text.contains("new Base[][]"),
            "{}/Object[] child must preserve the actual incompatible type pair:\n{}",
            leg.name,
            object_array.text
        );

        // Missing Mid removes the selected DerivedA -> Base path. Other direct/platform facts
        // remain available, while the complete own-grid body cannot borrow a different route.
        let files: Vec<_> = leg
            .files
            .iter()
            .copied()
            .filter(|(name, _)| *name != "Mid.class")
            .collect();
        assert_eq!(files.len(), 5);
        let jar = archive(&files);
        let (snapshot, _) = opened(&jar, "Main");
        let report = class_source(&snapshot, "Main");
        let missing_mid = body(&report, "ownGridDirect");
        assert_eq!(missing_mid.quality, Quality::Fallback);
        assert!(
            missing_mid.text.contains("@bytecode") && !missing_mid.text.contains("new Base[][]"),
            "{}/missing Mid cannot present the own-grid body:\n{}",
            leg.name,
            missing_mid.text
        );

        // Method-only recovery has no selected snapshot hierarchy facts. Even though the input
        // class carries the caller bytecode, it cannot infer DerivedA -> Base from its name.
        let method_only = method_only_own_grid(
            leg.files
                .iter()
                .find(|(name, _)| *name == "Main.class")
                .expect("Main class")
                .1,
        );
        assert_eq!(method_only.quality, Quality::Fallback);
        assert!(
            method_only.text.contains("@bytecode") && !method_only.text.contains("new Base[][]"),
            "{}/method-only recovery cannot infer the hierarchy:\n{}",
            leg.name,
            method_only.text
        );
    }
}

#[test]
fn missing_mid_header_refuses_only_initializers_whose_derived_path_uses_mid() {
    for leg in LEGS.iter().filter(|leg| leg.family == "factory") {
        let files: Vec<_> = leg
            .files
            .iter()
            .copied()
            .filter(|(name, _)| *name != "Mid.class")
            .collect();
        assert_eq!(files.len(), 5, "only Mid.class is removed");
        assert!(!files.iter().any(|(name, _)| *name == "Mid.class"));

        let jar = archive(&files);
        let (snapshot, _) = opened(&jar, "Main");
        let report = class_source(&snapshot, "Main");

        // Base directly implements the interface; DerivedA reaches Base only through absent Mid.
        let base = class_source(&snapshot, "Base");
        let derived_a = class_source(&snapshot, "DerivedA");
        assert!(
            base.text.contains("implements LocalInterface"),
            "{}",
            base.text
        );
        assert!(derived_a.text.contains("extends Mid"), "{}", derived_a.text);
        assert!(
            !derived_a.text.contains("implements LocalInterface"),
            "{}",
            derived_a.text
        );

        for (name, element, component, initializer) in [
            ("ownTwoHopFactory", "DerivedA", "Base", "new Base[]{"),
            (
                "ownInterfaceFactory",
                "DerivedA",
                "LocalInterface",
                "new LocalInterface[]{",
            ),
            ("ownGridFactory", "DerivedA[]", "Base[]", "new Base[][]{"),
        ] {
            let member = method(&report, name);
            let recovered = body(&report, name);
            assert_eq!(
                recovered.quality,
                Quality::Fallback,
                "{name}: {recovered:?}"
            );
            assert_eq!(
                recovered.content,
                RecoveryContent::ExplanationOnly,
                "{name}"
            );
            assert!(
                member
                    .markers
                    .iter()
                    .any(|marker| marker.contains("not recovered")),
                "{name} has no member refusal marker: {:?}",
                member.markers
            );
            assert!(
                recovered.text.contains("@bytecode 24"),
                "{name}: {}",
                recovered.text
            );
            let refusal = format!(
                "the array initializer element at BCI 24 is presented as `{element}`, while the array component is `{component}`; this `aastore` has no compatible reference fact for a Java initializer"
            );
            assert!(
                recovered.text.contains(&refusal),
                "{name} lost the precise missing-path refusal: {}",
                recovered.text
            );
            assert!(
                !recovered.text.contains(initializer),
                "{name} generated a refused initializer: {}",
                recovered.text
            );
        }

        // The missing edge does not erase the direct DerivedB -> Base fact or unrelated platform facts.
        for name in [
            "boxedFactory",
            "numberGridFactory",
            "derivedB",
            "derivedBArray",
        ] {
            let recovered = body(&report, name);
            assert_eq!(
                recovered.quality,
                Quality::Structured,
                "{name}: {recovered:?}"
            );
            assert_eq!(recovered.representation, Representation::Java, "{name}");
        }
    }
}

#[test]
fn class_source_budget_and_cancellation_report_their_exact_stops() {
    let jar = archive(FACTORY_JAVAC23);
    let (snapshot, request) = opened(&jar, "Main");

    let complete = class_source(&snapshot, "Main");
    let mut output_limits = complete.limits.clone();
    output_limits.output_bytes = 0;
    let output_stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(output_limits),
        )
        .expect("zero output budget is an operation stop");
    let output_report = match output_stopped {
        OperationOutcome::Performed(report) => report,
        other => panic!(
            "zero output budget resolves the class and returns its partial report: {other:?}"
        ),
    };
    assert!(
        matches!(
            &output_report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::OutputBytes
                },
                ..
            }
        ),
        "zero output budget must report OutputBytes: {:?}",
        output_report.execution
    );
    assert_eq!(
        output_report.coverage.artifact_structural.state,
        CoverageState::Partial,
        "the class-source coverage reports the stopped member"
    );
    assert_eq!(
        output_report.coverage.dynamic_analysis.state,
        CoverageState::NotRequested
    );
    assert!(output_report.text.contains("budget_exceeded_output_bytes"));
    let stopped_member = output_report
        .methods
        .iter()
        .find(|member| {
            matches!(&member.outcome,
            ClassSourceOutcome::Recovered { report, .. } if report.stop().is_some())
        })
        .expect("the output budget stop remains on its member");
    let ClassSourceOutcome::Recovered {
        report: stopped_body,
        ..
    } = &stopped_member.outcome
    else {
        unreachable!("the selected member has a recovery report")
    };
    assert_eq!(stopped_body.content, RecoveryContent::NotProduced);
    assert!(stopped_body.text.is_empty());
    assert!(stopped_body.source_map.is_empty());
    assert!(matches!(
        &stopped_body.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        }
    ));

    let mut analysis_limits = complete.limits.clone();
    analysis_limits.analysis_steps = complete.usage.analysis_steps.saturating_sub(1);
    let analysis_stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(analysis_limits),
        )
        .expect("the bounded analysis is an operation stop");
    let analysis_report = match analysis_stopped {
        OperationOutcome::Performed(report) => report,
        other => panic!(
            "mid-analysis budget resolves the class and returns its partial report: {other:?}"
        ),
    };
    assert!(
        matches!(
            &analysis_report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        ),
        "mid-analysis budget must report AnalysisSteps: {:?}",
        analysis_report.execution
    );
    assert_eq!(
        analysis_report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        analysis_report.coverage.dynamic_analysis.state,
        CoverageState::NotRequested
    );
    assert!(
        analysis_report.usage.analysis_steps <= analysis_report.limits.analysis_steps,
        "post-body analysis usage must stay within its limit: usage={}, limit={}",
        analysis_report.usage.analysis_steps,
        analysis_report.limits.analysis_steps
    );
    assert!(
        analysis_report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_analysis_steps"),
        "the class report must identify the request-level analysis stop: {:?}",
        analysis_report
            .diagnostics
            .iter()
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>()
    );
    let recovered_members: Vec<_> = analysis_report
        .methods
        .iter()
        .filter_map(|member| match &member.outcome {
            ClassSourceOutcome::Recovered { report, .. } => Some(report),
            _ => None,
        })
        .collect();
    assert!(!recovered_members.is_empty());
    assert!(
        recovered_members
            .iter()
            .all(|report| matches!(&report.execution, ExecutionReport::Complete { .. })),
        "this budget stops after the body attempts, during class assembly; member executions: {:?}",
        recovered_members
            .iter()
            .map(|report| &report.execution)
            .collect::<Vec<_>>()
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(complete.limits.clone(), token);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("pre-cancellation is an operation outcome");
    assert!(
        matches!(&stopped, OperationOutcome::Incomplete(candidates)
        if matches!(&candidates.execution, ExecutionReport::Cancelled { .. })),
        "pre-cancelled request must report cancellation without publishing source: {stopped:?}"
    );
}
