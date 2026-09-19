//! P4 1.3's fixed replay list: the modern legality goldens.
//!
//! Each file under `tests/fixtures/p4-golden/` records one group of replays: the fixture the group
//! names (its generator, its provenance, and the blake3 digest and length of the bytes that
//! generator produces), and the answer the public reader entries returned for it — the header planes
//! ([`HeaderInspection`]) and the modern facts' own planes ([`ModernFacts`]), each with its
//! diagnostics verbatim, code, severity and message.
//!
//! The mechanism is P2 5.3's (`tests/p2_golden.rs`), field for field: a fixed replay list the tests
//! compare with what the files really hold, a fixture rebuilt from the same generator whose digest
//! and length must match the record, and a comparison of the serialized projection under `expect`.
//! Nothing regenerates itself: a missing, changed or unreadable golden is a test failure, not an
//! update.
//!
//! # What these entries are about
//!
//! The class files here are **structurally readable and illegal for their release** — not corrupt
//! bytes, which are P0/P1's adversarial corpus. The claim each entry pins is a separation of planes:
//!
//! * the structure was read to the end of its schema (`structural_read: complete`) — a name in the
//!   wrong release, a name in the wrong structure, a flag a release does not define and an opcode a
//!   release forbids are all *readable*, and a diagnostic is not a read failure;
//! * the dialect plane states what this build validates (`version_capability`), which is not
//!   `supported` for any of the illegal entries;
//! * `verification` stays `not_performed` and the header's `output_level` stays `not_evaluated`
//!   whatever the diagnostics say: this reader runs no verifier and evaluates no output level, so
//!   neither plane may be read as a pass;
//! * the modern facts' own `output_level` *is* evaluated, because that pass ran the evaluation the
//!   header plane deliberately leaves unevaluated — the two planes are different statements about
//!   different work, and the golden records both.
//!
//! # What a flag entry can and cannot say
//!
//! One of the entries carries `ACC_MODULE`'s bit in a 52 class, and it is the *release* claim: the
//! registry holds the flags each release **introduced**, not the whole flag vocabulary of any
//! structure, so the bit is reported only when a rule declares this very structure and a later
//! release than the class declares. A *location* claim is never made from a bit, and
//! `a_flag_is_answered_by_name_and_never_inferred_from_a_bit` pins why with the committed sample's
//! own bytes: every class, field and method of `RecordSample.class` sets `0x0010`, JVMS 4.1/4.5/4.6
//! define it as `ACC_FINAL` at all three, the registry's only `0x0010` rule is `ACC_FINAL` for
//! `MethodParameters`, and a bit-driven location claim would therefore report a violation in a legal
//! class. `ACC_RECORD` is the same shape twice over: no rule for the name at all, and a bit that is
//! `ACC_SYNTHETIC` wherever a record flag could be misplaced.
//!
//! # Where the bytes come from
//!
//! Two kinds of input, the two the P1 fixture index already names:
//!
//! * **committed `javac` output with its two version fields patched** (`version_patched`): every
//!   byte but two is the checked-in sample of `tests/fixtures/p4-modern/`, whose SHA-256 is in that
//!   directory's `README.md`. The replay rebuilds the bytes from the committed file, so the base
//!   identity is the repository's own and only the patch is this test's.
//! * **hand-built classes** (`jsr_class`, `invokedynamic_class`, `nest_members_in_a_method`,
//!   `acc_module_class`): shapes `javac 23.0.1` cannot be asked to emit. Each generator states the
//!   exact constant-pool indexes, attribute entries and instruction bytes it assembles, and the
//!   replay asserts the blake3 digest and length of what it produced — the convention
//!   `tests/fixtures/README.md` records for in-memory fixtures.
//!
//! # The two inspection modes
//!
//! Every replay runs twice. `forensic` is the mode that keeps a readable structure; `strict` refuses
//! a release this build does not dialect-validate, and the recorded `strict` answer is the code of
//! that refusal. The two are recorded side by side because "the structure is readable" and "the
//! release is accepted" are two different statements, and one entry (a `minor_version` the format
//! rule rejects) is refused by a **format** rule rather than by a dialect band.

use jarde::{
    AttributePlacement, Budget, ClassfileLocation, ConstantPoolTagStatus, Diagnostic,
    FlagPlacement, HeaderInspection, HeaderStructuralRead, InspectionMode, Limits, ModernFacts,
    OutputLevel, OutputLevelStatus, VerificationStatus, VersionDialectSupport, class_facts,
    feature_registry, inspect_header, modern_facts,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The committed samples the version-patched fixtures start from.
const RECORD_SAMPLE: &[u8] = include_bytes!("fixtures/p4-modern/v16/RecordSample.class");
const SEALED_SAMPLE: &[u8] = include_bytes!("fixtures/p4-modern/v17/SealedSample.class");
const CONCAT_SAMPLE: &[u8] = include_bytes!("fixtures/p4-modern/v17/ConcatSample.class");
const JAVA8_SAMPLE: &[u8] = include_bytes!("fixtures/p4-modern/v8/ConcatJava8.class");

// ---------------------------------------------------------------------------
// Fixture builder
// ---------------------------------------------------------------------------

/// A `u2` in the class-file byte order.
fn u16b(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// A `u4` in the class-file byte order.
fn u32b(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// Constant-pool builder: entries keep the 1-based indexes they are appended at, so a fixture body
/// can name the exact entry it points at.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("fixture pool fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("fixture name fits u16"),
        );
        entry.extend_from_slice(text);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.push(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    fn method_ref(&mut self, owner: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![10];
        u16b(&mut entry, owner);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    /// `CONSTANT_MethodHandle` (JVMS 4.4.8): reference kind, then the member it names.
    fn method_handle(&mut self, kind: u8, member: u16) -> u16 {
        let mut entry = vec![15, kind];
        u16b(&mut entry, member);
        self.push(entry)
    }

    /// `CONSTANT_InvokeDynamic` (JVMS 4.4.10).
    fn invoke_dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![18];
        u16b(&mut entry, bootstrap);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn declared(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).expect("fixture pool fits u16")
    }

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

/// One `attribute_info` entry (JVMS 4.7): name index, content length, content.
fn attribute_entry(name_index: u16, content: &[u8]) -> Vec<u8> {
    let mut entry = Vec::new();
    u16b(&mut entry, name_index);
    u32b(
        &mut entry,
        u32::try_from(content.len()).expect("fixture attribute fits u32"),
    );
    entry.extend_from_slice(content);
    entry
}

/// A `Code` attribute's content (JVMS 4.7.3) around one body, with no handlers and no nested
/// attributes.
fn code_content(max_stack: u16, max_locals: u16, code: &[u8]) -> Vec<u8> {
    let mut content = Vec::new();
    u16b(&mut content, max_stack);
    u16b(&mut content, max_locals);
    u32b(
        &mut content,
        u32::try_from(code.len()).expect("fixture code fits u32"),
    );
    content.extend_from_slice(code);
    u16b(&mut content, 0); // exception_table_length
    u16b(&mut content, 0); // attributes_count
    content
}

/// One `field_info` or `method_info`.
struct Member {
    access: u16,
    name: u16,
    descriptor: u16,
    attributes: Vec<Vec<u8>>,
}

/// Assembles one class file around a finished pool.
struct Builder {
    pool: Pool,
    major: u16,
    minor: u16,
    access: u16,
    this_class: u16,
    super_class: u16,
    fields: Vec<Member>,
    methods: Vec<Member>,
    attributes: Vec<Vec<u8>>,
}

impl Builder {
    fn new(name: &[u8], major: u16) -> Self {
        let mut pool = Pool::default();
        let this_name = pool.utf8(name);
        let this_class = pool.class(this_name);
        let object = pool.utf8(b"java/lang/Object");
        let super_class = pool.class(object);
        Self {
            pool,
            major,
            minor: 0,
            access: 0x0021, // ACC_PUBLIC | ACC_SUPER
            this_class,
            super_class,
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
        }
    }

    fn access(&mut self, access: u16) -> &mut Self {
        self.access = access;
        self
    }

    /// One method with the attribute entries it carries, in order.
    fn method(
        &mut self,
        access: u16,
        name: &[u8],
        descriptor: &[u8],
        attributes: Vec<(&[u8], Vec<u8>)>,
    ) -> &mut Self {
        let name_index = self.pool.utf8(name);
        let descriptor_index = self.pool.utf8(descriptor);
        let entries = attributes
            .into_iter()
            .map(|(attribute_name, content)| {
                let index = self.pool.utf8(attribute_name);
                attribute_entry(index, &content)
            })
            .collect();
        self.methods.push(Member {
            access,
            name: name_index,
            descriptor: descriptor_index,
            attributes: entries,
        });
        self
    }

    /// One class-level `attribute_info`.
    fn class_attribute(&mut self, name: &[u8], content: &[u8]) -> &mut Self {
        let index = self.pool.utf8(name);
        self.attributes.push(attribute_entry(index, content));
        self
    }

    fn finish(&self) -> Vec<u8> {
        let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
        u16b(&mut bytes, self.minor);
        u16b(&mut bytes, self.major);
        u16b(&mut bytes, self.pool.declared());
        bytes.extend_from_slice(&self.pool.bytes());
        u16b(&mut bytes, self.access);
        u16b(&mut bytes, self.this_class);
        u16b(&mut bytes, self.super_class);
        u16b(&mut bytes, 0); // interfaces
        u16b(
            &mut bytes,
            u16::try_from(self.fields.len()).expect("fields fit u16"),
        );
        for field in &self.fields {
            write_member(&mut bytes, field);
        }
        u16b(
            &mut bytes,
            u16::try_from(self.methods.len()).expect("methods fit u16"),
        );
        for method in &self.methods {
            write_member(&mut bytes, method);
        }
        u16b(
            &mut bytes,
            u16::try_from(self.attributes.len()).expect("attributes fit u16"),
        );
        for entry in &self.attributes {
            bytes.extend_from_slice(entry);
        }
        bytes
    }
}

fn write_member(bytes: &mut Vec<u8>, member: &Member) {
    u16b(bytes, member.access);
    u16b(bytes, member.name);
    u16b(bytes, member.descriptor);
    u16b(
        bytes,
        u16::try_from(member.attributes.len()).expect("member attributes fit u16"),
    );
    for entry in &member.attributes {
        bytes.extend_from_slice(entry);
    }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// One committed sample with only its two version fields patched (bytes 4..6 and 6..8 of the class
/// file), the convention P4 1.2's version tests use.
///
/// Every other byte is the committed `javac` output, so the provenance of the fixture is the
/// sample's own SHA-256 (recorded in `fixtures/p4-modern/README.md`) plus this patch, and the
/// replay rebuilds it from that file rather than from a checked-in copy of the patched bytes.
fn version_patched(base: &[u8], major: u16, minor: u16) -> Vec<u8> {
    let mut bytes = base.to_vec();
    assert_eq!(
        &bytes[..4],
        &0xcafebabe_u32.to_be_bytes(),
        "a class file starts with its magic"
    );
    bytes[4..6].copy_from_slice(&minor.to_be_bytes());
    bytes[6..8].copy_from_slice(&major.to_be_bytes());
    bytes
}

/// `jsr` (JVMS 6.5), which no class file of major 51 or above may contain (JVMS 4.9.1).
///
/// The body is one subroutine call and its handler: `jsr +4; return; astore_0; ret 0`. The
/// subroutine stores the return address in local 0 and returns with `ret`, so a legacy verifier
/// would accept it — that is what makes it a *release* violation rather than a malformed body.
fn jsr_class(major: u16) -> Vec<u8> {
    let mut builder = Builder::new(b"p/Legacy", major);
    let code = [0xa8, 0x00, 0x04, 0xb1, 0x4b, 0xa9, 0x00];
    builder.method(
        0x0009, // ACC_PUBLIC | ACC_STATIC
        b"call",
        b"()V",
        vec![(b"Code", code_content(1, 1, &code))],
    );
    builder.finish()
}

/// `invokedynamic` (JVMS 6.5) at a release that does not define the opcode.
///
/// The instruction names the `CONSTANT_InvokeDynamic` entry this generator placed, and the class
/// carries the `BootstrapMethods` table such a site must resolve against — so the entry is the
/// opcode's evidence, not a dangling reference.
fn invokedynamic_class(major: u16) -> Vec<u8> {
    let mut builder = Builder::new(b"p/Caller", major);
    let name = builder.pool.utf8(b"call");
    let descriptor = builder.pool.utf8(b"()V");
    let bootstrap_name = builder.pool.utf8(b"bootstrap");
    let bootstrap_type = builder.pool.name_and_type(bootstrap_name, descriptor);
    let fixture = builder.pool.utf8(b"p/Fixture");
    let fixture_class = builder.pool.class(fixture);
    let handle_ref = builder.pool.method_ref(fixture_class, bootstrap_type);
    // REF_invokeStatic (JVMS 4.4.8 table 5.4.3.5-A).
    let handle = builder.pool.method_handle(6, handle_ref);
    let site_type = builder.pool.name_and_type(name, descriptor);
    let site = builder.pool.invoke_dynamic(0, site_type);
    let high = u8::try_from(site >> 8).expect("fixture pool index fits u8");
    let low = u8::try_from(site & 0x00ff).expect("fixture pool index fits u8");
    let code = [0xba, high, low, 0x00, 0x00, 0x57, 0xb1]; // invokedynamic site; pop; return

    let mut bootstrap_methods = Vec::new();
    u16b(&mut bootstrap_methods, 1); // num_bootstrap_methods
    u16b(&mut bootstrap_methods, handle);
    u16b(&mut bootstrap_methods, 0); // num_bootstrap_arguments
    builder.class_attribute(b"BootstrapMethods", &bootstrap_methods);
    builder.method(
        0x0009,
        b"call",
        b"()V",
        vec![(b"Code", code_content(1, 0, &code))],
    );
    builder.finish()
}

/// `NestMembers` — a class-level attribute since major 55 (JVMS 4.7.29) — inside a `method_info`.
fn nest_members_in_a_method() -> Vec<u8> {
    let mut builder = Builder::new(b"p/Host", 61);
    let other = builder.pool.utf8(b"p/Other");
    let other_class = builder.pool.class(other);
    let mut members = Vec::new();
    u16b(&mut members, 1); // number_of_classes
    u16b(&mut members, other_class);
    builder.method(
        0x0009,
        b"call",
        b"()V",
        vec![
            (b"Code", code_content(0, 0, &[0xb1])),
            (b"NestMembers", members),
        ],
    );
    builder.finish()
}

/// `ACC_MODULE` (0x8000, JVMS 4.1) in a class file of `major`, which the registry registers from 53.
fn acc_module_class(major: u16) -> Vec<u8> {
    let mut builder = Builder::new(b"p/ModuleFlag", major);
    builder.access(0x0021 | 0x8000);
    builder.method(
        0x0009,
        b"call",
        b"()V",
        vec![(b"Code", code_content(0, 0, &[0xb1]))],
    );
    builder.finish()
}

/// `ConstantValue` — a `field_info` attribute since major 45 (JVMS 4.7.2) — inside a `method_info`
/// of a 60 class. The content is not read: the registry refuses the placement first, and the entry's
/// range is what the placement row reports.
fn constant_value_in_a_method() -> Vec<u8> {
    let mut builder = Builder::new(b"p/MethodValue", 60);
    let mut value = Vec::new();
    u16b(&mut value, 1); // constantvalue_index, in the pool entry `Builder::new` placed
    builder.method(
        0x0009,
        b"call",
        b"()V",
        vec![
            (b"Code", code_content(0, 0, &[0xb1])),
            (b"ConstantValue", value),
        ],
    );
    builder.finish()
}

/// The bytes one named fixture is built from.
fn fixture_bytes(name: &str) -> Vec<u8> {
    match name {
        "record-at-59" => version_patched(RECORD_SAMPLE, 59, 0),
        "permitted-subclasses-at-60" => version_patched(SEALED_SAMPLE, 60, 0),
        "record-at-72" => version_patched(RECORD_SAMPLE, 72, 0),
        "record-at-60-minor-7" => version_patched(RECORD_SAMPLE, 60, 7),
        "record-sample" => RECORD_SAMPLE.to_vec(),
        "sealed-sample" => SEALED_SAMPLE.to_vec(),
        "concat-sample" => CONCAT_SAMPLE.to_vec(),
        "java8-concat-sample" => JAVA8_SAMPLE.to_vec(),
        "nest-members-in-a-method" => nest_members_in_a_method(),
        "jsr-at-52" => jsr_class(52),
        "invokedynamic-at-50" => invokedynamic_class(50),
        "acc-module-at-52" => acc_module_class(52),
        "constant-value-in-a-method" => constant_value_in_a_method(),
        other => panic!("the replay list names no generator for {other}"),
    }
}

/// Where one fixture's bytes come from, recorded in the golden next to its digest.
fn fixture_source(name: &str) -> &'static str {
    match name {
        "record-at-59" | "record-at-72" | "record-at-60-minor-7" => {
            "tests/fixtures/p4-modern/v16/RecordSample.class (committed `javac 23.0.1` output, SHA-256 \
             in tests/fixtures/p4-modern/README.md) with only bytes 4..6 (minor_version) and 6..8 \
             (major_version) patched by tests/p4_golden.rs::version_patched"
        }
        "permitted-subclasses-at-60" => {
            "tests/fixtures/p4-modern/v17/SealedSample.class (committed `javac 23.0.1` output, SHA-256 \
             in tests/fixtures/p4-modern/README.md) with only bytes 4..6 (minor_version) and 6..8 \
             (major_version) patched by tests/p4_golden.rs::version_patched"
        }
        "record-sample" => {
            "tests/fixtures/p4-modern/v16/RecordSample.class, unpatched (committed `javac 23.0.1` \
             output, SHA-256 in tests/fixtures/p4-modern/README.md)"
        }
        "sealed-sample" => {
            "tests/fixtures/p4-modern/v17/SealedSample.class, unpatched (committed `javac 23.0.1` \
             output, SHA-256 in tests/fixtures/p4-modern/README.md)"
        }
        "concat-sample" => {
            "tests/fixtures/p4-modern/v17/ConcatSample.class, unpatched (committed `javac 23.0.1` \
             output, SHA-256 in tests/fixtures/p4-modern/README.md)"
        }
        "java8-concat-sample" => {
            "tests/fixtures/p4-modern/v8/ConcatJava8.class, unpatched (committed `javac 23.0.1` \
             output, SHA-256 in tests/fixtures/p4-modern/README.md)"
        }
        "nest-members-in-a-method" => {
            "hand-built in tests/p4_golden.rs::nest_members_in_a_method: a 61 class whose only \
             method carries a `Code` entry and a `NestMembers` entry, the attribute JVMS 4.7.29 \
             declares for a class file only"
        }
        "jsr-at-52" => {
            "hand-built in tests/p4_golden.rs::jsr_class(52): `jsr +4; return; astore_0; ret 0`, one \
             subroutine call site, at a major the opcode is forbidden from"
        }
        "invokedynamic-at-50" => {
            "hand-built in tests/p4_golden.rs::invokedynamic_class(50): an `invokedynamic` \
             instruction naming the `CONSTANT_InvokeDynamic` entry this generator placed, then \
             `pop`, `return`, with a `BootstrapMethods` table whose one entry is a \
             `CONSTANT_MethodHandle` for `p/Fixture.bootstrap:()V`"
        }
        "acc-module-at-52" => {
            "hand-built in tests/p4_golden.rs::acc_module_class(52): ACC_PUBLIC | ACC_SUPER | 0x8000 \
             (the `ACC_MODULE` bit JVMS 4.1 assigns from major 53) on a two-releases-earlier class"
        }
        "constant-value-in-a-method" => {
            "hand-built in tests/p4_golden.rs::constant_value_in_a_method: a 60 class whose only \
             method carries a `Code` entry and a `ConstantValue` entry, the attribute JVMS 4.7.2 \
             declares for a field_info only"
        }
        other => panic!("the replay list names no source for {other}"),
    }
}

// ---------------------------------------------------------------------------
// The fixed replay list
// ---------------------------------------------------------------------------

/// The three golden files, in list order.
const GOLDEN_FILES: &[&str] = &[
    "illegal-modern.json",
    "version-boundaries.json",
    "legal-modern.json",
];

struct Replay {
    file: &'static str,
    name: &'static str,
    fixture: &'static str,
    note: &'static str,
}

/// The fixed list this slice replays, in file order.
fn replay_list() -> Vec<Replay> {
    vec![
        // ------------------------------------------------------------ illegal structures
        Replay {
            file: "illegal-modern.json",
            name: "record-in-a-59-class",
            fixture: "record-at-59",
            note: "the `Record` attribute is registered from major 60 (JVMS 4.7.30); this class declares \
               major 59, so the entry is misplaced by release and the components are not read",
        },
        Replay {
            file: "illegal-modern.json",
            name: "permitted-subclasses-in-a-60-class",
            fixture: "permitted-subclasses-at-60",
            note: "`PermittedSubclasses` is registered from major 61 (JVMS 4.7.31); a 60 class that \
               carries it is refused at 60 and read at 61",
        },
        Replay {
            file: "illegal-modern.json",
            name: "nest-members-in-a-method",
            fixture: "nest-members-in-a-method",
            note: "`NestMembers` is a class-level attribute (JVMS 4.7.29); in a `method_info` of a 61 \
               class the release is right and the location is wrong",
        },
        Replay {
            file: "illegal-modern.json",
            name: "constant-value-in-a-method",
            fixture: "constant-value-in-a-method",
            note: "`ConstantValue` is a `field_info` attribute (JVMS 4.7.2); in a `method_info` of a 60 \
               class the release is right and the location is wrong",
        },
        Replay {
            file: "illegal-modern.json",
            name: "acc-module-in-a-52-class",
            fixture: "acc-module-at-52",
            note: "the `ACC_MODULE` bit (0x8000) is assigned in a class file from major 53 (JVMS 4.1); a \
               52 class that sets it sets a flag of a later release — the one flag claim the \
               registry's introduced-only table can carry about a set bit",
        },
        Replay {
            file: "illegal-modern.json",
            name: "jsr-in-a-52-class",
            fixture: "jsr-at-52",
            note: "`jsr` is forbidden from major 51 on (JVMS 4.9.1); the body is one call site and its \
               handler, so the class is a legacy shape at a release that forbids it",
        },
        Replay {
            file: "illegal-modern.json",
            name: "invokedynamic-in-a-50-class",
            fixture: "invokedynamic-at-50",
            note: "`invokedynamic` is registered from major 51 (JVMS 6.5); the site names a real \
               `CONSTANT_InvokeDynamic` entry, so the violation is the opcode and not a dangling \
               constant-pool index",
        },
        // ------------------------------------------------------------ version boundaries
        Replay {
            file: "version-boundaries.json",
            name: "record-in-an-unregistered-release",
            fixture: "record-at-72",
            note: "major 72 is above every release the registry holds: the structure stays readable, the \
               registry makes no claim about the `Record` entry, and the dialect plane says the \
               release is unknown rather than refused",
        },
        Replay {
            file: "version-boundaries.json",
            name: "minor-form-invalid-at-60",
            fixture: "record-at-60-minor-7",
            note: "the format rule of JVMS 4.1 requires minor 0 or 65535 for major >= 56: a *format* \
               refusal, not a dialect band, so strict and forensic differ for a reason the \
               unregistered release does not share",
        },
        // ------------------------------------------------------------ legal controls
        Replay {
            file: "legal-modern.json",
            name: "legal-record-at-60",
            fixture: "record-sample",
            note: "the same class as the 59 entry, unpatched: the placement row is legal, the components \
               are read, and the release's band is still a structural probe rather than a validated \
               dialect",
        },
        Replay {
            file: "legal-modern.json",
            name: "legal-sealed-at-61",
            fixture: "sealed-sample",
            note: "`PermittedSubclasses` at the release that introduces it, with two permitted names",
        },
        Replay {
            file: "legal-modern.json",
            name: "legal-concat-at-61",
            fixture: "concat-sample",
            note: "two `StringConcatFactory` sites at 61: the output-level plane answers the Java 8 \
               question with a conflict and the sites themselves stay published",
        },
        Replay {
            file: "legal-modern.json",
            name: "legal-java8-concat-at-52",
            fixture: "java8-concat-sample",
            note: "a 52 class whose concat is a `StringBuilder` chain: nothing here is a modern fact, so \
               the Java 8 question is representable and no conflict is reported",
        },
    ]
}

// ---------------------------------------------------------------------------
// Replaying
// ---------------------------------------------------------------------------

/// One replay's answer from the public reader entries.
struct Observed {
    inspection: HeaderInspection,
    modern: ModernFacts,
}

/// Runs one fixture through the reader entries a caller has: the header plane, the declaration
/// structure and the fact plane.
fn observe(bytes: &[u8], mode: InspectionMode) -> Result<Observed, String> {
    let mut budget = Budget::new(limits());
    let inspection = inspect_header(bytes, &mut budget, mode).map_err(|error| refusal(&error))?;
    let facts =
        class_facts(bytes, &mut budget).expect("the fixture's declaration structure is readable");
    let modern = modern_facts(bytes, &facts, OutputLevel::Java8, &mut budget)
        .expect("the fixture's modern facts are readable");
    Ok(Observed { inspection, modern })
}

/// The code of a refused read, as a golden records it.
fn refusal(error: &jarde::Error) -> String {
    let value = serde_json::to_value(error).expect("an error serializes");
    match value.get("code").and_then(Value::as_str) {
        Some(code) => code.to_owned(),
        None => panic!("a refusal without a code: {error}"),
    }
}

/// The header planes one inspection publishes, exactly as a golden records them.
fn header_projection(inspection: &HeaderInspection) -> Value {
    json!({
        "structural_read": inspection.structural_read,
        "version_capability": inspection.version_capability,
        "verification": inspection.verification,
        "output_level": inspection.output_level,
        "diagnostics": diagnostic_rows(&inspection.diagnostics),
    })
}

/// The planes and facts one modern-facts read publishes.
fn modern_projection(facts: &ModernFacts) -> Value {
    let attributes = facts
        .attributes
        .iter()
        .map(|row| {
            json!({
                "name": String::from_utf8_lossy(&row.name.0),
                "placement": placement_name(&row.placement),
                "read": row.read,
            })
        })
        .collect::<Vec<_>>();
    let conflict_features = match &facts.output_level {
        OutputLevelStatus::Conflict { conflicts, .. } => {
            conflicts.iter().map(|fact| json!(fact.feature)).collect()
        }
        OutputLevelStatus::NotEvaluated | OutputLevelStatus::Representable { .. } => Vec::new(),
    };
    json!({
        "attributes": attributes,
        "record_read": facts.record.is_some(),
        "permitted_subclasses_read": facts.permitted_subclasses.is_some(),
        "concat_sites": facts.concat.len(),
        "dynamic_tag": tag_status_name(facts.condy.dynamic_tag),
        "output_level": output_level_name(&facts.output_level),
        "conflict_features": conflict_features,
        "diagnostics": diagnostic_rows(&facts.diagnostics),
    })
}

fn diagnostic_rows(diagnostics: &[Diagnostic]) -> Value {
    Value::Array(
        diagnostics
            .iter()
            .map(|diagnostic| {
                json!({
                    "code": diagnostic.code,
                    "severity": diagnostic.severity,
                    "message": diagnostic.message,
                })
            })
            .collect(),
    )
}

fn placement_name(placement: &AttributePlacement) -> &'static str {
    match placement {
        AttributePlacement::Legal { .. } => "legal",
        AttributePlacement::VersionNotApplicable { .. } => "version_not_applicable",
        AttributePlacement::LocationNotApplicable { .. } => "location_not_applicable",
        AttributePlacement::NotRegistered => "not_registered",
        AttributePlacement::UnregisteredRelease => "unregistered_release",
    }
}

fn tag_status_name(status: ConstantPoolTagStatus) -> &'static str {
    match status {
        ConstantPoolTagStatus::Registered { .. } => "registered",
        ConstantPoolTagStatus::VersionNotApplicable { .. } => "version_not_applicable",
        ConstantPoolTagStatus::Unregistered => "unregistered",
        ConstantPoolTagStatus::UnregisteredRelease => "unregistered_release",
    }
}

fn output_level_name(status: &OutputLevelStatus) -> &'static str {
    match status {
        OutputLevelStatus::NotEvaluated => "not_evaluated",
        OutputLevelStatus::Representable { .. } => "representable",
        OutputLevelStatus::Conflict { .. } => "conflict",
    }
}

fn limits() -> Limits {
    Limits {
        input_bytes: u64::MAX,
        archive_entries: u64::MAX,
        entry_bytes: u64::MAX,
        read_bytes: u64::MAX,
        class_bytes: u64::MAX,
        attribute_bytes: u64::MAX,
        code_bytes: u64::MAX,
        result_items: u64::MAX,
        output_bytes: u64::MAX,
        class_headers: u64::MAX,
        method_bodies: u64::MAX,
        ir_items: u64::MAX,
        ir_edges: u64::MAX,
        analysis_steps: u64::MAX,
        normalization_clones: u64::MAX,
        nested_depth: u64::MAX,
        dependency_depth: u64::MAX,
        elapsed_millis: u64::MAX,
    }
}

// ---------------------------------------------------------------------------
// The golden files
// ---------------------------------------------------------------------------

fn golden_path(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/p4-golden")
        .join(file)
}

fn golden(file: &str) -> Value {
    let path = golden_path(file);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is unreadable: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()))
}

fn golden_replays(golden: &Value) -> &[Value] {
    golden["replays"]
        .as_array()
        .expect("a golden holds a replay array")
}

fn golden_replay<'a>(golden: &'a Value, name: &str) -> &'a Value {
    golden_replays(golden)
        .iter()
        .find(|replay| replay["name"] == name)
        .unwrap_or_else(|| panic!("the golden holds no replay named {name}"))
}

/// The file's list and the test's fixed list are one list: a deleted, renamed or added replay fails
/// here even though every remaining replay would still replay correctly.
fn assert_replay_list(file: &str, replays: &[Replay]) {
    let golden = golden(file);
    assert_eq!(
        golden["file"].as_str(),
        Some(file),
        "{file}: the golden names its file"
    );
    let recorded = golden_replays(&golden)
        .iter()
        .map(|replay| replay["name"].as_str().expect("a replay name"))
        .collect::<Vec<_>>();
    let listed = replays
        .iter()
        .filter(|replay| replay.file == file)
        .map(|replay| replay.name)
        .collect::<Vec<_>>();
    assert_eq!(
        recorded, listed,
        "{file}: the golden's replay list is this test's list"
    );
}

/// One replay: the fixture rebuilt from its generator, and both planes compared with the record.
fn assert_replay(replay: &Replay, recorded: &Value) {
    let bytes = fixture_bytes(replay.fixture);
    assert_eq!(
        recorded["fixture"].as_str(),
        Some(replay.fixture),
        "{}: the golden names its fixture",
        replay.name
    );
    assert_eq!(
        recorded["source"].as_str(),
        Some(fixture_source(replay.fixture)),
        "{}: the fixture's provenance is recorded",
        replay.name
    );
    assert_eq!(
        recorded["bytes"].as_u64(),
        Some(u64::try_from(bytes.len()).expect("a fixture fits u64")),
        "{}: the fixture's length",
        replay.name
    );
    assert_eq!(
        recorded["blake3"].as_str(),
        Some(blake3::hash(&bytes).to_hex().to_string().as_str()),
        "{}: the fixture's blake3 digest",
        replay.name
    );
    assert_eq!(
        recorded["note"].as_str().map(str::trim),
        Some(replay.note.trim()),
        "{}: the recorded note is this test's own",
        replay.name
    );
    // The file an entry lives in *is* its classification: `illegal-modern.json` holds structures
    // that are illegal for their release, `version-boundaries.json` the releases this build does not
    // accept, and `legal-modern.json` the controls. The plane invariants are applied to every entry
    // whose recorded flag is set, so an entry cannot be moved out of them by editing one side alone.
    assert_eq!(
        recorded["expect"]["illegal"].as_bool(),
        Some(replay.file != "legal-modern.json"),
        "{}: the entry's illegal flag follows the file that holds it",
        replay.name
    );

    let forensic = observe(&bytes, InspectionMode::Forensic)
        .expect("a forensic read keeps a readable structure");
    assert_eq!(
        header_projection(&forensic.inspection),
        recorded["expect"]["header"],
        "{}: the header planes",
        replay.name
    );
    assert_eq!(
        modern_projection(&forensic.modern),
        recorded["expect"]["modern"],
        "{}: the fact planes",
        replay.name
    );

    match observe(&bytes, InspectionMode::Strict) {
        Ok(strict) => {
            assert_eq!(
                recorded["expect"]["strict"]["read"], "complete",
                "{}: strict accepted this class, and only that is recorded",
                replay.name
            );
            assert_eq!(
                header_projection(&strict.inspection),
                header_projection(&forensic.inspection),
                "{}: an accepted release reads the same planes in both modes",
                replay.name
            );
        }
        Err(code) => {
            assert_eq!(
                Some(code.as_str()),
                recorded["expect"]["strict"]["error"].as_str(),
                "{}: the strict refusal's code",
                replay.name
            );
        }
    }
}

/// Every replay the file holds was replayed by this run.
fn assert_golden_file(file: &str, replays: &[Replay]) {
    let golden = golden(file);
    for replay in replays.iter().filter(|replay| replay.file == file) {
        assert_replay(replay, golden_replay(&golden, replay.name));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn the_fixed_replay_list_is_each_goldens_own_list() {
    let replays = replay_list();
    for file in GOLDEN_FILES {
        assert_replay_list(file, &replays);
    }
    assert_eq!(
        replays.len(),
        GOLDEN_FILES
            .iter()
            .map(|file| golden(file)["replays"].as_array().expect("a list").len())
            .sum::<usize>(),
        "every recorded replay is named by the fixed list"
    );
}

#[test]
fn every_entry_replays_its_fixture_digest_and_planes() {
    let replays = replay_list();
    for file in GOLDEN_FILES {
        assert_golden_file(file, &replays);
    }
}

/// The plane separation this slice exists for: an illegal class file is still **read**, and no plane
/// is read as another. A diagnostic is not a read failure, a complete read is not a dialect answer,
/// and neither is a verification — while a release this build *does* validate is no promise that
/// every release rule holds in it.
#[test]
fn an_illegal_class_file_is_read_and_its_planes_stay_separate() {
    let replays = replay_list();
    let mut illegal = 0usize;
    let mut validated_releases = 0usize;
    let mut probed_releases = 0usize;
    for file in GOLDEN_FILES {
        let golden = golden(file);
        for replay in replays.iter().filter(|replay| replay.file == *file) {
            let recorded = golden_replay(&golden, replay.name);
            if recorded["expect"]["illegal"].as_bool() != Some(true) {
                continue;
            }
            illegal += 1;
            let bytes = fixture_bytes(replay.fixture);
            let observed = observe(&bytes, InspectionMode::Forensic).unwrap_or_else(|code| {
                panic!("{}: a forensic read was refused ({code})", replay.name)
            });

            // The golden says this entry's structure is read, and the typed report agrees.
            assert_eq!(
                recorded["expect"]["header"]["structural_read"], "complete",
                "{}: the entry is about a readable structure",
                replay.name
            );
            assert_eq!(
                observed.inspection.structural_read,
                HeaderStructuralRead::Complete,
                "{}: the structure is read to the end of its schema",
                replay.name
            );
            // No verifier ran here, and no output level was evaluated here.
            assert_eq!(
                observed.inspection.verification,
                VerificationStatus::NotPerformed,
                "{}: verification stays not_performed",
                replay.name
            );
            assert_eq!(
                observed.inspection.output_level,
                OutputLevelStatus::NotEvaluated,
                "{}: the header plane evaluated no output level",
                replay.name
            );
            // The dialect plane is its own answer, recorded and compared, and both directions of the
            // separation are carried by this set: a validated release can still break a release rule,
            // and a release this build only probes is still read.
            assert_eq!(
                json!(
                    observed
                        .inspection
                        .version_capability
                        .version_dialect_support
                ),
                recorded["expect"]["header"]["version_capability"]["version_dialect_support"],
                "{}: the dialect plane is the recorded one",
                replay.name
            );
            match observed
                .inspection
                .version_capability
                .version_dialect_support
            {
                VersionDialectSupport::Supported => validated_releases += 1,
                VersionDialectSupport::StructuralProbeOnly
                | VersionDialectSupport::UnsupportedPreview
                | VersionDialectSupport::FutureRelease => probed_releases += 1,
            }
            // The violation really was reported, by one of the two planes the reader publishes.
            let codes = observed
                .inspection
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .chain(
                    observed
                        .modern
                        .diagnostics
                        .iter()
                        .map(|diagnostic| diagnostic.code.as_str()),
                )
                .collect::<Vec<_>>();
            assert!(
                !codes.is_empty(),
                "{}: an illegal fixture states its violation",
                replay.name
            );
            for code in &codes {
                assert!(
                    code.starts_with("classfile_"),
                    "{}: {code} is a reader code, and the reader invents no dialect verdict",
                    replay.name
                );
            }
        }
    }
    assert!(
        illegal >= 9,
        "the illegal entries are the slice's subject ({illegal} replayed)"
    );
    assert!(
        validated_releases >= 1,
        "a release this build validates can still carry a release-rule violation"
    );
    assert!(
        probed_releases >= 1,
        "and a release it does not validate is still read completely"
    );
}

/// The placement row one class's modern facts publish for one attribute name.
fn attribute_row<'a>(facts: &'a ModernFacts, name: &str) -> &'a jarde::ModernAttributePlacement {
    facts
        .attributes
        .iter()
        .find(|row| row.name.0.as_slice() == name.as_bytes())
        .unwrap_or_else(|| panic!("the {name} entry is reported"))
}

/// A refused placement is still *reported*: the entry is a row with the registry's answer, and only
/// a legal placement is read into a typed fact.
#[test]
fn a_refused_placement_is_a_row_and_not_a_fact() {
    let refused = observe(&fixture_bytes("record-at-59"), InspectionMode::Forensic)
        .expect("the 59 class is readable")
        .modern;
    let accepted = observe(&fixture_bytes("record-sample"), InspectionMode::Forensic)
        .expect("the 60 class is readable")
        .modern;

    assert!(
        refused.record.is_none() && accepted.record.is_some(),
        "only the legal placement is read into components"
    );
    assert!(
        matches!(
            attribute_row(&refused, "Record").placement,
            AttributePlacement::VersionNotApplicable { .. }
        ),
        "the 59 class's row states why it was not read"
    );
    assert!(
        matches!(
            attribute_row(&accepted, "Record").placement,
            AttributePlacement::Legal { .. }
        ),
        "the 60 class's row is the legal placement"
    );
    assert!(
        attribute_row(&accepted, "Record").read && !attribute_row(&refused, "Record").read,
        "the read flag follows the placement"
    );
}

/// The boundary the flag wiring draws, pinned from the registry's own answers and from the bytes of
/// the committed sample: a flag a caller **names** is answerable, a bit is not.
#[test]
fn a_flag_is_answered_by_name_and_never_inferred_from_a_bit() {
    let registry = feature_registry();
    assert!(matches!(
        registry.flag_placement("ACC_MODULE", ClassfileLocation::ClassFile, 52),
        FlagPlacement::VersionNotApplicable { .. }
    ));
    assert!(matches!(
        registry.flag_placement("ACC_MODULE", ClassfileLocation::MethodInfo, 60),
        FlagPlacement::LocationNotApplicable { .. }
    ));
    assert!(matches!(
        registry.flag_placement("ACC_MODULE", ClassfileLocation::ClassFile, 60),
        FlagPlacement::Legal { .. }
    ));
    // The record flag is a name the registry makes no claim about, and this pass invents none: its
    // bit is `ACC_SYNTHETIC` wherever a record flag could be misplaced.
    assert!(matches!(
        registry.flag_placement("ACC_RECORD", ClassfileLocation::ClassFile, 60),
        FlagPlacement::NotRegistered
    ));

    // The bit route is the one this pass does not take. The committed `javac 23.0.1` output sets
    // 0x0010 in its class word, in its fields and in its methods — JVMS 4.1, 4.5 and 4.6 define that
    // bit as `ACC_FINAL` at all three, and the registry's only 0x0010 rule is `ACC_FINAL` for
    // `MethodParameters` — so a bit-driven location claim would report a violation in this legal
    // class. The pass states none.
    let bytes = fixture_bytes("record-sample");
    let mut budget = Budget::new(limits());
    let facts = class_facts(&bytes, &mut budget).expect("the sample is readable");
    assert_ne!(
        facts.access_flags & 0x0010,
        0,
        "the sample's own class word is ACC_FINAL"
    );
    assert!(
        facts
            .fields
            .iter()
            .any(|field| field.access_flags & 0x0010 != 0)
            && facts
                .methods
                .iter()
                .any(|method| method.access_flags & 0x0010 != 0),
        "and so are its fields and its methods"
    );
    let observed = observe(&bytes, InspectionMode::Forensic).expect("the sample is readable");
    assert!(
        observed.modern.diagnostics.is_empty(),
        "a legal class gets no flag verdict: {:?}",
        observed.modern.diagnostics
    );
}

/// The legal controls state no release violation, and their facts are read.
#[test]
fn the_legal_controls_state_no_violation() {
    for replay in replay_list()
        .iter()
        .filter(|replay| replay.file == "legal-modern.json")
    {
        let bytes = fixture_bytes(replay.fixture);
        let observed = observe(&bytes, InspectionMode::Forensic)
            .unwrap_or_else(|code| panic!("{}: readable ({code})", replay.name));
        assert!(
            observed.modern.diagnostics.is_empty(),
            "{}: a legal class states no placement violation",
            replay.name
        );
        for diagnostic in &observed.inspection.diagnostics {
            assert!(
                !diagnostic.code.starts_with("classfile_attribute_")
                    && !diagnostic.code.starts_with("classfile_flag_")
                    && !diagnostic.code.starts_with("classfile_opcode_"),
                "{}: {} is not a release-rule violation of a legal class",
                replay.name,
                diagnostic.code
            );
        }
        assert_eq!(
            observed.inspection.verification,
            VerificationStatus::NotPerformed,
            "{}: the legal controls run no verifier either",
            replay.name
        );
    }
}
