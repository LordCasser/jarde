// jarde: presentation of `NLocal` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NLocal extends java.lang.Object {
    public NLocal() {
        // @method <init>()V
        // @declaration a constructor of `NLocal`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String use(NLocal$Fn arg0) {
        // @method use(LNLocal$Fn;)Ljava/lang/String;
        // @declaration a static method of `NLocal`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        NLocal$Fn local1 = arg0;
        return local1 == null ? "n" : local1.apply("hi");
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NLocal`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) use((NLocal$Fn) null));
        return;
    }
}
