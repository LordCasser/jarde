# Anonymous class referenced across owner classes

`Owner.one()` and `Other.two()` are separate Java 8 source sites, each declaring `new Base() { ... }`. The methods are static, so both anonymous constructors have descriptor `()V`. `Main` compares their runtime classes.

`freeze.py` recompiles the untouched source with `javac --release 8 -g:none`, confirms the source execution prints `sameClass=false`, changes exactly the `Other$1` UTF8 constant in `Other.class` to the equal-length `Owner$1`, then runs the mutated class set with `java -Xverify:all` and confirms `sameClass=true`. It preserves both anonymous class files and writes the mutated class set plus SHA-256 hashes here. The frozen set is a bytecode mutation fixture, not a source-equivalent build.
