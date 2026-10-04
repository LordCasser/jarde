// jarde: presentation of `InterfaceSelfInvocation` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class InterfaceSelfInvocation extends java.lang.Object {
    public InterfaceSelfInvocation() {
        // @method <init>()V
        // @declaration a constructor of `InterfaceSelfInvocation`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static Renderer create() {
        // @method create()LRenderer;
        // @declaration a static method of `InterfaceSelfInvocation`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new InterfaceSelfInvocation$1();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `InterfaceSelfInvocation`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.Object) create());
        return;
    }
}
