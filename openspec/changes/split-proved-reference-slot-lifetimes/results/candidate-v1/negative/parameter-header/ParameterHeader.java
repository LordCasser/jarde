// jarde: presentation of `ParameterHeader` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ParameterHeader extends java.lang.Object {
    public ParameterHeader() {
        // @method <init>()V
        // @declaration a constructor of `ParameterHeader`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.Object rewrite(java.lang.Object arg0, boolean arg1) {
        // @method rewrite(Ljava/lang/Object;Z)Ljava/lang/Object;
        // @declaration a static method of `ParameterHeader`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        arg0 = "first";
        arg0 = new java.lang.StringBuilder("second");
        return arg1 ? arg0 : arg0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ParameterHeader`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) rewrite(new java.lang.Object(), true).getClass().getName());
        return;
    }
}
