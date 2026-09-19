//! Java 8 recovery: the layer that turns one method's analysis into Java text with a source map
//! (P3 1.3).
//!
//! # Where it sits
//!
//! ```text
//! jarde-reader  facts (class bytes, constant pool, attributes, bytecode)
//!       │
//!       ▼
//! jarde-jvm     canonical CFG, frames, SSA and effects — the read-only payload of one run
//!       │
//!       ▼
//! jarde-java    ① normal-flow view → ② facts → ③ Region → ④ AST + emitter → text, segments,
//!       │       diagnostics
//!       ▼
//! jarde / CLI   entry-point delegation only
//! ```
//!
//! The direction is one-way: nothing below names a region, an AST node, a name or a segment, which is
//! the property CI's layered-dependency gate checks by reading the manifest graph (`jarde-reader`,
//! `jarde-query` and `jarde-jvm` must not reach `jarde-java`).
//!
//! # The four layers of the recovery side, and the one direction between them
//!
//! 1. [`normal_flow`] — the canonical blocks projected onto their **plain transfers** (fall-through,
//!    branch, `goto`, and the successor of a `ret`). It deletes nothing, writes nothing back, and
//!    carries no exception semantics: the projection is a second, derived graph the structure layer
//!    runs dominance and cycle questions on.
//! 2. [`facts`] — what the class file says and the payload does not publish: the method's identity,
//!    the debug names of its locals, and the decoded [`facts::Operation`]s of its body. Facts, not
//!    syntax: "this BCI reads local 1", never "this BCI is an assignment to `count`".
//! 3. [`region`] — the *structure*: straight runs, `if`/`else` with both arms and their join, or a
//!    stated fallback with its reason. It emits no text and decides no syntax. Evidence that is not
//!    there is never an empty body.
//! 4. [`ast`] + [`build`] + [`emit`] — the *syntax*: the statement and expression nodes (each with
//!    its [`source_map::OriginSet`]), and the emitter that writes the text and the segment table in
//!    the same writes, checking the output budget at every write entry and escaping what Java cannot
//!    carry literally.
//!
//! # What this slice recovers, and what it refuses
//!
//! Straight-line bodies and `if`/`else` over the provable subset of `ast`; a real body in, Java text
//! plus a segment table out ([`recover`]). Loops, `switch`, `try`/`finally`, irreducible or crossing
//! exception regions, lambdas, string concatenation and inner classes are **not** recovered here —
//! they are 1.3b/2.x, and a body that needs one of them is reported as a fallback with its BCIs
//! quoted, or as a stop with no artifact at all. Neither is ever reported as an empty body or as a
//! success.
//!
//! # Cost
//!
//! No third-party dependency was added for the presentation: the AST, the escaping, the segment
//! table and the budget checks are this crate's own, because every candidate library emitted text in
//! one call and could not record a node's position while it wrote (P3 1.2's decisive admission
//! criterion). `petgraph` is reused for the projection's graph algorithms, exactly as the layer below
//! reuses it for the raw CFG.

pub mod accessor;
pub mod ast;
pub mod bridge;
pub mod concat;
pub mod declaration;
pub mod enumswitch;
pub mod facts;
pub mod field;
pub mod init;
pub mod lambda;
pub mod names;
pub mod normal_flow;
pub mod pass;
pub mod region;
pub mod report;
pub mod source_map;
pub mod stop;

pub(crate) mod build;
pub(crate) mod decode;
pub(crate) mod emit;
#[cfg(test)]
mod oracle;
pub(crate) mod refusal;

pub use accessor::{
    AccessorCandidate, AccessorField, AccessorRecord, AccessorRefusal, AccessorShape,
};
pub use bridge::{BridgeRecord, BridgeRefusal};
pub use concat::{ConcatAppend, ConcatRecord, ConcatRefusal};
pub use declaration::{DeclarationForm, DeclarationRecord, DeclarationRefusal};
pub use enumswitch::{EnumSwitchRecord, EnumSwitchRefusal, IndexCall, TableRead};
pub use field::{FieldRecord, FieldRefusal};
pub use init::{InitRecord, InitRefusal, NewRecord, NewRefusal};
pub use lambda::{LambdaCapture, LambdaForm, LambdaRecord, LambdaRefusal};
pub use pass::{IrTable, Pass, Precondition, RecoveryProfile, RuleVersion};
pub use report::{RecoveryOutcome, RecoveryReport, RecoveryRequest, RegionRecord, recover};

pub use ast::{BinaryOp, ConstructorTarget, Expr, ExprKind, Stmt, StmtKind, Type};
pub use emit::{comment_text, escape_string};
pub use facts::{
    ACC_BRIDGE, ACC_PUBLIC, ACC_STATIC, ACC_SYNTHETIC, ArithmeticOp, CallTarget, ClassMembers,
    CompareOp, ConstantValue, DeclaringClass, FieldAccess, InvokeKind, MemberBody, MethodFacts,
    Operation, RecoveryFacts,
};
pub use names::{AliasReason, NameTable, RenderedName, alias_for, is_java_identifier};
pub use normal_flow::{ExcludedEdges, NormalFlowView};
pub use region::{FallbackReason, Recovered, Region};
pub use source_map::{Origin, OriginSet, Provenance, Segment, SourceMap};
pub use stop::StopReason;
