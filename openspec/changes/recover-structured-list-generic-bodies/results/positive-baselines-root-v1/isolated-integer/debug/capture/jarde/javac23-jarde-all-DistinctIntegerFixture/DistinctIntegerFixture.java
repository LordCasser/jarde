// jarde: presentation of `cf11/DistinctIntegerFixture` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class DistinctIntegerFixture extends java.lang.Object {
    public static int effects;

    public DistinctIntegerFixture() {
        // @method <init>()V
        // @declaration a constructor of `cf11.DistinctIntegerFixture`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `numbers(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void numbers(java.util.List selected, java.util.List available) {
        // @method numbers(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.DistinctIntegerFixture`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Integer value;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : selected) {
            value = (java.lang.Integer) iteratorElement17;
            for (java.lang.Object iteratorElement47 : available) {
                java.lang.Integer option = (java.lang.Integer) iteratorElement47;
                if (value.equals((java.lang.Object) option)) {
                    if (value.intValue() == 5) {
                        available.add((java.lang.Object) value);
                        cf11.DistinctIntegerFixture.effects = cf11.DistinctIntegerFixture.effects + 1;
                    } else {
                        selected.remove((java.lang.Object) option);
                        cf11.DistinctIntegerFixture.effects = cf11.DistinctIntegerFixture.effects + 1;
                    }
                }
            }
        }
        if (available.size() > 0) {
            selected.clear();
            cf11.DistinctIntegerFixture.effects = cf11.DistinctIntegerFixture.effects + 1;
        }
        return;
    }
}
