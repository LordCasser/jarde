//! `recover-array-element-field-receiver`: the field a **subscript's element** reads.
//!
//! `RG`'s static lookup table is the critical anchor of the array-element-field-receiver patrol
//! (`openspec/evidence/java-syntax-2026-10-05/array-element-field-receiver-patrol/`):
//!
//! ```text
//! static { Item[] all = new Item[]{ A, B, C }; for (Item c : all) { BY_LABEL.put(c.label, c); } }
//! ```
//!
//! Every receiver that is an `aaload` result lost its field access — `the field access at BCI 91 is
//! not one this run proved names the member its own receiver's type declares` — because `aaload`
//! states no type of its own, and the field identity proof compared the receiver's **frame-stated**
//! name with the member's owner. What was lost is not merely text: the stripped text still compiled
//! and read `null/null/null` where the class reads two `Item`s and `null`, so the lookup table
//! stayed silently empty (the compilable-wrong this change closes).
//!
//! The array a value was read out of states the element's type — the frames name every array a
//! parameter, a field read, a call, an `anewarray` or a `multianewarray` states, and the chain of a
//! local holding one — which is the reading this layer already declared a local from
//! (`Item local1 = arg0[0];`). What this change adds is that the same reading reaches the field
//! identity proof's receiver comparison, handed in by the caller, and that nothing is guessed.
//!
//! What the presented texts pin, on **both** compiler legs (javac 23.0.1 `--release 8` and real
//! javac 8, the same sources both times):
//!
//! * `RG`'s `<clinit>` writes the whole loop body — `RG.BY_LABEL.put((java.lang.Object)
//!   local4.label, (java.lang.Object) local4);` — with BCI 91 an anchor of the statement and no
//!   `@bytecode` quote left in the class at all;
//! * the element read is a receiver **directly** (`xs[0].label`) and **through a local holding it**
//!   (`Item y = xs[0]; y.label`), for the current class's own array (`RL.selfElem`) and for a
//!   companion class's array (`RL.innerElem`) alike, and `RH`'s accumulating for-each body
//!   (whose value is `5`) recovers with `RH.first`'s single read;
//! * an enum's lookup table loop body recovers too (`EM.<clinit>`), though its **constants** render
//!   as blank declarations: the class's own `<clinit>` and `values()` are the pooled `$VALUES`
//!   shape, a rendering debt of the enum constant pool this change does not touch, so `EM` is
//!   pinned and never compiled;
//! * the shapes that already recovered stay **byte-identical**: a directly stated receiver
//!   (`RJ.direct`/`RJ.viaLocal`), an element **method call** (`RM.loopCall`/`RM.elemCall`) and a
//!   call's result (`RO.viaCall`/`RO.viaCallLocal`) render exactly as they did before;
//! * an element field **write** is presented by the same proof (`RN`): the receiver comparison is
//!   one comparison for reads and writes, so the write side moves with the read side;
//! * an array no fact states — a component type reached only through a merged local (`RP.merged`'s
//!   conditional, `RP.branchy`'s two branches) — keeps the field identity refusal verbatim: this
//!   reading follows a single chain of facts and does not guess a component type.
//!
//! The ignored replay (`cargo test -- --ignored`) strips the presentation the way the patrol's own
//! `verify-wrong-RG.java` was made — comment lines dropped, the merged companion's `$` spelling
//! rewritten to the nested simple name, the companion's own presentation nested in where the text
//! does not carry it — compiles every stripped text with the installed `javac --release 8` and runs
//! it with `-Xverify:all`: `RG` answers `of("beta")`/`of("alpha")` non-null and `of("?")` null
//! (where the pre-change text read `null/null/null`), and every other fixture answers exactly what
//! its own class files answer.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

// -------------------------------------------------------------------------------------------
// The fixtures: the patrol's own shapes, compiled by both javac legs.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names, the class files it produced, and the `EM` text its
/// constant pool's own shape produces (the two legs differ there, and nowhere else). The companions
/// ride in the same container as the class they belong to, which is what makes them members of its
/// family.
struct Leg {
    label: &'static str,
    files: &'static [(&'static str, &'static [u8])],
    em_clinit: &'static str,
}

impl Leg {
    /// One fixture family's container: its own class file and its companions'.
    fn fixture(&self, class: &str) -> Vec<u8> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let entries: Vec<(&[u8], &[u8])> = self
            .files
            .iter()
            .filter(|(name, _)| *name == own || name.starts_with(&nested))
            .map(|(name, bytes)| (name.as_bytes(), *bytes))
            .collect();
        assert_eq!(
            entries.first().map(|(name, _)| *name),
            Some(own.as_bytes()),
            "the fixture family of `{class}` is committed"
        );
        zip_of(&entries)
    }

    /// The companions of one family, by internal name.
    fn companions(&self, class: &str) -> Vec<String> {
        let nested = format!("{class}$");
        self.files
            .iter()
            .filter(|(name, _)| name.starts_with(&nested))
            .map(|(name, _)| {
                name.strip_suffix(".class")
                    .expect("a class file name ends in `.class`")
                    .to_owned()
            })
            .collect()
    }
}

/// javac 23.0.1, `javac --release 8 -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "EM.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/EM.class"),
    ),
    (
        "RG.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RG.class"),
    ),
    (
        "RG$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RG$Item.class"),
    ),
    (
        "RH.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RH.class"),
    ),
    (
        "RH$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RH$Item.class"),
    ),
    (
        "RJ.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RJ.class"),
    ),
    (
        "RJ$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RJ$Item.class"),
    ),
    (
        "RK.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RK.class"),
    ),
    (
        "RK$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RK$Item.class"),
    ),
    (
        "RL.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RL.class"),
    ),
    (
        "RL$Inner.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RL$Inner.class"),
    ),
    (
        "RM.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RM.class"),
    ),
    (
        "RM$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RM$Item.class"),
    ),
    (
        "RN.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RN.class"),
    ),
    (
        "RN$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RN$Item.class"),
    ),
    (
        "RO.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RO.class"),
    ),
    (
        "RO$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RO$Item.class"),
    ),
    (
        "RP.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RP.class"),
    ),
    (
        "RP$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RP$Item.class"),
    ),
    (
        "RP$Sub.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8/RP$Sub.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "EM.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/EM.class"),
    ),
    (
        "RG.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RG.class"),
    ),
    (
        "RG$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RG$Item.class"),
    ),
    (
        "RH.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RH.class"),
    ),
    (
        "RH$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RH$Item.class"),
    ),
    (
        "RJ.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RJ.class"),
    ),
    (
        "RJ$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RJ$Item.class"),
    ),
    (
        "RK.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RK.class"),
    ),
    (
        "RK$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RK$Item.class"),
    ),
    (
        "RL.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RL.class"),
    ),
    (
        "RL$Inner.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RL$Inner.class"),
    ),
    (
        "RM.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RM.class"),
    ),
    (
        "RM$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RM$Item.class"),
    ),
    (
        "RN.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RN.class"),
    ),
    (
        "RN$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RN$Item.class"),
    ),
    (
        "RO.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RO.class"),
    ),
    (
        "RO$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RO$Item.class"),
    ),
    (
        "RP.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RP.class"),
    ),
    (
        "RP$Item.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RP$Item.class"),
    ),
    (
        "RP$Sub.class",
        include_bytes!("fixtures/recover-array-element-field-receiver/v8-javac8/RP$Sub.class"),
    ),
];

const LEGS: &[Leg] = &[
    Leg {
        label: "javac 23.0.1 --release 8",
        files: V8_FILES,
        em_clinit: EM_CLINIT,
    },
    Leg {
        label: "Corretto 1.8.0_432 (real javac 8)",
        files: V8_JAVAC8_FILES,
        em_clinit: EM_CLINIT_JAVAC8,
    },
];

// -------------------------------------------------------------------------------------------
// The presented texts, pinned whole (the p3 surface tests' own convention).
// -------------------------------------------------------------------------------------------

/// `RG`'s `<clinit>`: the lookup table's loop body, written where the bytecode runs it.
const RG_CLINIT: &str = "    static {\n        // @method <clinit>()V\n        // @declaration a static initializer of `RG`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        RG$Item[] local1;\n        A = new RG$Item(\"alpha\");\n        B = new RG$Item(\"beta\");\n        C = new RG$Item(\"gamma\");\n        BY_LABEL = new java.util.HashMap();\n        RG$Item[] local0 = new RG$Item[]{RG.A, RG.B, RG.C};\n        local1 = local0;\n        for (RG$Item local4 : local1) {\n            RG.BY_LABEL.put((java.lang.Object) local4.label, (java.lang.Object) local4);\n        }\n    }\n";

/// `RG.of` and `RG.main`: the members around the anchor, unchanged by this change.
const RG_OF: &str = "    static RG$Item of(java.lang.String arg0) {\n        // @method of(Ljava/lang/String;)LRG$Item;\n        // @declaration a static method of `RG`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return (RG$Item) RG.BY_LABEL.get((java.lang.Object) arg0);\n    }\n";
const RG_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `RG`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println(\"\" + of(\"beta\") + \"/\" + of(\"alpha\") + \"/\" + of(\"?\"));\n        return;\n    }\n";

/// `RH`: the accumulating for-each body and the single-read return.
const RH_TOTAL: &str = "    static int total(RH$Item[] arg0) {\n        // @method total([LRH$Item;)I\n        // @declaration a static method of `RH`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local1;\n        RH$Item[] local2;\n        local1 = 0;\n        local2 = arg0;\n        for (RH$Item local5 : local2) {\n            local1 = local1 + local5.label.length();\n        }\n        return local1;\n    }\n";
const RH_FIRST: &str = "    static java.lang.String first(RH$Item[] arg0) {\n        // @method first([LRH$Item;)Ljava/lang/String;\n        // @declaration a static method of `RH`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0[0].label;\n    }\n";

/// `RK`: the element held in an explicitly typed local, and the element read directly.
const RK_VIA_ELEM_LOCAL: &str = "    static java.lang.String viaElemLocal(RK$Item[] arg0) {\n        // @method viaElemLocal([LRK$Item;)Ljava/lang/String;\n        // @declaration a static method of `RK`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        RK$Item local1 = arg0[0];\n        return local1.label;\n    }\n";
const RK_VIA_ELEM_DIRECT: &str = "    static java.lang.String viaElemDirect(RK$Item[] arg0) {\n        // @method viaElemDirect([LRK$Item;)Ljava/lang/String;\n        // @declaration a static method of `RK`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0[0].label;\n    }\n";

/// `RL`: the current class's own array and a companion class's array, both element receivers.
const RL_SELF_ELEM: &str = "    static java.lang.String selfElem(RL[] arg0) {\n        // @method selfElem([LRL;)Ljava/lang/String;\n        // @declaration a static method of `RL`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0[0].label;\n    }\n";
const RL_INNER_ELEM: &str = "    static java.lang.String innerElem(RL$Inner[] arg0) {\n        // @method innerElem([LRL$Inner;)Ljava/lang/String;\n        // @declaration a static method of `RL`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0[0].tag;\n    }\n";

/// `EM`'s `<clinit>`: the enum lookup table's loop body. The two legs differ in the shape the
/// constant pool gives the constants' array (`$values()` vs `new EM[]{…}`) — the debt this change
/// leaves alone; the loop body is this change's reading and is the same on both.
const EM_CLINIT: &str = "    static {\n        // @method <clinit>()V\n        // @declaration a static initializer of `EM`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        EM[] local0;\n        EM.A = new EM(\"A\", 0, \"alpha\");\n        EM.B = new EM(\"B\", 1, \"beta\");\n        EM.C = new EM(\"C\", 2, \"gamma\");\n        EM.$VALUES = $values();\n        EM.BY_LABEL = new java.util.HashMap();\n        local0 = values();\n        for (EM local3 : local0) {\n            EM.BY_LABEL.put((java.lang.Object) local3.label, (java.lang.Object) local3);\n        }\n    }\n";
const EM_CLINIT_JAVAC8: &str = "    static {\n        // @method <clinit>()V\n        // @declaration a static initializer of `EM`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        EM[] local0;\n        EM.A = new EM(\"A\", 0, \"alpha\");\n        EM.B = new EM(\"B\", 1, \"beta\");\n        EM.C = new EM(\"C\", 2, \"gamma\");\n        EM.$VALUES = new EM[]{EM.A, EM.B, EM.C};\n        EM.BY_LABEL = new java.util.HashMap();\n        local0 = values();\n        for (EM local3 : local0) {\n            EM.BY_LABEL.put((java.lang.Object) local3.label, (java.lang.Object) local3);\n        }\n    }\n";

/// `RJ`: the directly stated receiver and the parameter aliased into a local — the control that
/// must not move. Its texts are the ones this repository rendered **before** this change.
const RJ_DIRECT: &str = "    static java.lang.String direct(RJ$Item arg0) {\n        // @method direct(LRJ$Item;)Ljava/lang/String;\n        // @declaration a static method of `RJ`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0.label;\n    }\n";
const RJ_VIA_LOCAL: &str = "    static java.lang.String viaLocal(RJ$Item arg0) {\n        // @method viaLocal(LRJ$Item;)Ljava/lang/String;\n        // @declaration a static method of `RJ`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        RJ$Item local1 = arg0;\n        return local1.label;\n    }\n";

/// `RM`: the element's **method calls**, which resolve through the instruction's own descriptor and
/// never went through the field identity proof — byte-identical to the pre-change rendering.
const RM_LOOP_CALL: &str = "    static int loopCall(RM$Item[] arg0) {\n        // @method loopCall([LRM$Item;)I\n        // @declaration a static method of `RM`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local1;\n        RM$Item[] local2;\n        local1 = 0;\n        local2 = arg0;\n        for (RM$Item local5 : local2) {\n            local1 = local1 + local5.len();\n        }\n        return local1;\n    }\n";
const RM_ELEM_CALL: &str = "    static int elemCall(RM$Item[] arg0) {\n        // @method elemCall([LRM$Item;)I\n        // @declaration a static method of `RM`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0[0].len();\n    }\n";

/// `RO`: a call's result as the receiver — byte-identical to the pre-change rendering.
const RO_VIA_CALL: &str = "    static java.lang.String viaCall() {\n        // @method viaCall()Ljava/lang/String;\n        // @declaration a static method of `RO`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return make().label;\n    }\n";
const RO_VIA_CALL_LOCAL: &str = "    static java.lang.String viaCallLocal() {\n        // @method viaCallLocal()Ljava/lang/String;\n        // @declaration a static method of `RO`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        RO$Item local0 = make();\n        return local0.label;\n    }\n";

/// `RN`: the element's field **write** — read and write are one receiver comparison, so the write
/// moves with the read.
const RN_WRITE_ELEM: &str = "    static java.lang.String writeElem(RN$Item[] arg0, java.lang.String arg1) {\n        // @method writeElem([LRN$Item;Ljava/lang/String;)Ljava/lang/String;\n        // @declaration a static method of `RN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        arg0[0].tag = arg1;\n        return arg0[0].tag;\n    }\n";
const RN_WRITE_LOOP: &str = "    static int writeLoop(RN$Item[] arg0, java.lang.String arg1) {\n        // @method writeLoop([LRN$Item;Ljava/lang/String;)I\n        // @declaration a static method of `RN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        RN$Item[] local2;\n        local2 = arg0;\n        for (RN$Item local5 : local2) {\n            local5.tag = arg1;\n        }\n        return arg0[0].tag.length();\n    }\n";

/// `RP`: an array no fact states — a conditional and a two-branch merge — keeps the field identity
/// refusal, with the very message the pre-change run wrote.
const RP_MERGED: &str = "    static java.lang.String merged(RP$Sub[] arg0, RP$Item[] arg1, boolean arg2) {\n        // jarde: not recovered: the recovery run for `merged([LRP$Sub;[LRP$Item;Z)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method merged([LRP$Sub;[LRP$Item;Z)Ljava/lang/String;\n        // @declaration a static method of `RP`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 4 8 1\n        // the two values joined at BCI 9 do not have a conditional Java type this run can prove\n        // @bytecode 9\n        // the value at BCI 9 is the entry state of stack depth 0, which no instruction produced\n        // @bytecode 12\n        // the array instruction at BCI 12 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text\n        // @bytecode 13 12\n        // the field access at BCI 13 is not one this run proved names the member its own receiver's type declares, and a field instruction is presented only where the member it names is proven\n        // @bytecode 16 12\n        // the value at BCI 16 comes from the field access at BCI 13, which this run did not prove names the member its receiver's type declares\n    }\n";
const RP_BRANCHY: &str = "    static java.lang.String branchy(RP$Sub[] arg0, RP$Item[] arg1, boolean arg2) {\n        // @method branchy([LRP$Sub;[LRP$Item;Z)Ljava/lang/String;\n        // @declaration a static method of `RP`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        RP$Sub[] local3;\n        if (arg2) {\n            local3 = arg0;\n        } else {\n            local3 = arg1;\n        }\n        // @bytecode 13\n        // the array instruction at BCI 13 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text\n        // @bytecode 14 13\n        // the field access at BCI 14 is not one this run proved names the member its own receiver's type declares, and a field instruction is presented only where the member it names is proven\n        // @bytecode 17 13\n        // the value at BCI 17 comes from the field access at BCI 14, which this run did not prove names the member its receiver's type declares\n    }\n";

/// The one sentence a kept refusal carries, on both negative shapes.
const FIELD_IDENTITY_REFUSAL: &str =
    "is not one this run proved names the member its own receiver's type declares";

// -------------------------------------------------------------------------------------------
// The request path (the surface tests' shape): one snapshot, one class, the whole scope.
// -------------------------------------------------------------------------------------------

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a ZIP")
}

/// One class-source presentation over the snapshot's own root container: the policy `--policy
/// plain-jar` declares, which is what makes a companion class a member of the class it belongs to.
fn class_source_of(
    snapshot: &ArtifactSnapshot,
    name: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
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

/// One member's own record in the assembled source.
fn method_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's recovery report, for the planes the text alone cannot state.
fn recovered<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &method_of(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` is a recovered member: {other:?}"),
    }
}

/// One member's text, for the assertions that compare a whole presentation.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &method_of(report, name).text
}

/// The one ZIP the class-source request reads, built from the committed class files.
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

// -------------------------------------------------------------------------------------------
// The anchors.
// -------------------------------------------------------------------------------------------

#[test]
fn the_static_lookup_tables_loop_body_is_written() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RG"));
        let report = class_source_of(&snapshot, "RG", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "<clinit>"),
            RG_CLINIT,
            "{}: `RG.<clinit>` writes the loop body that was quoted away",
            leg.label
        );
        assert_eq!(text_of(&report, "of"), RG_OF, "{}: `RG.of`", leg.label);
        assert_eq!(
            text_of(&report, "main"),
            RG_MAIN,
            "{}: `RG.main`",
            leg.label
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{}: no instruction of `RG` stays quoted:\n{}",
            leg.label,
            report.text
        );
        // The statements the proof read stay answerable as anchors of the statement that took their
        // place: the table's own read (86), the field instruction (91) and the two values the call
        // reads (94, 96).
        let anchored: std::collections::BTreeSet<u32> = recovered(&report, "<clinit>")
            .source_map
            .segments()
            .iter()
            .flat_map(|segment| segment.origin().bcis())
            .collect();
        for bci in [86, 91, 94, 96] {
            assert!(
                anchored.contains(&bci),
                "{}: BCI {bci} is an anchor of the recovered statement: {anchored:?}",
                leg.label
            );
        }
    }
}

#[test]
fn the_accumulating_and_returning_shapes_are_written() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RH"));
        let report = class_source_of(&snapshot, "RH", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "total"),
            RH_TOTAL,
            "{}: `RH.total` accumulates the element's field",
            leg.label
        );
        assert_eq!(
            text_of(&report, "first"),
            RH_FIRST,
            "{}: `RH.first` reads the element's field",
            leg.label
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{}: no instruction of `RH` stays quoted:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_element_is_a_receiver_directly_and_through_a_local() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RK"));
        let report = class_source_of(&snapshot, "RK", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "viaElemLocal"),
            RK_VIA_ELEM_LOCAL,
            "{}: an element held in an explicitly typed local keeps its field access",
            leg.label
        );
        assert_eq!(
            text_of(&report, "viaElemDirect"),
            RK_VIA_ELEM_DIRECT,
            "{}: an element read directly keeps its field access",
            leg.label
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{}: no instruction of `RK` stays quoted:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_current_class_and_the_companion_class_arrays_both_recover() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RL"));
        let report = class_source_of(&snapshot, "RL", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "selfElem"),
            RL_SELF_ELEM,
            "{}: the current class's own array element",
            leg.label
        );
        assert_eq!(
            text_of(&report, "innerElem"),
            RL_INNER_ELEM,
            "{}: a companion class's array element",
            leg.label
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{}: no instruction of `RL` stays quoted:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn an_enum_lookup_tables_loop_body_is_written() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("EM"));
        let report = class_source_of(&snapshot, "EM", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "<clinit>"),
            leg.em_clinit,
            "{}: `EM.<clinit>` writes the loop body that was quoted away",
            leg.label
        );
        assert!(
            !text_of(&report, "<clinit>").contains("@bytecode"),
            "{}: the loop body quotes nothing:\n{}",
            leg.label,
            text_of(&report, "<clinit>")
        );
        // Only the constants' own pooled shape (the `$VALUES` array of the class's `values()`) is
        // left as the debt this change does not touch; the loop body is the reading it adds.
        assert!(
            text_of(&report, "<clinit>").contains("EM.BY_LABEL.put("),
            "{}: the lookup call is written:\n{}",
            leg.label,
            text_of(&report, "<clinit>")
        );
    }
}

#[test]
fn the_direct_parameter_and_local_alias_reads_do_not_move() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RJ"));
        let report = class_source_of(&snapshot, "RJ", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "direct"),
            RJ_DIRECT,
            "{}: a directly stated receiver is not this change's surface",
            leg.label
        );
        assert_eq!(
            text_of(&report, "viaLocal"),
            RJ_VIA_LOCAL,
            "{}: a parameter aliased into a local is not this change's surface",
            leg.label
        );
    }
}

#[test]
fn the_element_method_calls_do_not_move() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RM"));
        let report = class_source_of(&snapshot, "RM", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "loopCall"),
            RM_LOOP_CALL,
            "{}: the for-each element's method call renders as it always did",
            leg.label
        );
        assert_eq!(
            text_of(&report, "elemCall"),
            RM_ELEM_CALL,
            "{}: the direct element's method call renders as it always did",
            leg.label
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{}: no instruction of `RM` is quoted, before or after:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_call_result_receivers_do_not_move() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RO"));
        let report = class_source_of(&snapshot, "RO", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "viaCall"),
            RO_VIA_CALL,
            "{}: a call's result as receiver renders as it always did",
            leg.label
        );
        assert_eq!(
            text_of(&report, "viaCallLocal"),
            RO_VIA_CALL_LOCAL,
            "{}: a call's result held in a local renders as it always did",
            leg.label
        );
    }
}

#[test]
fn an_element_field_write_is_presented_by_the_same_proof() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RN"));
        let report = class_source_of(&snapshot, "RN", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "writeElem"),
            RN_WRITE_ELEM,
            "{}: the element's field write and its read back",
            leg.label
        );
        assert_eq!(
            text_of(&report, "writeLoop"),
            RN_WRITE_LOOP,
            "{}: the for-each element's field write",
            leg.label
        );
    }
}

#[test]
fn an_array_no_fact_states_keeps_the_field_identity_refusal() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RP"));
        let report = class_source_of(&snapshot, "RP", EnvironmentPolicy::PlainJar);
        assert_eq!(
            text_of(&report, "merged"),
            RP_MERGED,
            "{}: a conditional of two array types states no component type",
            leg.label
        );
        assert_eq!(
            text_of(&report, "branchy"),
            RP_BRANCHY,
            "{}: two branches of two array types state no component type",
            leg.label
        );
        for name in ["merged", "branchy"] {
            assert!(
                text_of(&report, name).contains(FIELD_IDENTITY_REFUSAL),
                "{}: `{name}` keeps the field identity refusal",
                leg.label
            );
        }
        // `merged`'s whole body is quoted (the conditional itself is unproved); `branchy`'s two
        // assignments are statements of their own and stay written, and only the field read the
        // merged local's element would be a receiver of stays refused.
        assert_eq!(
            recovered(&report, "merged").content,
            RecoveryContent::ExplanationOnly,
            "{}: `merged` presents no statement",
            leg.label
        );
        assert_eq!(
            recovered(&report, "branchy").content,
            RecoveryContent::ContainsStatements,
            "{}: `branchy` writes its two assignments",
            leg.label
        );
    }
}

// -------------------------------------------------------------------------------------------
// The replay: the presented text, stripped, compiled and run (needs a JDK on PATH).
// -------------------------------------------------------------------------------------------

static UNIQUE: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-array-element-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the replay directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// One member text with every `//` comment line dropped — the first half of the strip the patrol's
/// own `verify-wrong-RG.java` was made by.
fn comment_lines_dropped(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_owned)
        .collect()
}

/// The class-source text of one fixture as the patrol's own `verify-wrong-RG.java` reads it.
///
/// Three mechanical steps, none of them a rewrite of the recovery:
///
/// 1. every `//` comment line is dropped (the strip the patrol's file was made by);
/// 2. each companion's `X$Y` spelling becomes the nested simple name `Y` — the text spells a merged
///    companion's name as the class file's own internal name, and nesting it is what the patrol did
///    by hand for `RG$Item`;
/// 3. a companion the text does not already declare as a nested member class is nested in before
///    the class's own closing brace (a presentation merges a companion's declaration only where a
///    member's type projection needs the name, so the root sometimes spells the name alone).
fn stripped(root: &str, companions: &[(String, String)]) -> String {
    let mut body = comment_lines_dropped(root);
    for (internal, text) in companions {
        let simple = internal
            .rsplit('$')
            .next()
            .expect("a companion name holds its nesting");
        body = body
            .iter()
            .map(|line| line.replace(internal, simple))
            .collect();
        if body
            .iter()
            .any(|line| line.contains(&format!("static class {simple} extends")))
        {
            continue;
        }
        let mut block: Vec<String> = comment_lines_dropped(text)
            .iter()
            .map(|line| line.replace(internal, simple))
            .collect();
        for line in block.iter_mut() {
            if let Some(rest) = line.strip_prefix("class ")
                && let Some((_, tail)) = rest.split_once(" extends ")
            {
                *line = format!("static class {simple} extends {tail}");
            }
        }
        let closing = body.pop().expect("a class text closes");
        body.extend(block);
        body.push(closing);
    }
    body.join("\n") + "\n"
}

/// The stripped presentation of one fixture: its own text, with every companion's text nested in.
fn replay_source(leg: &Leg, class: &str) -> String {
    let snapshot = open(&leg.fixture(class));
    let root = class_source_of(&snapshot, class, EnvironmentPolicy::PlainJar).text;
    let companions: Vec<(String, String)> = leg
        .companions(class)
        .into_iter()
        .map(|name| {
            let text = class_source_of(&snapshot, &name, EnvironmentPolicy::PlainJar).text;
            (name, text)
        })
        .collect();
    stripped(&root, &companions)
}

/// The fixture's own class files, written where a JVM can load them.
fn original_classes(leg: &Leg, class: &str, directory: &Path) {
    let own = format!("{class}.class");
    let nested = format!("{class}$");
    for (name, bytes) in leg
        .files
        .iter()
        .filter(|(name, _)| *name == own || name.starts_with(&nested))
    {
        fs::write(directory.join(name), bytes).expect("the fixture class is written");
    }
}

/// Compile one unit with the installed JDK's `javac --release 8`.
fn compile(directory: &Path, source: &Path) {
    let output = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", "-d"])
        .arg(directory)
        .arg(source)
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        output.status.success(),
        "javac rejected the presented text:\n{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        fs::read_to_string(source).expect("the text reads")
    );
}

/// Run one class under `-Xverify:all` and answer what it printed.
fn run(directory: &Path, class: &str) -> String {
    let output = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(directory)
        .arg(class)
        .output()
        .expect("the installed JDK provides java");
    assert!(
        output.status.success(),
        "`{class}` failed under -Xverify:all:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_owned()
}

/// The stripped text compiled and run, and the fixture's own classes run — the two answers a leg's
/// replay compares.
fn replay(leg: &Leg, class: &str) -> (String, String) {
    let temp = TempDir::new(class);
    let presented = temp.path().join("presented");
    let original = temp.path().join("original");
    fs::create_dir_all(&presented).expect("create the presented directory");
    fs::create_dir_all(&original).expect("create the original directory");
    let source = temp.path().join(format!("{class}.java"));
    fs::write(&source, replay_source(leg, class)).expect("write the presented text");
    compile(&presented, &source);
    original_classes(leg, class, &original);
    (run(&presented, class), run(&original, class))
}

#[test]
#[ignore = "needs a JDK on PATH: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` and runs it with `-Xverify:all` (see the module doc)"]
fn the_recovered_lookup_table_answers_what_its_class_answers() {
    for leg in LEGS {
        let (presented, original) = replay(leg, "RG");
        let answers = |output: &str| {
            output
                .split('/')
                .map(str::to_owned)
                .collect::<Vec<String>>()
        };
        let presented = answers(&presented);
        let original = answers(&original);
        assert_eq!(
            presented.len(),
            3,
            "{}: `RG.main` answers three lookups",
            leg.label
        );
        assert_eq!(
            original.len(),
            3,
            "{}: the fixture answers three lookups",
            leg.label
        );
        // `Item` has no `toString`, so the two classes' identity hashes differ: what the anchor
        // states is which lookups find a member at all. The pre-change text read `null/null/null`
        // here, which is the compilable-wrong this change closes.
        assert_ne!(presented[0], "null", "{}: `of(\"beta\")`", leg.label);
        assert_ne!(presented[1], "null", "{}: `of(\"alpha\")`", leg.label);
        assert_eq!(presented[2], "null", "{}: `of(\"?\")`", leg.label);
        assert_ne!(original[0], "null", "{}: the fixture's own run", leg.label);
        assert_ne!(original[1], "null", "{}: the fixture's own run", leg.label);
        assert_eq!(original[2], "null", "{}: the fixture's own run", leg.label);
    }
}

#[test]
#[ignore = "needs a JDK on PATH: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` and runs it with `-Xverify:all` (see the module doc)"]
fn every_recovered_fixture_answers_what_its_class_answers() {
    // The value each fixture's entry point prints: the patrol's own readings (`5` for `RH.total`,
    // `q`/`q`, `s`/`i`, `5`/`2` for `RM`'s method calls) and the controls' own values.
    for (class, expected) in [
        ("RH", "5/xy"),
        ("RK", "q/q"),
        ("RL", "s/i"),
        ("RM", "5/2"),
        ("RJ", "d/d"),
        ("RO", "m/m"),
        ("RN", "X/2"),
    ] {
        for leg in LEGS {
            let (presented, original) = replay(leg, class);
            assert_eq!(
                original, expected,
                "{}: the fixture's own classes answer `{expected}`",
                leg.label
            );
            assert_eq!(
                presented, original,
                "{}: `{class}`'s recovered text answers what the class answers",
                leg.label
            );
        }
    }
}

#[test]
#[ignore = "needs a JDK on PATH: the enum constants' pooled shape is a rendering debt this change \
            does not touch, and the stripped text must stay uncompilable (see the module doc)"]
fn the_enum_text_stays_uncompilable_on_its_constant_shape() {
    for leg in LEGS {
        let temp = TempDir::new("EM-debt");
        let source = temp.path().join("EM.java");
        fs::write(&source, replay_source(leg, "EM")).expect("write the presented text");
        let output = Command::new("javac")
            .args(["--release", "8", "-Xlint:-options", "-d"])
            .arg(temp.path())
            .arg(&source)
            .output()
            .expect("the installed JDK provides javac");
        // The reviewed state of the debt: `EM`'s constants are the class's own `$VALUES` shape and
        // render as blank declarations, so the text is pinned and never compiled. A run that
        // compiles it is a debt this test's assertion wants to hear about.
        assert!(
            !output.status.success(),
            "the enum text must stay uncompilable while its constants render as blank declarations:\n{}",
            fs::read_to_string(&source).expect("the text reads")
        );
    }
}
