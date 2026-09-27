// jarde: presentation of `cf06/InnerAssignCases` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf06;

public class InnerAssignCases extends java.lang.Object {
    private java.lang.String field;

    private java.lang.String swapField;

    public InnerAssignCases() {
        // @method <init>()V
        // @declaration a constructor of `cf06.InnerAssignCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int lengthBranch(java.lang.String arg0) {
        // @method lengthBranch(Ljava/lang/String;)I
        // @declaration a static method of `cf06.InnerAssignCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (!arg0.isEmpty()) {
            arg0.length();
            // @bytecode 11
            // the instruction at BCI 11 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
            // @bytecode 12
            // the value at BCI 12 comes from an Duplicate at BCI 11, which produces no expression this subset writes
            // @bytecode 7 19 14
            // the value at BCI 14 comes from an Duplicate at BCI 11, which produces no expression this subset writes
        }
        return -1;
    }

    public boolean assignedAndChecked(java.lang.String arg1) {
        // jarde: not recovered: the recovery run for `assignedAndChecked(Ljava/lang/String;)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method assignedAndChecked(Ljava/lang/String;)Z
        // @declaration an instance method of `cf06.InnerAssignCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 5 8 9 12 13 14 17 18 21 24 25 28 29
        // the short-circuit chain from BCI 5 through 21 reaches a shared value consumer at BCI 29, but this slice has no SSA proof for that value; the complete region is quoted
    }

    private boolean call(java.lang.String arg1) {
        // @method call(Ljava/lang/String;)Z
        // @declaration an instance method of `cf06.InnerAssignCases`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.field = this.swapField;
        return arg1.isEmpty();
    }

    public boolean run(java.lang.String arg1, java.lang.String arg2) {
        // @method run(Ljava/lang/String;Ljava/lang/String;)Z
        // @declaration an instance method of `cf06.InnerAssignCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.field = null;
        this.swapField = arg2;
        return this.assignedAndChecked(arg1);
    }
}
