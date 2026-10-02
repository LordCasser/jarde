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
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 35 32 29 5 4 8 9 10 11 14 15 16 17 18 21 22 23 24 25 28
        // the copy at BCI 3 has no proved local assignment
        java.util.ArrayList local1 = new java.util.ArrayList();
        addSuper((java.util.List) local1, (java.lang.Integer) java.lang.Integer.valueOf(7));
        // @bytecode 53 56 57 60 61 64 67 69 72 73 74 79 82 84 87 89 92 95 98
        // the statement at BCI 98 reads `local0`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `W1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) use());
        return;
    }
}
