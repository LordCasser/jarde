// jarde: presentation of `N2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class N2 extends java.lang.Object {
    public N2() {
        // @method <init>()V
        // @declaration a constructor of `N2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String take(boolean arg0) {
        // @method take(Z)Ljava/lang/String;
        // @declaration a static method of `N2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? "1" : "0";
    }

    public static java.lang.String ops(boolean arg0, boolean arg1) {
        // @method ops(ZZ)Ljava/lang/String;
        // @declaration a static method of `N2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return take(arg0 & arg1);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `N2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) ops(true, true));
        java.lang.System.out.println((java.lang.String) ops(true, false));
        return;
    }
}
