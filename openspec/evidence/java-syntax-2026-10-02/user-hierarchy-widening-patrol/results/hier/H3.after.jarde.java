// jarde: presentation of `H3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class H3 extends java.lang.Object {
    public H3() {
        // @method <init>()V
        // @declaration a constructor of `H3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String via(H3$Sig arg0) {
        // @method via(LH3$Sig;)Ljava/lang/String;
        // @declaration a static method of `H3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return "d:" + arg0.tag();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `H3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) via((H3$Sig) new H3$L7()));
        // @bytecode 29 16 26
        // the parameter 0 of the invocation at BCI 26 is declared `H3$Sig` presents `H3$L8` but the invocation requires `H3$Sig` and this layer has no safe reference conversion evidence
        return;
    }
}
