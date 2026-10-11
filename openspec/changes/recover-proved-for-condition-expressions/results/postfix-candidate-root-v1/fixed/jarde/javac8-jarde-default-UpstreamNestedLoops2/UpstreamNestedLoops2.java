// jarde: presentation of `cf11/UpstreamNestedLoops2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public class UpstreamNestedLoops2 extends java.lang.Object {
    public UpstreamNestedLoops2() {
        // @method <init>()V
        // @declaration a constructor of `cf11.UpstreamNestedLoops2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `test(Ljava/util/List;)Z`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public boolean test(java.util.List list) {
        // @method test(Ljava/util/List;)Z
        // @declaration an instance method of `cf11.UpstreamNestedLoops2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int j;
        int i;
        j = 0;
        java.lang.String s;
        for (i = 0; i < list.size(); i = i + 1) {
            s = (java.lang.String) list.get(i);
            while (j < s.length()) {
                j = j + 1;
            }
        }
        return j > 10;
    }
}
