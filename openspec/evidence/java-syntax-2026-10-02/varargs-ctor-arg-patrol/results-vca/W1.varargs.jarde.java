// jarde: presentation of `W1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class W1 extends java.lang.Object {
    public W1() {
        // @method <init>()V
        // @declaration a constructor of `W1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `sumExt(Ljava/util/List;)D`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static double sumExt(java.util.List arg0) {
        // @method sumExt(Ljava/util/List;)D
        // @declaration a static method of `W1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        double local1;
        local1 = 0x0.0000000000000p-1022d;
        for (java.lang.Object iteratorElement19 : arg0) {
            java.lang.Number local4 = (java.lang.Number) iteratorElement19;
            local1 = local1 + local4.doubleValue();
        }
        return local1;
    }

    // jarde: generic Signature projection refused for `addSuper(Ljava/util/List;Ljava/lang/Integer;)V`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static void addSuper(java.util.List arg0, java.lang.Integer arg1) {
        // @method addSuper(Ljava/util/List;Ljava/lang/Integer;)V
        // @declaration a static method of `W1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.add((java.lang.Object) arg1);
        return;
    }

    // jarde: generic Signature projection refused for `nameOf(Ljava/lang/Class;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String nameOf(java.lang.Class arg0) {
        // @method nameOf(Ljava/lang/Class;)Ljava/lang/String;
        // @declaration a static method of `W1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.getSimpleName();
    }

    public static java.lang.String use() {
        // @method use()Ljava/lang/String;
        // @declaration a static method of `W1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayList local0 = new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2), java.lang.Integer.valueOf(3)}));
        java.util.ArrayList local1 = new java.util.ArrayList();
        addSuper((java.util.List) local1, (java.lang.Integer) java.lang.Integer.valueOf(7));
        return "" + sumExt((java.util.List) local0) + ":" + local1.get(0) + ":" + nameOf(W1.class);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `W1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) use());
        return;
    }
}
