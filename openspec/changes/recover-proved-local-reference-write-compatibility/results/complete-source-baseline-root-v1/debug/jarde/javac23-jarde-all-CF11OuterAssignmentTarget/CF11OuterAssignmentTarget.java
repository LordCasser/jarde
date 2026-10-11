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

    public static java.lang.String positive(java.util.List values) {
        // @method positive(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `cf11.CF11OuterAssignmentTarget`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String target;
        target = "seed";
        for (java.lang.Object iteratorElement20 : values) {
            target = (java.lang.String) iteratorElement20;
            cf11.Runner.assigned(target);
        }
        return target;
    }

    public static java.lang.Object noCastAssign(java.util.List values) {
        // @method noCastAssign(Ljava/util/List;)Ljava/lang/Object;
        // @declaration a static method of `cf11.CF11OuterAssignmentTarget`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String target;
        java.util.Iterator iterator;
        target = "seed";
        iterator = values.iterator();
        while (iterator.hasNext()) {
            target = iterator.next();
        }
        return target;
    }

    public static java.lang.String bodyEffectBeforeAssign(java.util.List values) {
        // @method bodyEffectBeforeAssign(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `cf11.CF11OuterAssignmentTarget`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String target;
        java.util.Iterator iterator;
        target = "seed";
        iterator = values.iterator();
        while (iterator.hasNext()) {
            cf11.Runner.beforeAssignment();
            target = (java.lang.String) iterator.next();
            cf11.Runner.assigned(target);
        }
        return target;
    }

    public static java.lang.String extraNextConsumption(java.util.List values) {
        // @method extraNextConsumption(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `cf11.CF11OuterAssignmentTarget`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String target;
        java.util.Iterator iterator;
        target = "seed";
        iterator = values.iterator();
        while (iterator.hasNext()) {
            iterator.next();
            target = (java.lang.String) iterator.next();
            cf11.Runner.assigned(target);
        }
        return target;
    }
}
