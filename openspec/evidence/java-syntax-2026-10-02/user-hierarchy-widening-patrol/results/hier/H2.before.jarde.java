// jarde: presentation of `H2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class H2 extends java.lang.Object {
    public H2() {
        // @method <init>()V
        // @declaration a constructor of `H2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String takeT(H2$Target arg0) {
        // @method takeT(LH2$Target;)Ljava/lang/String;
        // @declaration a static method of `H2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return "t:" + arg0.tag();
    }

    static void sink(java.lang.Throwable arg0) {
        // @method sink(Ljava/lang/Throwable;)V
        // @declaration a static method of `H2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("sinkT:" + arg0.getMessage());
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `H2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 13 0 10
        // the parameter 0 of the invocation at BCI 10 is declared `H2$Target` presents `H2$Ext` but the invocation requires `H2$Target` and this layer has no safe reference conversion evidence
        // @bytecode 25
        // the parameter 0 of the invocation at BCI 25 is declared `java.lang.Throwable` presents `H2$MyErr` but the invocation requires `java.lang.Throwable` and this layer has no safe reference conversion evidence
        return;
    }
}
