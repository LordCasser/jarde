// jarde: presentation of `NAnchor` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NAnchor extends java.lang.Object {
    public NAnchor() {
        // @method <init>()V
        // @declaration a constructor of `NAnchor`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static NAnchor$M pick(NAnchor$M arg0) {
        // @method pick(LNAnchor$M;)LNAnchor$M;
        // @declaration a static method of `NAnchor`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        NAnchor$M local1 = arg0;
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NAnchor`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the saved producer at BCI 0 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 3
        // the saved producer at BCI 3 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 19 0 3
        // the value at BCI 19 was produced by a saved declaration this run could not commit
        return;
    }
}
