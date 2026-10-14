# Focused ordinary try admission fixture

`OrdinaryTryListFixture.java` is an independent complete class for positive and negative
source-proof cases. The rename-only `renamedTry`, String/Integer branches, nested foreach, and
`remove(Object)` prefix follow the corresponding ordinary-try topologies in the frozen baseline
source; `stringBranch` adds the direct catch-owned `List.add(null)` case. Its Runner executes
normal, empty, null-list, and null-element inputs plus RuntimeException/Error injections at
`iterator`, `hasNext`, `next`, `add`, and prefix removal. Unknown catch calls, reassignment, raw
aliases, multi-catch, and index-overload controls remain in the class as negative cases.
`catchVisibleIteratorGap` is an additional negative control: it writes a marker after the explicit
`Iterator` initializer and before the first `while` test, then reads that marker in the catch. The
Runner covers normal completion and injected RuntimeException/Error from `iterator()`; the caught
RuntimeException must observe marker 0, while Error escapes the RuntimeException handler.

The source and Runner are byte-pinned by `manifest.json`. `tools/replay_admission.sh` (in the
change's tools directory) reuses the complete-class collector. Run it serially with `debug` and
`none`, each in a fresh output directory; each invocation covers JDK 8/23 and Jarde default/all.

Root compiled the committed class with Amazon Corretto 8.504.04.1 using `javac -source 8
-target 8 -g -proc:none -d <fresh-directory> OrdinaryTryListFixture.java Runner.java`, then ran the
complete original Runner with `java -Xverify:all`. The class is 5,457 bytes with SHA-256
`bb1f948667421ba7ee49ef38d1af7c9d38aff20d015f877bc85ef26f2e851ffe`. Source pins are 5,432 bytes /
`4b548725ec08c31bcbb09b31a22a3e54c74481b3e7773eb8c9fe2c66c5b155f2` and 10,678 bytes /
`dee1e694ee91c71806bfeea4609876d11d3331304e469300c2247ea6c2e1fd98` for the class and Runner.
The complete original produces 33 lines, 3,822 stdout bytes (SHA-256
`599122bdd1c6fa460a074f2a6e1117a7211772492a932a73ab980afdbc68de6d`) and empty stderr. This
pins the original's behavior; acceptance of generated source requires the separate full-class
replays and independent audit.
