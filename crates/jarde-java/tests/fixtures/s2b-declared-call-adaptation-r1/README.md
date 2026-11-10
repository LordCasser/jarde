# Public declared-call adaptation controls R1

Frozen Corretto 8 Java 8 input with three methods: `typedAdd(List<Foo>, Foo, int) -> Foo`, `genericForward(List<Foo>, List<Foo>, int) -> void`, and `rawWrite(List<Foo>, Object) -> void`.

`javap` confirms `typedAdd` has its original `checkcast Foo` after `List.get`; `genericForward` passes the erased `Object` returned by `List.get` directly to `List.add` without a Foo checkcast; and `rawWrite` writes the Object argument through a raw List local. This is an API/component control, not a complete Engine class recovery or runtime execution.

The first Java source attempt declared `typedAdd` as returning `List<Foo>` while its final expression is `local.get(index)` (a Foo). Corretto 8 rejected that source; the initial source and diagnostic are preserved. The frozen successful source returns Foo, matching the expression and requested behavior. `pins.json` includes both the rejected attempt and final compiled bytes.
