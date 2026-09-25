# Mixed local return of a short-circuit value

Compile `MixedLocalReturn.java` with `javac --release 8 -g:none`; `Runner.java` with `javac --release 8`. The original JVM trace covers all eight assignments of `a`, `bValue`, and `cValue`, with separate RHS call counters. The class file is the frozen subject; `Runner.class` is intentionally not retained.
