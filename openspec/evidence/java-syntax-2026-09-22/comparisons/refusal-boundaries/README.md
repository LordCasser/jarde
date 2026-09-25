# `long_eq` refusal-boundary probes

These are derived, verifier-valid probes for the frozen
`tests/fixtures/p3-numeric-comparison/v8/NumericComparisons.class`. The fixture itself remains
unchanged (SHA-256
`7ab3d3cc033ce1bed61b60a91a90e1ecf968495081b0578aff22bed716f44aa3`). The Python script matches
the complete `long_eq` Code sequence exactly once; it does not parse or rewrite the constant pool
and never writes the fixture.

The original method is:

```text
0: lload_0
1: lload_2
2: lcmp
3: ifne 9
6: bipush 7
8: ireturn
9: bipush 9
11: ireturn
```

The `dup-pop` probe inserts `dup; pop` at BCI 3. The branch moves to BCI 5 and its target moves to
BCI 11, so the original same frame becomes `frame_type = 11`. The `local` probe inserts
`istore 4; iload 4` at BCI 3, raises `max_locals` from 4 to 5, and changes the frame to
`append_frame(offset_delta = 13, Integer)`. The Code and nested StackMapTable lengths are updated
by the script. Both classes pass `java -Xverify:all` and the temporary driver prints `7`.

The archived `javap.txt` files show the exact patched instruction streams. The Rust tests construct
the same patches in memory and require source-map entries for `lcmp` and the moved zero branch while
rejecting a direct `if (arg0 == arg2)` rewrite. These probes are refusal evidence for the
single-reader, immediately-consumed condition rule; they do not change the fixture census or claim
that either shape is recoverable.

