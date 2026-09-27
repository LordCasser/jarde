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
            int local1;
            if ((local1 = arg0.length()) > 5) {
            } else {
                return local1;
            }
        }
        return -1;
    }

    public boolean assignedAndChecked(java.lang.String arg1) {
        // @method assignedAndChecked(Ljava/lang/String;)Z
        // @declaration an instance method of `cf06.InnerAssignCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local2;
        return this.call(arg1) || (local2 = this.field) != null && local2.isEmpty();
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
