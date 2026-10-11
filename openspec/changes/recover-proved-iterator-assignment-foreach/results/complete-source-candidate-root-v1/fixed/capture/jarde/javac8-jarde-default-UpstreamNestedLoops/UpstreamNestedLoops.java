// jarde: presentation of `cf11/UpstreamNestedLoops` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class UpstreamNestedLoops extends java.lang.Object {
    public UpstreamNestedLoops() {
        // @method <init>()V
        // @declaration a constructor of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `test(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void test(java.util.List l1, java.util.List l2) {
        // @method test(Ljava/util/List;Ljava/util/List;)V
        // @declaration an instance method of `cf11.UpstreamNestedLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String s1;
        java.util.Iterator local5;
        for (java.lang.Object iteratorElement17 : l1) {
            s1 = (java.lang.String) iteratorElement17;
            for (java.lang.Object iteratorElement47 : l2) {
                java.lang.String s2 = (java.lang.String) iteratorElement47;
                if (s1.equals((java.lang.Object) s2)) {
                    if (s1.length() == 5) {
                        l2.add((java.lang.Object) s1);
                    } else {
                        l1.remove((java.lang.Object) s2);
                    }
                }
            }
        }
        if (l2.size() > 0) {
            l1.clear();
        }
        return;
    }
}
