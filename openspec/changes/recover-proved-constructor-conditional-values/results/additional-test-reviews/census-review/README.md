# Class fixture census candidate

`classfile-census.patch` changes only the census comment and expected tuple in `crates/jarde-reader/src/classfile.rs`. It adds the three frozen javac 23 `--release 8` class files named in `pins.json`: `UpstreamSequentialLoops2`, `ConditionalConstructionPositive`, and `ConditionalConstructionRefusals`.

The measured increment is 3 class paths, 21 methods with `Code`, 1 exception-table record, 34 branch/switch target records excluding handler targets, and 0 `jsr`/`jsr_w` instructions. The proposed total is `(1103, 4911, 469, 2987, 10)`. The exact definitions and measured class breakdown are in `analysis.md`; frozen class, javap, parser, capture, and source identities are in `pins.json`.

The existing ignored corpus-fingerprint updater must run after fixture installation to produce the new fingerprint. This preparation did not add fixture files, edit shared files, run the updater, or compile/run Rust, Java, JADX, or CLI.
