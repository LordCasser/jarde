# recover-array-element-field-receiver — frozen fixtures

The array-element-field-receiver patrol's own shapes
(`openspec/evidence/java-syntax-2026-10-05/array-element-field-receiver-patrol/`), compiled by both
javac legs. `tests/recover_array_element_field_receiver.rs` reads exactly these bytes.

## Sources

`RG`–`RO` are the patrol's fixtures; `RJ`, `EM` and `RP` are this change's additions for the two
controls and the negative (`RJ` = the directly stated receiver the patrol's matrix calls
`direct(Item)`/`viaLocal`, `EM` = the enum lookup table the patrol names `EM.Code`, `RP` = an array
whose component type no fact states).

One deliberate difference from the patrol's sources: **the entry points**. The patrol writes
`main(…){ println(total(new Item[]{ new Item("ab"), … })) }` — an inline array literal whose
elements are allocations, which this build refuses independently of this change (the inline
array-initializer allocation channel, out of scope here). Each fixture's `main` therefore hoists the
array into a local and reads its elements from static fields; the methods this change is about
(`total`, `first`, `selfElem`, `innerElem`, `viaElemLocal`, `viaElemDirect`, `loopCall`, `elemCall`,
`direct`, `viaLocal`, `writeElem`, `writeLoop`, `viaCall`, `viaCallLocal`, `RG`'s `<clinit>`, `EM`'s
`<clinit>`) are the patrol's own, verbatim, and every program still prints the patrol's values
(`5`, `q/q`, `s/i`, `X/2`, …). `RP.merged`/`RP.branchy` are new shapes with no patrol counterpart.

| source | SHA-256 |
| --- | --- |
| `EM.java` | `617384830e0f81ed8802b198e8fb83abd9182518af8b8faac600250e47c83628` |
| `RG.java` | `10ee58f03885dd6b0745f300520cd2fe5ac8e11eb48c5123fb204ca3f8147749` |
| `RH.java` | `969d841cf7e83308dac2e7ac1bbfa0c247f1b5808043584e097bcf44c6807e61` |
| `RJ.java` | `f3ead146fd042b93040d6fdd25f8db3a331c131c149cc9aceea391a71c4bc472` |
| `RK.java` | `883438284c3b9a15e614014a8ffadba6e0201858f2d6fc22eeebbb2b028f2518` |
| `RL.java` | `49c758361091a9187f3de50660144bd7acd28d884c0445776f32dd1e222e614e` |
| `RM.java` | `3fadf1e686778efe15fcfdf7b72533f3f105b9b8c3be964bb62e187df920ed15` |
| `RN.java` | `f52ef94e9e2a93cabdbfff05c3417f7b71190c462459f6c376617dbdd15715b0` |
| `RO.java` | `c1b61eb4164f36b0a2b4e0e7200a26b651708b5b58934c3b89b0d8f463f3849f` |
| `RP.java` | `dc87a88dc1e51c3c90b06c165921598ef874bc86387016f5fa064e217f210bf7` |

## The two legs (one source, two compilers)

- `v8/` — **javac 23.0.1**, `javac --release 8 -Xlint:-options -d v8 *.java` (default debug info:
  no `LocalVariableTable`, which is why the presented texts spell parameters `arg0` and locals
  `localN`, exactly as the patrol's own readings do);
- `v8-javac8/` — **real javac 8**, Corretto 1.8.0_432
  (`/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`,
  `javac -version` → `javac 1.8.0_432`, **no `--release`**: the default target is Java 8),
  `javac -d v8-javac8 *.java`.

Both legs present byte-identical member texts for every pinned member **except `EM.<clinit>`**: the
two compilers write the enum constant pool differently (`$VALUES = $values();` under javac 23 vs
`$VALUES = new EM[]{…}` under javac 8), so that one text is pinned per leg.

| class file | bytes | SHA-256 |
| --- | ---: | --- |
| `v8/EM.class` | 1,940 | `482c75b5cf18f25fdcd99288eefb049d4e84f630b9d96f2b05b8c6444ec838e1` |
| `v8/RG.class` | 1,503 | `05f9e152e1353ba766dc48114421c788a4bf26c9d54129624ce2f7f01333ed1a` |
| `v8/RG$Item.class` | 305 | `4c641fa2955892e1141ff4ac47846f8877a32a753bba1215ac340c9695c74ad4` |
| `v8/RH.class` | 1,283 | `b8f1bc779d8ddd52e34d6bf7cd90e4aef4fd204872f3785eaf615b8c3389a11a` |
| `v8/RH$Item.class` | 305 | `56406b3462a72e241c7392a975120c7bbb43a900d3c10436f79b0b1b6a7c3f32` |
| `v8/RJ.class` | 945 | `e09dd636b672ffdaf6698709fb2bd1388a67322c4552898a1df56e0acf6ab875` |
| `v8/RJ$Item.class` | 305 | `dad9654e44f1d477ab7ab189d1dfe5368acb3e9142752139ac0ab64d30f06c25` |
| `v8/RK.class` | 968 | `e7bb504c1bc332722a623a9f9fb78b0826208568649e48629af1fdbe33c31267` |
| `v8/RK$Item.class` | 305 | `737e29c547c3cc7a06964b605c8d7e887053486b184b54e6ae23ee37ffc925f7` |
| `v8/RL.class` | 1,099 | `35f563401e36bd6888a62727145b2b991404762a938d357e29b4bcf29a8ad7fa` |
| `v8/RL$Inner.class` | 305 | `d1e093b5c73d47ff08adea4240cfe485c6a7c1b0e711cd400f5547dd40975f24` |
| `v8/RM.class` | 1,181 | `61442c740ef04e64932b36c1c6ca438c40f319397d24b1b96a3ffdde02565604` |
| `v8/RM$Item.class` | 404 | `06a9877d9cb33a4a7d6cc632dbb668083965971218081101bfe43290d46bdfe4` |
| `v8/RN.class` | 1,262 | `03027303bf725f119c6da6be4242d6a44fe848abef750d1b68670f1530ddb4aa` |
| `v8/RN$Item.class` | 303 | `f0c92aee618f5cf2626dea10dae0191518c2f8c49b02c6550e3fb9fb7a893751` |
| `v8/RO.class` | 899 | `ce2b72ea870a2f4ddfa243f7681f3d9132bca8a306e78b7a34b53f855f66ab9e` |
| `v8/RO$Item.class` | 305 | `0fdc1ca85b2100581c8682b7c39dea8ab40f360b386e058b1cdb6bed047ba5b0` |
| `v8/RP.class` | 1,159 | `5ae6322070253ec4f77467d5473b98021c17e8021520b837da1eda718a91e494` |
| `v8/RP$Item.class` | 305 | `ec88b4efdf09737cb62bc3623139d5d97e97ff30548933a0d6b76e96eb675b0f` |
| `v8/RP$Sub.class` | 252 | `324df2673d3bc04efced8090d8d2ec305730dea4894aaceac5e20cb556c0c8e` |
| `v8-javac8/EM.class` | 1,832 | `6559a3d584b229cc4aef43afdfcd019a18f4420cf72b58758dacaf58f28dc6b8` |
| `v8-javac8/RG.class` | 1,503 | `f1d37e3c4b91cef51ec06e391c1222a0cc4cd9c0c6f1788ec0a19abd514f8264` |
| `v8-javac8/RG$Item.class` | 305 | `d414d0fd292631397e967f16aa25eb44b72db31582215d9b5082985dd5304a36` |
| `v8-javac8/RH.class` | 1,283 | `bbadb659c5b4cffb86e71699d782393fd71f75e585c338ea895d255508f97216` |
| `v8-javac8/RH$Item.class` | 305 | `0eea6d424804feb308b5f97f1afe6097a5f269c255a0d04dda280453e73f823b` |
| `v8-javac8/RJ.class` | 945 | `49f6a3280ef6e195cfeda874c3590977ebeeca10ae2e610ee24dcc45ec44536f` |
| `v8-javac8/RJ$Item.class` | 305 | `83d2cd0c9686e30f02bc23178173820f44a05fd5b0f24e30966e06c183a52927` |
| `v8-javac8/RK.class` | 968 | `ec6befc0f50b615f5eb8d7c7c51e68229fd1ba7954ccc93dc7b005426c840e7b` |
| `v8-javac8/RK$Item.class` | 305 | `9704723fdc313d8e5b3fb16c6229e09782afa9a6b4acf3374ba15f876897af11` |
| `v8-javac8/RL.class` | 1,099 | `9a2aae3e833c6aec636c68fcecb277ffa8f4c5335d021120b70efda8df691c6b` |
| `v8-javac8/RL$Inner.class` | 305 | `382e754c073de12038286bc811b5635d37d8ca3e8ef57b2af8494018e67f8fc6` |
| `v8-javac8/RM.class` | 1,181 | `5b7ede176972cf791a97b2cb51e913103bd060cd80b7c86878868ea3a2f0ad18` |
| `v8-javac8/RM$Item.class` | 404 | `01221596d9cb356fc1bf76bb49b64b81bbdf260d676acca5be07b18f6091de95` |
| `v8-javac8/RN.class` | 1,262 | `49f5bdd8f37a1ce402c4353e51ced2df539c0ac5b130f6aaaa3dd0603b56b01d` |
| `v8-javac8/RN$Item.class` | 303 | `a03c9b19037f25291080e335694f28de358e5aba18b2073b5fab1263637bcbb9` |
| `v8-javac8/RO.class` | 899 | `e9f1b6a68d0977d251a939163e0bb69667241d88b49a5df1153190095bd920a5` |
| `v8-javac8/RO$Item.class` | 305 | `5bc896120ebdf40b82be77d0d2300772068edf38f4301ec7c60e224da3de3bcb` |
| `v8-javac8/RP.class` | 1,159 | `4a239b4dc04a2a3bf73ef48961b65151b3b55fd4584c7dcb1c418d5d5cef42f7` |
| `v8-javac8/RP$Item.class` | 305 | `e5a90492f13058bb51e82a0b7e80c9c9d84cddf2d8fe2434349e49700ec7cf6f` |
| `v8-javac8/RP$Sub.class` | 252 | `551d2fb6194ee32cc09c69dddb54b193234061072658c2b9cba6c9b9aa72ab33` |

## The discriminating facts (javap, `v8` leg; both legs share the BCIs)

`RG.<clinit>` — the array comes from a **local holding a created array**: `47: anewarray RG$Item …
68: astore_0; 69: aload_0; 70: astore_1`; the enhanced-for body is
`81: aload_1; 82: iload_3; 83: aaload; 84: astore 4; 86: getstatic BY_LABEL; 89: aload 4;
91: getfield RG$Item.label; 94: aload 4; 96: invokeinterface Map.put; 101: pop`. BCI 91 is the
field identity proof's own site: before this change the whole body was quoted and the stripped text
read `null/null/null` where the class reads two `Item`s and `null`.

`RK.viaElemLocal` — `0: aload_0; 2: aaload; 3: astore_1; 4: aload_1; 5: getfield label`: the element
held in an explicitly typed local. `RK.viaElemDirect` reads the element directly.

`RH.total` — the array is a **parameter** (`aload_0; astore_2`), the element is `19: aaload;
20: astore 5`, and the field read is `25: getfield RH$Item.label` inside the accumulation.
`RH.first` is `arg0[0].label`.

`RL.selfElem`/`RL.innerElem` — the same element-receiver shape over the current class's own array
and over a companion class's array: the component being the current class changes nothing.

`EM.<clinit>` — the array is the **call result** `values()` (a pooled return of `$VALUES`), and the
loop body is `EM.BY_LABEL.put(c.label, c)`. The enum constants themselves render as blank
declarations (`public static final EM A;`), the constant-pool debt this change does not touch.

`RP.merged` — `z = b ? subs : items` with `RP$Sub[]` and `RP$Item[]`: the two frames merge to the
conservative unknown reference, `12: aaload`'s array operand states no component, and `13: getfield`
keeps the field identity refusal verbatim. `RP.branchy` assigns the same two arrays in two branches;
the merge is a phi, so the field read is refused by the same link while the two assignments are
written as ordinary statements.

## The tests that read them

`tests/recover_array_element_field_receiver.rs`:

- not ignored: both legs render every fixture once and pin the members' whole texts (the recovery
  anchors, the zero-regression controls, the two refusals), plus BCI 86/91/94/96 as anchors of `RG`'s
  recovered statement and `RecoveryContent` for the refused members;
- ignored (`cargo test -- --ignored`, needs a JDK on `PATH`): every stripped text is compiled with
  `javac --release 8` and run with `-Xverify:all`, and compared with the fixture's own class files'
  run — `RG` by the shape of its three lookups (`non-null/non-null/null`), every other fixture by the
  exact output (`5/xy`, `q/q`, `s/i`, `5/2`, `d/d`, `m/m`, `X/2`). `EM`'s stripped text must stay
  uncompilable while its constants render as blank declarations.
