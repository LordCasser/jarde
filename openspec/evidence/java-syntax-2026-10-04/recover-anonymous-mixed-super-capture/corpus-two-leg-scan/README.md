# Corpus two-leg scan (2026-10-04)

Legs: `996ac2b7` (baseline, built in a throwaway worktree + CARGO_TARGET_DIR) versus this
branch's binary, both driven over every frozen `tests/fixtures/proved-java-structure/`
fixture set (one jar per directory, one `class-source --policy plain-jar --release 8`
request per class). 49 renders per leg.

`diff.txt`: exactly one differing rendering — `anonymous-super-mixed-direct`'s root, the
new mixed-form anchor (`old` = physical `new AnonymousSuperMixedDirect$1(…)` text;
`new` = projected `new Base(text(…), number(…)) { … }`). Every other rendering of the
corpus is byte-identical, including the two same-shaped fixtures that keep refusing for
pre-existing reasons (`anonymous-capture`: nested superclass name; `anonymous-top-level`:
root return type not the superclass).

Reproduce: pack each fixture directory's `.class` files, request every class with both
binaries, `diff -rq`.
