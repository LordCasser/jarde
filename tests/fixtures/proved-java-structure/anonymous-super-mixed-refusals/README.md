# Mixed-parameter anonymous refusals

Six frozen negative inputs for `recover-anonymous-mixed-super-capture`: each must keep the
physical class text because the proved parameter partition (or the one-field / one-allocation
shape) refuses it. Two are javac-source families; four are minimal classfile patches of the
`anonymous-super-mixed-direct` anchor child, derived exactly as the CI test
`anonymous_superclass_refuses_unproved_mixed_parameter_roles` derives them in memory. The
evidence-side derivation script (it patches, verifies with `java -Xverify:all`, and records
before/after presentations) is
`openspec/evidence/java-syntax-2026-10-04/recover-anonymous-mixed-super-capture/negative-derivations.py`.

| input | shape | expected refusal |
| --- | --- | --- |
| `two-capture-fields/` | one super-argument pair plus **two** captured locals → two `val$` fields, ctor `(String,int,String,String)` | one-field shape (`anonymous_super_child_shape_unproved`) |
| `two-mixed-sites/` (patched root: the second site's class constant names the first child) | two direct-return allocations of **one** physical class | single-allocation proof cannot close (no site, physical text) |
| anchor child, capture store nop'ed out | the third constructor parameter has **no consumption** | parameter role partition |
| anchor child, `aload_1` → `aload_3` at the invoke's first argument | one parameter feeds **both** the capture store and a super argument | parameter role partition |
| anchor child, leading arguments swapped and the invoked descriptor rewritten to `(ILjava/lang/String;)V` | super arguments leave the physical parameter order (the second `Base` constructor keeps the class runnable) | super arguments are not the leading physical parameters in order |
| anchor child, closing `return` replaced by a second `putfield`+`return` | the capture field is **stored twice** | one constructor write |

Every input runs under `java -Xverify:all` (the originals are valid, runnable classes — the
run logs sit beside the derivation script) and every Jarde rendering keeps `new …$1(…)`
physical text instead of a projected `new Base(…) { … }`.
