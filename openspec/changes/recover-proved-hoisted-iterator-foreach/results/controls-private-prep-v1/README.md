# Hoisted iterator controls (private source preparation)

These are two standalone Java 8 target classes plus the collector-format Runner input. They are controls for the inner-iterator/hoisted-iterator change; no source was compiled or run.

`HoistedIteratorPositive.sumNested` has nested `List<String>` enhanced-for loops, accumulation, and a counted body call. Its inner list is iterated again for each outer value. Runner exercises 2×2 traversal, empty outer, empty inner with repeated outer entry, null outer/inner, iterator/hasNext/next exceptions at both inner and outer phases, and a non-String element that triggers the erased `String` cast at the loop consumer. Runner itself is the test list/iterator: it extends raw `AbstractList`, implements `Iterator`, and each independently constructed instance records its own iterator phases into one shared trace. This avoids extra generated support-class files, anonymous classes, and nesting while still giving the collector exactly its two targets plus `cf11.Runner`.

`HoistedIteratorRefusal.sumWithWrittenGap` keeps an observable `seen = 1` write after `values.iterator()` and before the while condition. Its catch returns `seen`, so the runner’s iterator-construction exception observes 0 before that write, while a later `hasNext` exception observes 1. This is the ordering-sensitive refusal probe for the new hoisted path. Any older ordinary-helper ordering concern remains a separate debt and is not asserted by this input.

The manifest uses the existing controls collector shape: two top-level target classes and one runner, each with exact source path, byte count, and SHA-256. The whole-class compile must use these unchanged source files. Root should freeze a byte-identical copy, preserving the manifest and pins; no predicted rendering, runtime transcript, or expected refusal result is claimed as observed evidence.

No shared files were modified. No Git, Cargo, Java/JDK, JADX, network, or CLI command was used. Static checks here only parsed the manifest and checked expected top-level class declarations/source pins.
