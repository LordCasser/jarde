// jarde: presentation of `CharProducerIntOverload` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CharProducerIntOverload extends java.lang.Object {
    public CharProducerIntOverload() {
        // @method <init>()V
        // @declaration a constructor of `CharProducerIntOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String render(java.lang.String arg0) {
        // @method render(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `CharProducerIntOverload`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        char local1 = arg0.charAt(0);
        local1 = '.';
        return new java.lang.StringBuilder().append((int) local1).toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `CharProducerIntOverload`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) render("x"));
        return;
    }
}
