# Minimal Clippy correction

- Source: `/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs`
- Base SHA-256: `26f785844eba008150acd4debdba3133642090202a177322af5a87d782dd2e43`
- Candidate SHA-256: `c9021b57bce4d8968efc8b65f60d2a7c9aaa5a9df9494f6604609b4a611b9582`
- Exact one-expression replacement at line 5294: `prefix.first().is_none()` → `prefix.is_empty()`. Here `prefix` is bound by `let [Region::Straight { blocks: prefix }] = run.as_slice()`, and both predicates identify an empty block sequence.
- Candidate bytes equal the base bytes with only that unique substring replaced.
- Static read only; no lint, Cargo, Git, or other toolchain was run.
