// jarde: presentation of `Main` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class Main extends java.lang.Object {
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
        Base local1 = Owner.one();
        Base local2 = Other.two();
        // @bytecode 8
        // the saved producer at BCI 8 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 11 14 15
        // the saved producer at BCI 15 has no bounded final expression consumer
        // @bytecode 11 14 15 20
        // the saved producer at BCI 20 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 45 8 42 39 20 15 11 14
        // the value at BCI 45 was produced by a saved declaration this run could not commit
        return;
    }
}
