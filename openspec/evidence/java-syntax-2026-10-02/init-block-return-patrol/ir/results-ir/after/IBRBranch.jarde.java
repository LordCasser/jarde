// jarde: presentation of `IBRBranch` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class IBRBranch extends java.lang.Object {
    static int low;

    static int high;

    public IBRBranch() {
        // @method <init>()V
        // @declaration a constructor of `IBRBranch`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IBRBranch`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(IBRBranch.low).append(":").append(IBRBranch.high).toString());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `IBRBranch`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (java.lang.Boolean.getBoolean("ibr-branch-boom")) {
            throw new java.lang.IllegalStateException("branch-fail");
        } else {
            IBRBranch.low = 7;
            if (java.lang.Boolean.getBoolean("ibr-branch-guard")) {
                throw new java.lang.IllegalStateException("branch-guard");
            } else {
                IBRBranch.high = IBRBranch.low + 2;
            }
        }
    }
}
