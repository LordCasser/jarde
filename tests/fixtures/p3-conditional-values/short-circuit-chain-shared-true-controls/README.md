# Three-test shared-true duplicate-consumer control

`ChainOrFieldDuplicatePhi.assign(ZZ)V` is compiled from the chained assignment
`result = mirror = extra || left || rhs()`. Its three tests share the true
producer at BCI 14 and its stack Phi feeds two `putstatic` instructions at BCI
20 and 23. This control isolates the duplicate-consumer boundary while keeping
the shared-true OR chain produced by `javac --release 8`.

Rebuild and verify the frozen class from the repository root:

```sh
FIXTURE=tests/fixtures/p3-conditional-values/short-circuit-chain-shared-true-controls
TMP=$(mktemp -d /tmp/jarde-chain-controls.XXXXXX)
javac --release 8 -g:none -d "$TMP" "$FIXTURE/ChainOrFieldDuplicatePhi.java"
cmp "$TMP/ChainOrFieldDuplicatePhi.class" "$FIXTURE/ChainOrFieldDuplicatePhi.class"
javac --release 8 -g:none -cp "$TMP" -d "$TMP" "$FIXTURE/VerifyChainControls.java"
java -Xverify:all -cp "$TMP" VerifyChainControls
javap -classpath "$TMP" -c -v -p ChainOrFieldDuplicatePhi
```

Expected output:

```text
extra=true,left=false,rhs=false,result=true,mirror=true,calls=0
extra=false,left=true,rhs=false,result=true,mirror=true,calls=0
extra=false,left=false,rhs=true,result=true,mirror=true,calls=1
extra=false,left=false,rhs=false,result=false,mirror=false,calls=1
```
