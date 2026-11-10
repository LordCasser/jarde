# Static and raw generic member controls R1

Frozen Java 8 fixture for `Box<T>` member binding. `Box<T>` declares instance `T value`, instance `List<String> fixed`, method `<U> U echo(U)`, static `List<String> fixedStatic`, static `List<String> passthrough(List<String>)`, and static `<U> U staticEcho(U)`. `RawChildBox extends Box` and `StringChildBox extends Box<String>` provide raw and parameterized inherited receiver paths. `BoxConsumer` contains one small caller per use.

The physical declarations were verified with `Box.javap.txt` and `BoxConsumer.javap.txt`. In particular, value/echo descriptors erase to Object, fixed/static list members erase to List, and static passthrough erases to `(List)List` while its Signature retains `List<String>`. No target method is executed.

Compilation used Corretto 8 `javac -source 8 -target 8 -g`, with only the frozen Foo fixture on the classpath. `pins.json` records source, class, and javap hashes.
