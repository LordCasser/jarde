# Mixed short-circuit value as a call argument

Compile `MixedBooleanArgument.java` with `javac --release 8 -g:none`. `Runner.java` covers all eight Boolean inputs under `java -Xverify:all`, recording the field value and all three call counters. The class file is the frozen subject; `Runner.class` is intentionally not retained.
