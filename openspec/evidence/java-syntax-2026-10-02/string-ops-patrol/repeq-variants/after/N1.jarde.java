// jarde: presentation of `N1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class N1 extends java.lang.Object {
    public N1() {
        // @method <init>()V
        // @declaration a constructor of `N1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String take(boolean arg0) {
        // @method take(Z)Ljava/lang/String;
        // @declaration a static method of `N1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? "1" : "0";
    }

    public static java.lang.String ops(java.lang.String arg0) {
        // jarde: not recovered: the recovery run for `ops(Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method ops(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `N1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 16 13
        // the parameter 0 of the invocation at BCI 13 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `N1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) ops("abcd"));
        java.lang.System.out.println((java.lang.String) ops(""));
        return;
    }
}
