// jarde: presentation of `AssertDuplicateStatusWrite` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AssertDuplicateStatusWrite extends java.lang.Object {
    static final boolean $assertionsDisabled;

    public AssertDuplicateStatusWrite() {
        // @method <init>()V
        // @declaration a constructor of `AssertDuplicateStatusWrite`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean disabled() {
        // @method disabled()Z
        // @declaration a static method of `AssertDuplicateStatusWrite`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return !AssertDuplicateStatusWrite.class.desiredAssertionStatus();
    }

    static void check(boolean condition) {
        // @method check(Z)V
        // @declaration a static method of `AssertDuplicateStatusWrite`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!AssertDuplicateStatusWrite.$assertionsDisabled) {
            if (!condition) {
                throw new java.lang.AssertionError();
            }
        }
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `AssertDuplicateStatusWrite`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $assertionsDisabled = disabled();
        $assertionsDisabled = false;
    }
}
