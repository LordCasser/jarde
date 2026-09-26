// jarde: presentation of `AssertCore` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AssertCore extends java.lang.Object {
    private static int guardCalls;

    private static int detailCalls;

    static final boolean $assertionsDisabled;

    public AssertCore() {
        // @method <init>()V
        // @declaration a constructor of `AssertCore`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean guard(boolean ok) {
        // @method guard(Z)Z
        // @declaration a static method of `AssertCore`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AssertCore.guardCalls = AssertCore.guardCalls + 1;
        return ok;
    }

    private static java.lang.String detail() {
        // @method detail()Ljava/lang/String;
        // @declaration a static method of `AssertCore`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AssertCore.detailCalls = AssertCore.detailCalls + 1;
        return "bad";
    }

    public static java.lang.String check(boolean ok) {
        // @method check(Z)Ljava/lang/String;
        // @declaration a static method of `AssertCore`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (!AssertCore.$assertionsDisabled) {
            if (!guard(ok)) {
                throw new java.lang.AssertionError((java.lang.Object) detail());
            }
        }
        return new java.lang.StringBuilder().append(AssertCore.guardCalls).append("|").append(AssertCore.detailCalls).toString();
    }

    public static java.lang.String counts() {
        // @method counts()Ljava/lang/String;
        // @declaration a static method of `AssertCore`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(AssertCore.guardCalls).append("|").append(AssertCore.detailCalls).toString();
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `AssertCore`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $assertionsDisabled = (!AssertCore.class.desiredAssertionStatus() ? 2 : 3) % 2 != 0;
    }
}
