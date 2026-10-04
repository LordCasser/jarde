// jarde: presentation of `SupertypeReturnIndirect` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class SupertypeReturnIndirect extends java.lang.Object {
    public SupertypeReturnIndirect() {
        // @method <init>()V
        // @declaration a constructor of `SupertypeReturnIndirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static Top create() {
        // @method create()LTop;
        // @declaration a static method of `SupertypeReturnIndirect`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new SupertypeReturnIndirect$1();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `SupertypeReturnIndirect`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) create().tag());
        return;
    }
}
