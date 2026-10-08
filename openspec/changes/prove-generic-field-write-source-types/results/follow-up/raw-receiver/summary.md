Raw receiver generic-field source audit

Input: source/RawOtherWriter.java
Original javac --release 8: exit 0; reports unchecked raw member assignment.
Baseline class-source CLI: exit 0; field v is `public T v` with projected Signature.
Final class-source CLI: exit 0; field v falls back to `public java.lang.Object v` with `field_generic_write_source_unproved` at put(...)@2.
Baseline recovered source javac --release 8: exit 0, unchecked assignment warning.
Final recovered source javac --release 8: exit 0, no unchecked assignment warning.
Bytecode: putfield v:Ljava/lang/Object;; receiver descriptor is raw LRawOtherWriter; RHS is Object.

Interpretation: receiver is raw, so javac sees raw field selection `raw.v` as Object and permits Object assignment (unchecked warning). Current write proof checks Object RHS against class-scope T as if the receiver were parameterized; it conservatively refuses the field Signature. This is a source-compatibility capability gap, but RawOtherWriter is not one of the frozen positive fixtures inspected in the change. Existing listed positives cover this/same-class receiver, null, direct matching generic parameter, deferred same-class writer, raw RHS assignments, and direct raw allocations. So it is not currently a frozen-fixture blocker; it would need a separately scoped raw-receiver/member-selection proof if desired. Do not describe the raw-receiver case as restored.

Files: baseline/RawOtherWriter.java, final/RawOtherWriter.java, class-source.diff, javap.txt, javac-original.stderr, javac-baseline.stderr, javac-final.stderr.
