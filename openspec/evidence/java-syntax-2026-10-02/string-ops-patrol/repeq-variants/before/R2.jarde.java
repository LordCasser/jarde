// jarde: presentation of `R2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class R2 extends java.lang.Object {
    public R2() {
        // @method <init>()V
        // @declaration a constructor of `R2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String mark(boolean arg0, boolean arg1) {
        // @method mark(ZZ)Ljava/lang/String;
        // @declaration a static method of `R2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.valueOf(arg0) + arg1;
    }

    public static java.lang.String ops(java.lang.String arg0) {
        // @method ops(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `R2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1 = arg0 + "x";
        // @bytecode 43 40 37 36
        // the parameter 0 of the invocation at BCI 40 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `R2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) ops("abcd"));
        java.lang.System.out.println((java.lang.String) ops(""));
        return;
    }
}
