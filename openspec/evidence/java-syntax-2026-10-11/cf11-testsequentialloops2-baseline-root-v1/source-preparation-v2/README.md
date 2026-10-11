# TestSequentialLoops2 input fixture (v2)

This JUnit-free fixture keeps only one top-level class, `UpstreamSequentialLoops2`, containing the original field and method. There is no additional owner class. The field declaration and `asciiToLowerCase(String)` method are copied byte-for-byte, including their original indentation; only the package and enclosing class name differ from upstream. `lowercases` remains exactly `private static char[] lowercases = new char[] { 'a' };`.

`Runner` calls the original static method directly for each case. It prints successful results as four-digit uppercase UTF-16 code units separated by spaces, or the caught exception's fully qualified class name. Cases: empty, allNonASCII, NUL, NULNUL, U+0080NUL, NULU+0080, `a`, aNUL, NULa, and null.

Upstream source: `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestSequentialLoops2.java`, SHA-256 `adb34f80f7d55cd3309ed7708f0fdb46768cf32e4004dbd13d4e278530306720`.

The pins use zero-based UTF-8 byte offsets with exclusive end offsets:

- Field: `[330, 386)`, SHA-256 `3209860a1335c2128aa1a82e3e5668cd15580a18d7ae07c1407a36a3bd5801fd`.
- Method `public static String asciiToLowerCase(String s)`: `[388, 808)`, SHA-256 `80c97a42c86da19956b70d9626219b7f456d277d5e2a3a0e580fa7586f6ea577`.

Static expectations from source, not runtime results: empty → empty; allNonASCII → U+0080; NUL → `0061`; NULNUL → `0061 0061`; U+0080NUL → `0080 0061`; NULU+0080 → `0061 0080`; `a`, aNUL, and NULa → `java.lang.ArrayIndexOutOfBoundsException`; null → `java.lang.NullPointerException`.

For `while (i-- > 0)`, the old `i` is compared, then `i` decreases on both successful and failed condition checks; the body uses the decreased index. Normally the second loop runs only after the first loop breaks. All-non-ASCII input exhausts the first loop with `i == -1`, so the second loop immediately fails its condition and never dereferences null `c`; the method returns the original string.

The earlier v1 private fixture was not run. This v2 only removes its extra nested `Owner` entity and the corresponding unused instance creation; the two pinned source slices are unchanged. This v2 fixture and runner have not been compiled or run. The expectations above remain to be confirmed against a real runtime oracle.
