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
        java.lang.System.out.println((java.lang.String) via((H1$Greet) new H1$TwoLevel(), "two"));
        java.lang.System.out.println((java.lang.String) via((H1$Greet) new H1$Multi(), "multi"));
        java.lang.System.out.println((java.lang.String) viaOther((H1$Other) new H1$Multi()));
        java.lang.System.out.println((java.lang.String) via((H1$Greet) new H1$1(), "anon"));
        java.lang.System.out.println((java.lang.String) via((H1$Greet) new H1$ViaSub(), "sub"));
        java.lang.System.out.println((java.lang.String) lead("lead", (H1$Greet) new H1$Multi()));
        java.lang.System.out.println((java.lang.String) new H1$Caller().call((H1$Greet) new H1$TwoLevel()));
        java.lang.System.out.println((java.lang.String) viaObject((java.lang.Object) new H1$Fin()));
        return;
    }
}
