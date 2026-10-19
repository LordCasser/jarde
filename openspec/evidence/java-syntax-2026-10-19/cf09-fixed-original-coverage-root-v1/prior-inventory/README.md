# CF-09 fixed-source baseline coverage inventory

Read-only source/evidence review. The later explicit fixed inventory in `control-flow.md` (line 136) lists 13 test files. The earlier category table row (line 69) lists five different files and includes `TestSequentialLoops`; this memo follows the requested 13-file inventory and treats `TestSequentialLoops` as CF-11. Three have accepted full-original baselines: `TestBreakInLoop`, `TestBreakWithLabel`, and `TestContinueInLoop`. `TestDoWhileBreak` has a private preparation bundle only; it has no execution receipt. `TestSequentialLoops` is separate CF-11 evidence and should not be counted as one of the 13.

The accepted markers report distinct states: BreakInLoop is a full original baseline with no new mechanism warranted (94 commands); BreakWithLabel preserves a known Jarde boolean-local stdout gap (94 commands); ContinueInLoop preserves a whole-method refusal gap (46 commands). The older Grid/OuterContinue replay is not evidence for these full fixed classes.

Among uncovered candidates, `TestBreakInComplexIf` is the recommended next target: its public `check()` invokes the target with two bounded maps, covering both normal completion and a break caused by the nested condition. It avoids external filesystem state, the sleep path in BreakInLoop2, the disabled test in BreakInLoop3, the Smali-only BreakInLoop6, and the uninitialized iterator field in DoWhileBreak3. Other finite checks are documented in `review.json`; they were not executed.

`review.json` records SHA-256 and sizes for each of the 13 exact upstream Java test files, evidence pins, method inventory, and scope limitations. No shared files or evidence were changed. No JDK, Cargo, CLI, JADX, Git, or network command was run.
