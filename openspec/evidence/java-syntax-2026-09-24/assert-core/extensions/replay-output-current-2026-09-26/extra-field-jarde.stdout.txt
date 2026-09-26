// jarde: presentation of `AssertExtraFieldAccess` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AssertExtraFieldAccess extends java.lang.Object {
    static final boolean $assertionsDisabled;

    static int extraWrites;

    public AssertExtraFieldAccess() {
        // @method <init>()V
        // @declaration a constructor of `AssertExtraFieldAccess`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean disabled() {
        // @method disabled()Z
        // @declaration a static method of `AssertExtraFieldAccess`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return !AssertExtraFieldAccess.class.desiredAssertionStatus();
    }

    static boolean statusProbe() {
        // @method statusProbe()Z
        // @declaration a static method of `AssertExtraFieldAccess`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return AssertExtraFieldAccess.$assertionsDisabled;
    }

    static void check(boolean condition) {
        // @method check(Z)V
        // @declaration a static method of `AssertExtraFieldAccess`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (AssertExtraFieldAccess.$assertionsDisabled) {
            return;
        } else {
            AssertExtraFieldAccess.extraWrites = AssertExtraFieldAccess.extraWrites + 1;
            if (!condition) {
                throw new java.lang.AssertionError();
            } else {
                return;
            }
        }
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `AssertExtraFieldAccess`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $assertionsDisabled = disabled();
    }
}
