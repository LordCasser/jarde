# CF-19 implementation replay

The frozen original class and complete frozen JADX class, plus the complete class emitted by the rebuilt Jarde CLI, were each paired with `SynchronizedMultiExitRunner.java`, compiled using `javac --release 8 -g:none`, and run with `java -Xverify:all`. The three outputs in this directory match byte-for-byte. The CLI was run both with default evidence and `--evidence all`; their text outputs compared equal.

The two-line source diff adds only the proven conditional structure inside the existing monitor body. The method's selected producer executes once, and the runner confirms both return values, call trace, exception class, and exception object identity.
