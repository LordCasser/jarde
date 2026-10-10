# Unused byte-array initializer baseline verifier v1

This is an independent, read-only verifier for `baseline-root-v1`; it does not launch Java, JADX, Jarde, Cargo, or Git, and it does not import or execute the collector. It verifies the frozen manifest and closed 127-file inventory, the 35 archived commands and raw streams, the pinned JDK/JADX/Jarde binaries, exact fixture inputs and complete class sets, the four Jarde reports and their owner/source-map facts, and all ten archived `-Xverify:all` raw runs.

The collector's original-case `success` flags are knowingly false because its copied `physical_members_exact` assumption expects the wrong test descriptor. The verifier independently checks archived `javap` declarations and bytecode and requires the actual physical method set to be `<init>()V` and `test()V`, with no fields. It also requires the exact three recorded collector failures and the two original false success flags, while separately confirming original compile/runtime/class-set success. It therefore reports `accepted-baseline-with-recorded-collector-error`, not ten collector success flags.

The source-presentation assertion is limited to the exact local initializer text: Jarde emits `byte[] local1 = new byte[]{10, 20, 30};`; JADX emits `byte[] bArr = {10, 20, 30};`. Runner's `done` output establishes only that the complete recovered class compiles and runs under the archived oracle; it cannot establish the local array's values or retention semantics. The explicit `new byte[]` form is valid Java and is not recorded as a production defect or change request.

The verifier has not been executed. Only Python syntax compilation is permitted for this preparation; it does not certify the archived evidence.
