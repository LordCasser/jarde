// jarde: presentation of `em11simple/NullArrayCalls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em11simple;

public class NullArrayCalls extends java.lang.Object {
    public NullArrayCalls() {
        // @method <init>()V
        // @declaration a constructor of `em11simple.NullArrayCalls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String call(java.lang.String arg0) {
        // @method call(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `em11simple.NullArrayCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "String";
    }

    // jarde: generic Signature projection refused for `call(Ljava/util/List;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public static java.lang.String call(java.util.List arg0) {
        // @method call(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `em11simple.NullArrayCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "List";
    }

    // jarde: generic Signature projection refused for `call(Ljava/util/ArrayList;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public static java.lang.String call(java.util.ArrayList arg0) {
        // @method call(Ljava/util/ArrayList;)Ljava/lang/String;
        // @declaration a static method of `em11simple.NullArrayCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "ArrayList";
    }

    public static java.lang.String choose(java.lang.Object[][] arg0, java.lang.Object arg1) {
        // @method choose([[Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `em11simple.NullArrayCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "Object[][]";
    }

    public static java.lang.String choose(int[][] arg0, int arg1) {
        // @method choose([[II)Ljava/lang/String;
        // @declaration a static method of `em11simple.NullArrayCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "int[][]";
    }

    public static java.lang.String run() {
        // @method run()Ljava/lang/String;
        // @declaration a static method of `em11simple.NullArrayCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[][][] local0 = new int[1][1][1];
        return new java.lang.StringBuilder().append((java.lang.String) call((java.lang.String) null)).append("/").append((java.lang.String) call((java.util.List) null)).append("/").append((java.lang.String) call((java.util.ArrayList) null)).append("/").append((java.lang.String) choose((java.lang.Object[][]) local0, (java.lang.Object) java.lang.Integer.valueOf(-1))).append("/").append((java.lang.String) choose(local0[0], -2)).toString();
    }
}
