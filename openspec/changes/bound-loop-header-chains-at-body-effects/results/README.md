# Completed acceptance: 6/6

Product `c8af822ef6d929f8ad1221f58d02e27669c63fc7` preserves a later effectful AND body using the existing loop/body/If/break ownership path. No new Region/pass/CFG/ForHeader/Builder mechanism is introduced.

- [Same-run original IR](same-run-ir-root-v1/README.md): real boundary and exact temporary-source restoration.
- [Focused and full primary comparison](focused-and-full-comparison-root-v1/README.md): 16 APIs, eight JDK23 full source-pair compiles, 144 equivalent observations; target map32/33, Point12/12.
- [Second fixed original](../../../evidence/java-syntax-2026-10-19/cf09-testbreakincomplexif2-fixed-root-v1/README.md): same product, eight Structured APIs, eight JDK23 full compiles and 176 equivalent observations; map33/34.
- [Fresh workspace/current-product own CI](native-and-own-ci-root-v1/README.md): 392 targets/3816-0-98,71 native gates; four jobs/52 official steps/two seeds116 gates and all3914 named outcomes;5263 Git-bound source paths. Main-spec1 requirement/4 scenarios and all385 strict checks passed.

JDK8 original-framework boundaries and the two implicit-transfer provenance gaps remain explicit. [Mapping debt](../../../evidence/java-syntax-2026-10-19/cf09-implicit-loop-transfer-mapping-debt-root-v1/README.md) is separate. [Next Loop2 baseline](../../../evidence/java-syntax-2026-10-19/cf09-testbreakinloop2-original-root-v1/README.md) confirms a different ownership gap; its [trace preparation](../../../evidence/java-syntax-2026-10-19/cf09-testbreakinloop2-ir-probe-preparation-root-v1/README.md) is unexecuted and adds no current product code. CF09/71 units/612 files remain partial.
