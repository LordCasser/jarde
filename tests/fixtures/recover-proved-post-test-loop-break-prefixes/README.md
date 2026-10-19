# Original TestDoWhileBreak public fixture candidate

Private, unexecuted OpenSpec task 2.2 candidate. The four class files are copied byte-for-byte from the accepted complete TestDoWhileBreak baseline. The two plain JARs are copied exactly from that baseline (`debug-full-classes.jar` and `no-debug-full-classes.jar`), including the matching outer and child classes. The complete 563-byte upstream source and baseline driver are copied unchanged, as are the existing JADX LICENSE/NOTICE. No source is trimmed or recompiled. `manifest.json` records each fixture input’s relative path, bytes, SHA-256 and mode.

`tests/p3_post_test_loop_break_prefixes.rs` follows the existing continue-ladder public facade test: class/JAR × debug/no-debug × default/all; complete two-method/zero-field inventory; structured `test(I)I`; stable default/all class/method text, identity and map; valid UTF-8 spans and method-owned BCI anchors. It proposes full 14-instruction BCI coverage, which root should confirm against actual reports before integration. Source-shape checks preserve the conditional reset-before-break and increment path afterward but accept equivalent while/do spellings. A second public test verifies pre-cancellation does not publish partial class text.

The Reader delta `[4,8,0,6,0]` and total `[1334,5866,631,4223,16]` are unverified projections. Root must independently parse the fixtures and run the census/fingerprint acceptance gates. The fingerprint regeneration entry point is `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`; it was not run during this preparation.

This is a private static candidate only. No tests or compiler/runtime tools were run.
