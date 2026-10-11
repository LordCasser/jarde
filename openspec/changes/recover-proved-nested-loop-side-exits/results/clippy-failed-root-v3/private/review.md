# Minimal field_reassign_with_default correction

- Base SHA-256: `c9021b57bce4d8968efc8b65f60d2a7c9aaa5a9df9494f6604609b4a611b9582`
- Candidate SHA-256: `5313e42faa9d7025d929f203c29e1c3bba271ec6eaf63a70323d2816a6a42604`
- The only edit replaces the two-line `Frame::default()` then `scope` assignment with one `Frame { scope: Some(...), ..Frame::default() }` initializer. `mut` remains because the following code appends loop targets. The scope value and all other fields are unchanged.
- Candidate bytes equal the base after exactly one replacement. Static inspection only; no lint or toolchain was run.
