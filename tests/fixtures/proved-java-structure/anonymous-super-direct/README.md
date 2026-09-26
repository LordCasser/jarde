# Anonymous superclass arguments without capture

`AnonymousSuperDirect.java` is a Java 8 source oracle for direct arguments to an anonymous subclass's superclass constructor. It deliberately has no lexical locals or enclosing-instance state captured by the anonymous body. `next()` increments an observable counter and returns 7 then 2; `Base.sum()` is asymmetric (`left * 2 - right`) so reversing the two constructor arguments changes the output. The `(int, long)` overload distinguishes the selected `(int, int)` constructor in the frozen bytecode.

Compile with `javac --release 8 -g:none`, then run with `java -Xverify:all -cp <classes> AnonymousSuperDirect`; expected output is `13:2`. `SHA256SUMS` freezes every physical class emitted by that compilation. `original-javap.txt` in the sibling evidence directory preserves the allocation BCI, ordered calls, and selected constructor descriptor.
