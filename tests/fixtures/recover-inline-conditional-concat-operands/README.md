# `recover-inline-conditional-concat-operands` fixtures

The inline conditional value as a `+` chain operand — `"" + a + (x == y) + b` — on **both** compiler
legs. Each class was compiled from the same source by javac 23.0.1 with `--release 8` and by real
javac 8 (Corretto 1.8.0_432); the two legs state the same shape (only constant-pool indices differ),
which is what makes the change's criterion a *bytecode* criterion and not a compiler's habit.

| leg | directory | command |
| --- | --- | --- |
| javac 23.0.1 | `v8/` | `javac --release 8 -g -nowarn -d v8 ICC.java ICB.java ICQ.java ICM.java ICN.java` |
| Corretto 1.8.0_432 | `v8-javac8/` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 ICC.java ICB.java ICQ.java ICM.java ICN.java` |

Debug information is kept (`-g`) so every local the presentation names is the source's own name.

## The anchors — the shape the change recovers

`ICC` — the reference-equality form of the patrol anchor `NI` (a nested class, a qualified-`this`
return and `==` against the outer instance):

```text
main:
    0: new ICC; 3: dup; 4: invokespecial; 7: astore_1
    8: getstatic System.out
   11: new StringBuilder; 14: dup; 15: invokespecial <init>()
   18: ldc ""            20: append(String)
   23: aload_1; 24: iconst_3; 25: invokevirtual make; 28: getfield v; 31: append(I)
   34: ldc "/"           36: append(String)
   39: aload_1; 40: iconst_2; 41: invokevirtual make; 44: invokevirtual outerTag; 47: append(I)
   50: ldc "/"           52: append(String)
   55: aload_1; 56: iconst_1; 57: invokestatic externalMake; 60: invokevirtual outerRef
   63: getfield tag; 66: append(I)
   69: ldc "/"           71: append(String)
   74: aload_1; 75: iconst_1; 76: invokestatic externalMake; 79: invokevirtual outerRef
   82: aload_1; 83: if_acmpne 90
   86: iconst_1; 87: goto 91
   90: iconst_0
   91: append(Z)          <- the join: the Phi's one consumer
   94: toString; 97: println; 100: return
```

The chain's head block **ends in the comparison** at BCI 83, the two arms push `1`/`0`, and the
chain continues in the join — so its `toString` (BCI 94) stands in another block. Before this
change `concat@1` refused the whole chain with `jre_concat_split` (the `toString` terminal in
another block) and the method was quoted; now the chain is presented in source form with the
comparison inlined:

```java
java.lang.System.out.println("" + local1.make(3).v + "/" + local1.make(2).outerTag() + "/" +
    externalMake(local1, 1).outerRef().tag + "/" + (externalMake(local1, 1).outerRef() == local1));
```

`ICB` — the same shape on a class of its own (`(n.self() == n)` as the last operand, with a
`getfield` operand in the middle of the chain); `ICQ` — the minimal form with no local reads at all
(`"eq=" + (p == q) + " tag=" + p.hashCode()`, the frozen patrol probe `CMP`).

## The control — the same chain **without** the branch

`ICM` — `"" + n.add(1) + "/" + n.add(2) + "/" + n.self().tag + "/" + n.tag`, i.e. the patrol probe
`NMA`. Its chain contains `getfield` operands, so `concat@1`'s ordinary walk refuses it
(`jre_concat_interleaved_effect` at the field read) and the fallback presents the builder chain
itself:

```java
java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("").append(n.add(1)).append("/").append(n.add(2)).append("/").append(n.self().tag).append("/").append(n.tag).toString());
```

That text is pinned byte-for-byte: a chain with no cut is not this change's shape, and the
certificate never sees it (it is entered only for a candidate the split attribution refused).

## The negatives — four shapes that keep their refusal

`ICN` holds one method per shape the change must **not** admit, all of them refused with
`jre_concat_split` naming the chain's own cross-block `toString` on both legs:

| method | shape | why the certificate refuses it |
| --- | --- | --- |
| `armCall` | `"" + pick(1) + (flag ? f() : g()) + "/" + pick(2)` | the cut is a zero test, and its arms **call**; an arm holds one constant and nothing else |
| `armStore` | `"" + pick(1) + (flag ? (y = 1) : (y = 2)) + "/" + pick(2)` | the arms **store**; same reason |
| `nested` | `"" + (p == q) + (r == s) + "/" + p.hashCode()` | the join block holds a **second** comparison: the chain does not continue as a value run |
| `guarded` | the chain inside a `try` whose `catch (RuntimeException)` covers it | the branch's block carries an **exceptional** edge, so the two-arm proof refuses the region |

## SHA-256 of the committed class files

```text
190270524b5d8178a94793a6a2fa0fbc78a6bc0ae04e5ccb7ffa5a28bea37df8  v8/ICB.class
10ec388e3ae9d8af34e91119d338f3da32f01e77d297b5777855416350088b6c  v8/ICC.class
bcf9d22235894fcbba2fd1f42ecbbfcfbd5b3eb3e6b0b45724d75b6a4e102095  v8/ICC$Inner.class
208631ed753485d7f38b515fc1b993140885d857c43ceed6b4c9be5cf72f852c  v8/ICM.class
3aba555d571f2c43264719dbc9a7d82f89db985e4744914c9e5c130b17b661b2  v8/ICN.class
444c6904d9ad40b97caa2bac74219274db912851df9db3e8159a652a817405fc  v8/ICQ.class
aa51bdeecaaac88b829a9c9d8c64177656d114e5e1145a83bfb05988bcf39108  v8-javac8/ICB.class
bd6855a81f46a6257fd5cff53d8bf789af168c1c37e22777e3f638f0498a967c  v8-javac8/ICC.class
e59ac4b0530798e17af581c5f9c56bc5675e99bc9dbf3388b9f6996b9e13b619  v8-javac8/ICC$Inner.class
83dfa35495c0dc8e0a966b775b66d636e624f67649ee7a2eca4d81361b927e65  v8-javac8/ICM.class
6dab49ef03ac07830db9aaea67a57857005b626153b51bbd90c81ff13643b14d  v8-javac8/ICN.class
a23eef6de98967fcc84d9256d3dfa82a8b5351c7db45ef534f47838fff68ba64  v8-javac8/ICQ.class
```
