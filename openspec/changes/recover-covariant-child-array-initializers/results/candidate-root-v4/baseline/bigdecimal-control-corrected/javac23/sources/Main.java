// jarde: presentation of `Main` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Main extends java.lang.Object {
    public Main() {
        // @method <init>()V
        // @declaration a constructor of `Main`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 16 1 0 4 5 6 9 10 12 15
        // the array initializer element at BCI 16 is presented as `java.math.BigDecimal`, while the array component is `java.lang.Number`; this `aastore` has no compatible reference fact for a Java initializer
        // @bytecode 17 20 23 24 27 28 29 32 34 37 38 39 40 43 46
        // the statement at BCI 46 reads `local1`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        return;
    }
}
