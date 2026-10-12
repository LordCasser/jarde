// jarde: presentation of `cf11/DistinctStringFixture` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class DistinctStringFixture extends java.lang.Object {
    public static int effects;

    public DistinctStringFixture() {
        // @method <init>()V
        // @declaration a constructor of `cf11.DistinctStringFixture`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `renamed(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void renamed(java.util.List arg1, java.util.List arg2) {
        // @method renamed(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.DistinctStringFixture`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local4;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : arg1) {
            local4 = (java.lang.String) iteratorElement17;
            for (java.lang.Object iteratorElement47 : arg2) {
                java.lang.String local6 = (java.lang.String) iteratorElement47;
                if (local4.equals((java.lang.Object) local6)) {
                    if (local4.length() == 5) {
                        arg2.add((java.lang.Object) local4);
                        cf11.DistinctStringFixture.effects = cf11.DistinctStringFixture.effects + 1;
                    } else {
                        arg1.remove((java.lang.Object) local6);
                        cf11.DistinctStringFixture.effects = cf11.DistinctStringFixture.effects + 1;
                    }
                }
            }
        }
        if (arg2.size() > 0) {
            arg1.clear();
            cf11.DistinctStringFixture.effects = cf11.DistinctStringFixture.effects + 1;
        }
        return;
    }
}
