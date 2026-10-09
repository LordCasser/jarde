// jarde: presentation of `NullThenBuilder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NullThenBuilder extends java.lang.Object {
    public NullThenBuilder() {
        // @method <init>()V
        // @declaration a constructor of `NullThenBuilder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.Object run(boolean arg0) {
        // @method run(Z)Ljava/lang/Object;
        // @declaration a static method of `NullThenBuilder`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        Object local2;
        local1 = 0;
        local2 = null;
        if (arg0) {
            if (local2 == null) {
                local1 = local1 + 1;
            }
        }
        local2 = new java.lang.StringBuilder("second");
        local1 = local1 + local2.length();
        return java.lang.Integer.valueOf(local1);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NullThenBuilder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.Object) run(true));
        return;
    }
}
