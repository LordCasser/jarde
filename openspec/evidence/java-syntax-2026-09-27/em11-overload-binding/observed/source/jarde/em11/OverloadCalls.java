// jarde: presentation of `em11/OverloadCalls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em11;

public class OverloadCalls extends java.lang.Object {
    public OverloadCalls() {
        // @method <init>()V
        // @declaration a constructor of `em11.OverloadCalls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String call(java.lang.String arg0) {
        // @method call(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `em11.OverloadCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "String";
    }

    // jarde: generic Signature projection refused for `call(Ljava/util/List;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public static java.lang.String call(java.util.List arg0) {
        // @method call(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `em11.OverloadCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "List";
    }

    // jarde: generic Signature projection refused for `call(Ljava/util/ArrayList;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public static java.lang.String call(java.util.ArrayList arg0) {
        // @method call(Ljava/util/ArrayList;)Ljava/lang/String;
        // @declaration a static method of `em11.OverloadCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "ArrayList";
    }

    public static java.lang.String choose(java.lang.Object[][] arg0, java.lang.Object arg1) {
        // @method choose([[Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `em11.OverloadCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "Object[][]";
    }

    public static java.lang.String choose(int[][] arg0, int arg1) {
        // @method choose([[II)Ljava/lang/String;
        // @declaration a static method of `em11.OverloadCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "int[][]";
    }

    public static java.lang.String run(java.lang.Object arg0) {
        // @method run(Ljava/lang/Object;)Ljava/lang/String;
        // @declaration a static method of `em11.OverloadCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1;
        java.lang.String local2;
        java.lang.String local3;
        java.lang.String local4;
        java.lang.String local5;
        local1 = call(new java.util.ArrayList());
        // @bytecode 21 18
        // the parameter 0 of the invocation at BCI 18 is declared `java.util.List` presents `java.util.ArrayList` but the invocation requires `java.util.List` and this layer has no safe reference conversion evidence
        local3 = call((java.lang.String) null);
        local4 = call((java.util.List) null);
        local5 = call((java.util.ArrayList) null);
        Object local6 = arg0 instanceof java.lang.String ? call((java.lang.String) arg0) : "none";
        int[][][] local7 = new int[1][1][1];
        java.lang.String local8 = choose((java.lang.Object[][]) local7, (java.lang.Object) java.lang.Integer.valueOf(-1));
        java.lang.String local9 = choose(local7[0], -2);
        return local1 + "/" + local2 + "/" + local3 + "/" + local4 + "/" + local5 + "/" + local6 + "/" + local8 + "/" + local9;
    }
}
