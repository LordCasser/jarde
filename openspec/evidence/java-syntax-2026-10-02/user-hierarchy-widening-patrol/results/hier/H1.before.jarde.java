// jarde: presentation of `H1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class H1 extends java.lang.Object {
    public H1() {
        // @method <init>()V
        // @declaration a constructor of `H1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String via(H1$Greet arg0, java.lang.String arg1) {
        // @method via(LH1$Greet;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `H1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.hello(arg1);
    }

    static java.lang.String viaOther(H1$Other arg0) {
        // @method viaOther(LH1$Other;)Ljava/lang/String;
        // @declaration a static method of `H1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return "tag:" + arg0.tag();
    }

    static java.lang.String lead(java.lang.String arg0, H1$Greet arg1) {
        // @method lead(Ljava/lang/String;LH1$Greet;)Ljava/lang/String;
        // @declaration a static method of `H1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 + ":" + arg1.hello("x");
    }

    static java.lang.String viaObject(java.lang.Object arg0) {
        // @method viaObject(Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `H1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return "obj";
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `H1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 15 0 12
        // the parameter 0 of the invocation at BCI 12 is declared `H1$Greet` presents `H1$TwoLevel` but the invocation requires `H1$Greet` and this layer has no safe reference conversion evidence
        // @bytecode 33 18 30
        // the parameter 0 of the invocation at BCI 30 is declared `H1$Greet` presents `H1$Multi` but the invocation requires `H1$Greet` and this layer has no safe reference conversion evidence
        // @bytecode 49 36 46
        // the parameter 0 of the invocation at BCI 46 is declared `H1$Other` presents `H1$Multi` but the invocation requires `H1$Other` and this layer has no safe reference conversion evidence
        // @bytecode 67 52 64
        // the parameter 0 of the invocation at BCI 64 is declared `H1$Greet` presents `H1$1` but the invocation requires `H1$Greet` and this layer has no safe reference conversion evidence
        // @bytecode 85 70 82
        // the parameter 0 of the invocation at BCI 82 is declared `H1$Greet` presents `H1$ViaSub` but the invocation requires `H1$Greet` and this layer has no safe reference conversion evidence
        // @bytecode 103 88 100
        // the parameter 1 of the invocation at BCI 100 is declared `H1$Greet` presents `H1$Multi` but the invocation requires `H1$Greet` and this layer has no safe reference conversion evidence
        // @bytecode 126 106 123
        // the parameter 0 of the invocation at BCI 123 is declared `H1$Greet` presents `H1$TwoLevel` but the invocation requires `H1$Greet` and this layer has no safe reference conversion evidence
        java.lang.System.out.println((java.lang.String) viaObject((java.lang.Object) new H1$Fin()));
        return;
    }
}
