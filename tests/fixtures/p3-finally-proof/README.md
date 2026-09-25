# Bounded finally-copy proof fixtures

`TargetCopies.java`, `RepeatedCopies.java`, and `ExtraConsumer.java` are compiled with `javac --release 8 -g:none`. The committed class files let the guard unit tests inspect the same bytecode without requiring a JDK during Rust tests.

`TargetCopiesDifferentTarget.class` is a one-byte mutation of `TargetCopies.class`: in `run()`'s handler only, `invokestatic #13` (`first:()V`) becomes `invokestatic #16` (`second:()V`). Both constant-pool entries have the same descriptor, so the bytecode remains verifiable; the two cleanup effects differ. Regenerate it with:

```sh
javac --release 8 -g:none TargetCopies.java RepeatedCopies.java ExtraConsumer.java
python3 - <<'PY'
from pathlib import Path
source = Path('TargetCopies.class').read_bytes()
old = bytes.fromhex('b8 00 0d 2b bf')
assert source.count(old) == 1
Path('TargetCopiesDifferentTarget.class').write_bytes(source.replace(old, bytes.fromhex('b8 00 10 2b bf')))
PY
```

`RepeatedCopies.run()` has two cleanup invocations per copy and stays outside the one-call certificate slice.
`ExtraConsumer.run()` stores the cleanup call's result and reads it in another effect; that is outside the local stack-producer comparison slice.
