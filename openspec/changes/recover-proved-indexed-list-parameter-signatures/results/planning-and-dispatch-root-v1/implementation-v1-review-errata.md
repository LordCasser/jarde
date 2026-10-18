# Erratum for review v1

This note supersedes finding 4 in `review.md` (SHA256
`eca994e7672fe1b66b83e822736c72abccb01639728036ff1d55367c8dea6338`). That
finding was incorrect and should be disregarded; the original review is retained
unchanged for traceability.

`SignatureType::ClassTypeSegment.binary_name` is cumulative for nested class
segments. In the pinned reader source, `Parser::class_type` starts with the
outer binary name, appends `$` plus each nested identifier, then clones the
cumulative name into each segment (`crates/jarde-reader/src/signature.rs:836-855`).
Consequently, using `element.segments.last().binary_name` preserves the full
binary name (for example `p/Outer$Inner`); it does not discard the outer class.
There is no nested-class false-refusal issue established by that code, and no
selector change is warranted on this basis.

The remaining findings in `review.md` are unchanged. This erratum is static
source verification only; no build or tests were run.
