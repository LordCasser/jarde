# ConditionalConstructionPositive range audit

Static-only supplement to the conditional-constructor public API test review. No shared file was edited and no compiler, test runner, CLI, Git, or Java toolchain was run.

The candidate selects the first target-method physical BCI present in the full source map and outside every `NewRecord` position, then uses that instruction's width. In the frozen v2 `javac23 --release 8` all-evidence report for `twoConstructedArms(ZZZZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;`, the mapped target BCIs begin at 0. The two NewRecords cover `[8,11,15,18]` and `[24,27,31,34]`, so BCI 0 is eligible. The frozen instruction table shows BCI 0 is `iload_0`, width 1; therefore the exact shared request interval is `[0,1)`.

Applying that interval to every other method in `ConditionalConstructionPositive` yields exactly two invalid members. `condition(ZZ)Z` and `argument(Ljava/lang/String;Z)Ljava/lang/String;` both begin with `getstatic` at BCI 0, whose width is 3, so BCI 1 is not an instruction boundary. The constructor begins with one-byte `aload_0`; `reset()` begins with one-byte `iconst_0`; `oneConstructedArm`, `twoConstructedArms`, and `writeStaticSlot` begin with one-byte `iload`; `writeInstanceSlot` begins with one-byte `aload_0`. Those ranges are valid. The class has no `<clinit>` method.

Thus the exact expected invalid set is:

- `condition(ZZ)Z`
- `argument(Ljava/lang/String;Z)Ljava/lang/String;`

The earlier v5 range raw showed the constructor complete and only these two helpers refused, consistent with this derivation, but that raw is not the basis for determining the new selection: the frozen v2 map and instruction table are.

The corrected private candidate file SHA-256 is `9a301776907ceded649aa9d1addba7d71ce45d128bff4cab3a8b15078d84ad9d`.
