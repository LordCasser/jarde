# Analysis/runtime runner notes for the int-overload boundary

This is a proposal for root's private runner, not a command executed here.

1. Compile `CharProducerIntOverload.java` with the selected javac/debug mode into a private directory. Preserve source, class bytes, javac version/flags, and hashes.
2. Through the same production class reader and `analyze_method_ir` path used by the permanent CF12 test, inspect `render(Ljava/lang/String;)Ljava/lang/String;`. Confirm from decoded operations that the producer is `String.charAt(I)C`, the local's actual physical writes include its assignment to the literal 46, and the pool-selected builder call is `append(I)Ljava/lang/StringBuilder;`. Capture actual BCIs and stored SSA definitions rather than assuming javac layout.
3. Pass only the reader's actual debug-local facts through the facade seam. Compare default and all source text/maps. Assert the local may be rendered as `char` only if the full all-write proof warrants it, while the call expression retains an explicit int widening cast because its descriptor is `(I)`.
4. Run the untouched valid fixture class and assert stdout is exactly `46` (with the runner's normal trailing newline). If a recovered class is separately recompiled, compare that behavior too. Never execute byte-mutated negative-control classes.

The key adversarial distinction is `46` versus `.`: an overload chosen as `append(I)` renders decimal digits, while an accidental `append(C)` renders the character with code point 46. The fixture should also be checked in the class/IR report so output alone is not treated as proof of which overload the original bytes selected.
