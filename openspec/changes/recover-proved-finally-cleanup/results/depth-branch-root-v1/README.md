# CF16 nested-branch depth probe preparation

`prepare-depth-branch-root-v1.py` prepares two complete Java 8 source/class observations at
`run/`: `BranchFinally` with two nested if/else levels and the same class with thirty-three
levels. Each `if` tests `x > depthIndex`; each true branch adds its one-based depth weight before
entering the next if, and each else subtracts its depth weight. Both versions put a single
`return x` inside one try and a single
`cleanup()` call in its finally. The complete Runner resets `trace` between `run(0)` and
`run(40)`, asserts each result and trace, and prints the calculated output.

When explicitly run, the script verifies the returned-array baseline manifest, frozen CLI v2
identity, and both JDKs' `java`/`javac`/`javap` hashes. It compiles and verifies only each original
complete class plus Runner with empty classpath/sourcepath, then requests fresh `class-source`
`all` JSON from the frozen CLI. It saves command argv/exit/raw streams, full source, class files,
CLI documents, extracted run-body text and the method's actual outcome/stop/produced/diagnostic
fields. It does not compile or execute any recovered text. The final `file-inventory.json` includes
all files under `run/` except itself.

This is preparation only. No JDK, frozen CLI, Git, Cargo, or project command has been run while
writing this probe. The depth-33 report must be read as observed; the script makes no claim that it
reaches a hard recursion bound.

After review, root can prepare observations with:

```sh
python3 openspec/changes/recover-proved-finally-cleanup/results/depth-branch-root-v1/prepare-depth-branch-root-v1.py
```
