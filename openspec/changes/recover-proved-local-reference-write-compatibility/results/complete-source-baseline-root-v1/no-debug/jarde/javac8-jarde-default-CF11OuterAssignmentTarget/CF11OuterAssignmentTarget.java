// jarde: presentation of `cf11/CF11OuterAssignmentTarget` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class CF11OuterAssignmentTarget extends java.lang.Object {
    public CF11OuterAssignmentTarget() {
        // @method <init>()V
        // @declaration a constructor of `cf11.CF11OuterAssignmentTarget`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String positive(java.util.List arg0) {
        // @method positive(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `cf11.CF11OuterAssignmentTarget`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1;
        local1 = "seed";
        for (java.lang.Object iteratorElement20 : arg0) {
            local1 = (java.lang.String) iteratorElement20;
            cf11.Runner.assigned(local1);
        }
        return local1;
    }

    public static java.lang.Object noCastAssign(java.util.List arg0) {
        // @method noCastAssign(Ljava/util/List;)Ljava/lang/Object;
        // @declaration a static method of `cf11.CF11OuterAssignmentTarget`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1;
        java.util.Iterator local2;
        local1 = "seed";
        local2 = arg0.iterator();
        while (local2.hasNext()) {
            local1 = local2.next();
        }
        return local1;
    }

    public static java.lang.String bodyEffectBeforeAssign(java.util.List arg0) {
        // @method bodyEffectBeforeAssign(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `cf11.CF11OuterAssignmentTarget`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1;
        java.util.Iterator local2;
        local1 = "seed";
        local2 = arg0.iterator();
        while (local2.hasNext()) {
            cf11.Runner.beforeAssignment();
            local1 = (java.lang.String) local2.next();
            cf11.Runner.assigned(local1);
        }
        return local1;
    }

    public static java.lang.String extraNextConsumption(java.util.List arg0) {
        // @method extraNextConsumption(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `cf11.CF11OuterAssignmentTarget`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1;
        java.util.Iterator local2;
        local1 = "seed";
        local2 = arg0.iterator();
        while (local2.hasNext()) {
            local2.next();
            local1 = (java.lang.String) local2.next();
            cf11.Runner.assigned(local1);
        }
        return local1;
    }
}
