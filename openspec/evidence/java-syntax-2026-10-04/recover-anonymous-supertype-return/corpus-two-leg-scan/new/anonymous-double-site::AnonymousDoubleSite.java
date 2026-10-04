// jarde: presentation of `AnonymousDoubleSite` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class AnonymousDoubleSite extends java.lang.Object {
    public AnonymousDoubleSite() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousDoubleSite`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static DoubleBase create(boolean first) {
        // @method create(Z)LDoubleBase;
        // @declaration a static method of `AnonymousDoubleSite`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (first) {
            return new AnonymousDoubleSite$1();
        } else {
            return new AnonymousDoubleSite$1();
        }
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousDoubleSite`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        DoubleBase first = create(true);
        DoubleBase second = create(false);
        java.lang.System.out.println("first=" + first.value());
        java.lang.System.out.println("second=" + second.value());
        // @bytecode 66
        // the saved producer at BCI 66 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 69 72 73
        // the saved producer at BCI 73 has no bounded final expression consumer
        // @bytecode 69 72 73 78
        // the saved producer at BCI 78 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 103 66 100 97 78 73 69 72
        // the value at BCI 103 was produced by a saved declaration this run could not commit
        return;
    }
}
