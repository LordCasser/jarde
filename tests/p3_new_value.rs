//! P3 2c.26/2c.27: a proved construction chain, written where its one consumer runs.
//!
//! `javac --release 8` builds every construction the same way — `new T; dup; args…; invokespecial
//! T.<init>` — and the copy the `dup` made is the constructor's receiver while the other copy is the
//! constructed value. `new@1` already verifies that chain and already writes the `new` expression
//! where a **store** consumes it (`Object o = new Object()` is `local1 = new java.lang.Object()`).
//! What used to be refused is every other single consumer of the leftover: a field write (P3 2c.26),
//! and a `return` or a call argument (P3 2c.27). The refusal was not in the rendering — the `new`
//! expression is written from the site the same way for every consumer — but in the site plan, which
//! asks whether the instance is written into its consumer's text at all, and did not count a field
//! access among the instructions that write the values they read.
//!
//! The texts below pin that on the class bytes committed in `tests/fixtures/p3-new-value/` (see its
//! `README.md` for the command, the version and the digest). The three field writes are the shape the
//! change is about; `localNew` is the control that keeps the store consumer's spelling untouched, and
//! `directNew`/`take`/`argNew` are the return and argument positions of 2c.27 — which were already
//! presented and must stay exactly what they were. The last case is the boundary neither task moves:
//! a leftover that more than one instruction reads is still quoted, because one `new` expression in
//! one place cannot spell two consumers.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 606 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-new-value/v8/Built.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one committed sample answers one definition, got {} candidate(s)",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one committed sample answers one definition, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

/// One member of the sample, by its own raw name.
fn member_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's own text in the assembled source.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &member_of(report, name).text
}

/// One member's own recovery report: the planes and the decisions the text was written under.
fn recovered_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member_of(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` has no recovered body: {other:?}"),
    }
}

/// What every member of this sample is: a presented body made of statements, with no bytecode quoted
/// anywhere in it. A quote is exactly what a `new` expression no statement could hold looks like, so
/// its absence is the shape this change is about.
fn assert_presented(report: &ClassSourceReport, name: &str) -> String {
    let text = text_of(report, name).to_string();
    assert!(
        !text.contains("@bytecode"),
        "`{name}` presents its body instead of quoting bytecode:\n{text}"
    );
    assert!(
        !text.contains("Duplicate") && !text.contains("no expression this subset writes"),
        "`{name}` quotes no copy it could not write:\n{text}"
    );
    assert_eq!(
        recovered_of(report, name).content,
        RecoveryContent::ContainsStatements,
        "`{name}` is a body of statements:\n{text}"
    );
    text
}

/// The sample with one same-width patch applied, so every BCI stands: `needle` is replaced by
/// `replacement` at its one occurrence.
fn patched(bytes: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    assert_eq!(needle.len(), replacement.len(), "a patch keeps every BCI");
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0usize;
    let mut sites = 0usize;
    while at < bytes.len() {
        if bytes[at..].starts_with(needle) {
            out.extend_from_slice(replacement);
            at += needle.len();
            sites += 1;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    assert_eq!(
        sites, 1,
        "the sample holds the sequence this case patches once"
    );
    out
}

#[test]
fn a_field_write_takes_the_construction_as_its_value() {
    // `set()V` is `aload_0; new; dup; invokespecial Object.<init>; putfield a; return`: the value the
    // `putfield` stores is the `dup`'s leftover, and the statement is written where the `putfield`
    // runs — no local is invented for it and the construction is not written as a second statement
    // beside the assignment.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Built");
    let set = assert_presented(&report, "set");
    assert_eq!(
        set,
        "    void set() {\n        // @method set()V\n        // @declaration an instance method of `Built`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        this.a = new java.lang.Object();\n        return;\n    }\n",
        "the field write is the construction's one consumer"
    );
    // The constructor's own body is the same write after its prologue: `this.a = new Object()` is an
    // instance initializer of a constructor, and the initializer's statement lands where its
    // `putfield` runs — after `super()`, which is where the bytecode has it.
    let constructor = assert_presented(&report, "<init>");
    assert_eq!(
        constructor,
        "    Built() {\n        // @method <init>()V\n        // @declaration a constructor of `Built`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        this.a = new java.lang.Object();\n        return;\n    }\n",
        "the initializer is written where its own `putfield` runs"
    );
    // `setStatic()V` is `new; dup; invokespecial; putstatic s; return`: a static write's receiver is
    // the owner type, which is the form `field@1` already writes for a static read and write.
    let set_static = assert_presented(&report, "setStatic");
    assert_eq!(
        set_static,
        "    static void setStatic() {\n        // @method setStatic()V\n        // @declaration a static method of `Built`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        Built.s = new java.lang.Object();\n        return;\n    }\n",
        "a static write names the owner its own pool entry names"
    );
}

#[test]
fn the_store_consumer_keeps_the_spelling_it_had() {
    // `localNew()` is `new; dup; invokespecial; astore_1; aload_1; areturn`: a store is the consumer
    // this rule always wrote, and neither this change nor the argument/return half may move it. The
    // declaration, the name and the return of the name are the whole of the member.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Built");
    let local_new = assert_presented(&report, "localNew");
    assert_eq!(
        local_new,
        "    java.lang.Object localNew() {\n        // @method localNew()Ljava/lang/Object;\n        // @declaration an instance method of `Built`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.Object local1 = new java.lang.Object();\n        return local1;\n    }\n",
        "the store consumer's text is untouched"
    );
    assert!(
        local_new.matches("new java.lang.Object()").count() == 1,
        "the construction is written once, into the declaration:\n{local_new}"
    );
}

#[test]
fn a_return_and_an_argument_write_the_construction_in_place() {
    // `directNew()` is `new; dup; invokespecial; areturn`: the `areturn` reads the leftover, so the
    // `new` expression *is* the returned value — no local, no second statement.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Built");
    let direct = assert_presented(&report, "directNew");
    assert_eq!(
        direct,
        "    java.lang.Object directNew() {\n        // @method directNew()Ljava/lang/Object;\n        // @declaration an instance method of `Built`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        return new java.lang.Object();\n    }\n",
        "the return writes the construction it read"
    );
    // `argNew()` is `aload_0; new; dup; invokespecial; invokevirtual take; areturn`: the leftover is
    // an **argument** of the call, written nested in the argument list exactly like any other
    // argument expression — the call is not split and no local holds the instance.
    let arg = assert_presented(&report, "argNew");
    assert_eq!(
        arg,
        "    java.lang.String argNew() {\n        // @method argNew()Ljava/lang/String;\n        // @declaration an instance method of `Built`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        return this.take(new java.lang.Object());\n    }\n",
        "the construction is the call's argument"
    );
    // `take` is the callee the argument reaches: its own body is untouched by this change, and it is
    // the member whose text shows that no `String.valueOf` was folded into the construction.
    let take = assert_presented(&report, "take");
    assert_eq!(
        take,
        "    java.lang.String take(java.lang.Object arg1) {\n        // @method take(Ljava/lang/Object;)Ljava/lang/String;\n        // @declaration an instance method of `Built`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        return java.lang.String.valueOf(arg1);\n    }\n",
        "the callee keeps its own text"
    );
    assert!(
        !arg.contains("valueOf"),
        "no call is folded into the argument:\n{arg}"
    );
}

#[test]
fn a_leftover_two_instructions_read_keeps_its_refusal() {
    // The boundary: `localNew()`'s leftover read by **two** instructions instead of one. The patch
    // keeps the body the same width and the stack shape verifiable — `astore_1; aload_1` becomes
    // `dup; pop`, so the instance is copied once more and one copy is discarded — and every BCI of
    // the sample stands. Two consumers of one constructed instance have no single Java spelling, so
    // the whole construction keeps the refusal it has always had rather than writing the `new`
    // expression twice: the member's text quotes the bytes and presents no `new` at all.
    let sample = open(&patched(SAMPLE, &[0x4c, 0x2b], &[0x59, 0x57]));
    let report = class_source_of(&sample, "Built");
    let local_new = text_of(&report, "localNew");
    assert!(
        !local_new.contains("new java.lang.Object()"),
        "a leftover two instructions read is not written as a construction:\n{local_new}"
    );
    assert!(
        local_new.contains("@bytecode 7"),
        "the copy the second reader made is named by a quote:\n{local_new}"
    );
    assert!(
        local_new.contains("the value at BCI 9 comes from an Duplicate at BCI 7"),
        "the second reader is quoted as the copy it read:\n{local_new}"
    );
    assert_eq!(
        recovered_of(&report, "localNew").content,
        RecoveryContent::ExplanationOnly,
        "a refused construction is explained, not written:\n{local_new}"
    );
    // The patch is about one member's own bytes: every other consumer of this class is still written.
    assert_presented(&report, "set");
    assert_presented(&report, "setStatic");
    assert_presented(&report, "directNew");
}
