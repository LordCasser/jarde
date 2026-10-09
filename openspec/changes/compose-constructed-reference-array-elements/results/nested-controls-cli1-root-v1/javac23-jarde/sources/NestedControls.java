// jarde: presentation of `NestedControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NestedControls extends java.lang.Object {
    public NestedControls() {
        // @method <init>()V
        // @declaration a constructor of `NestedControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String mark(java.lang.String arg0) {
        // @method mark(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `NestedControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print("mark:");
        java.lang.System.out.println(arg0);
        return arg0;
    }

    static java.lang.Object[] nested() {
        // @method nested()[Ljava/lang/Object;
        // @declaration a static method of `NestedControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Object[]{new java.lang.StringBuilder((java.lang.CharSequence) new java.lang.StringBuilder((java.lang.String) mark("nested")))};
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NestedControls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object[] local1 = nested();
        java.lang.Object local2 = local1[0];
        java.lang.System.out.println(local2);
        return;
    }
}
