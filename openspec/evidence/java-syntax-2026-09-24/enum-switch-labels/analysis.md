# Enum switch labels: method-local shape versus cross-class mapping

`javac 23.0.1 --release 8 -g:none` builds `Hue`, `EnumSwitchSubject`, its synthetic
`EnumSwitchSubject$1`, and a source-only runner. The original verified class and the complete
JADX 1.5.6 output both recompile and run with `-Xverify:all`:

| Hue input | Original class | Recompiled JADX |
| --- | --- | --- |
| RED | `1|1` | `1|1` |
| BLUE | `2|2` | `2|2` |
| GREEN | `3|3` | `3|3` |
| null | `null|0` | `null|0` |

The subject's `choose` reads `EnumSwitchSubject$1.$SwitchMap$Hue[hue.ordinal()]` and switches
on integers 1 and 2. Its method contains no link from 1/2 to RED/BLUE. The synthetic class's
`<clinit>` stores 1 at `Hue.RED.ordinal()` and 2 at `Hue.BLUE.ordinal()`, each under a
`NoSuchFieldError` handler. The method-level `enumswitch@1` correctly recognizes the array read,
but does not have the other class's initializer facts.

`patch_swapped_map.py` changes only those two `iconst` opcodes, preserving the class version,
switch method, field names, descriptors and handlers. The patched class SHA-256 is
`65e6ad3fcfc997a554dac632132f866ac1d09a6ea2821c2bf464180b912852fd`; it passes JVM
verification and produces `2|2`, `1|1`, `3|3`, `null|0`. JADX reads the altered mapping and emits
`case BLUE: return mark(1)` followed by `case RED: return mark(2)`; that complete source also
recompiles and matches the patched four lines. Field names and integer switch keys alone cannot
justify source enum labels.

Architectural implication: a future source-level enum-switch projection needs a bounded,
same-environment read of the selected synthetic class and a proof of its whole map initializer,
including `values().length`, uniquely owned array writes, each enum field/ordinal and the
`NoSuchFieldError` edges. It should then bind each integer key to the proven enum field and check
that an emitted direct `switch (hue)` preserves null evaluation and the selected arm's effects.
This can reuse the existing physical identities, resolver and class-source projection seam; it
does not require a new general JVM IR node or a name-based `$SwitchMap` shortcut. Unknown or
mutable mappings must leave the integer selector visible or refuse projection.

`replay.py` rebuilt all four original class files byte-for-byte, then replayed both selected JARs
against Jarde CLI SHA-256 `6e2d1a3ee7c18622d98c0695ecc87bbaaad03acecc8795c1d17567aa6330859f`.
The full logs and source are in `replay-baseline/`. Both Jarde inputs produced the same complete
`EnumSwitchSubject` text (SHA-256 `3a47145066921960fed93c18b1353bb9072b4ecf930c988bb73370494fbf6c39`):
`switch (EnumSwitchSubject$1.$SwitchMap$Hue[arg0.ordinal()])` with integer cases 1/2.
Both `javac --release 8` runs failed at that expression: javac cannot resolve the synthetic
`$SwitchMap$Hue` field even when the matching helper `.class` is on its classpath. Original and
JADX complete class runs both compiled, verified and matched their respective four-line oracles.
This is therefore a full-class compilability gap as well as a source-level label gap.

## Negative boundaries

Three additional, verifier-checked class sets exercise cases where the table proof must refuse.
`negative/multi-write` stores RED twice in one handler-protected region; with
`java -Xverify:all` it runs as `2|2`, `2|2`, `3|3`, `null|0`. Its helper SHA-256 is
`ef4d861e0c04d9395a706651d6f0bb18e124881e11cae8f42c47b2c1fbafea66`. The second store
overwrites the first, so treating only the first value as the mapping would be wrong.

`negative/duplicate-key` starts from the frozen helper and changes only BLUE's `iconst_2` to
`iconst_1` at `<clinit>` BCI 33. Its SHA-256 is
`da5a7ddb94ccc3b952d7661fdfbae74a12aa51b94469fd9c10a846f227067884`; it passes
`-Xverify:all` and runs as `1|1`, `1|1`, `3|3`, `null|0`. The `patch_duplicate_key.py` replay
checks the original helper hash and the instruction at the patch site before writing this class.

`negative/missing-enum` contains the frozen subject, helper and runner but omits `Hue.class`.
With `-Xverify:all`, class loading exits 1 with `NoClassDefFoundError: Hue` and a
`ClassNotFoundException` cause. This records an unresolved dependency boundary; it does not claim
that no `Hue` definition exists outside the selected environment. The positive swapped helper
remains byte-for-byte frozen by `patch_swapped_map.py` at SHA-256
`65e6ad3fcfc997a554dac632132f866ac1d09a6ea2821c2bf464180b912852fd` and has unchanged subject
and method bytes.

`negative/aliased-enum` starts from the frozen swapped-map input and changes only `Hue.<clinit>`
BCI 13–22, replacing the BLUE allocation with `getstatic Hue.RED` plus seven `nop` bytes. The
patched `Hue.class` SHA-256 is `c170d13e3e0bacaed624b7f5c4d4f845135a4b7633ee563040414329a7811d9b2`;
`java -Xverify:all` accepts it. Both `Hue.RED` and `Hue.BLUE` then refer to the same object and
ordinal, so the original helper's later BLUE mapping overwrites RED's earlier mapping. The original
four lines are `1|1`, `1|1`, `3|3`, `null|0`, while a direct switch using the map-proved swapped
labels recompiles to `2|2`, `2|2`, `3|3`, `null|0`. The replay is
`negative/aliased-enum/replay.py`; this proves the need to validate enum object identity/ordinal
uniqueness in addition to the helper's actual stores and the enum's field flags.

OpenSpec planning artifacts are in `openspec/changes/project-proved-enum-switch-labels/`.
