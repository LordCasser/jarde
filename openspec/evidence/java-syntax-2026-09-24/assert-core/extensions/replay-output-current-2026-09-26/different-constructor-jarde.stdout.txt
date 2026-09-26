// jarde: presentation of `AssertDifferentConstructor` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AssertDifferentConstructor extends java.lang.Object {
    static final boolean $assertionsDisabled;

    public AssertDifferentConstructor() {
        // @method <init>()V
        // @declaration a constructor of `AssertDifferentConstructor`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean disabled() {
        // @method disabled()Z
        // @declaration a static method of `AssertDifferentConstructor`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return !AssertDifferentConstructor.class.desiredAssertionStatus();
    }

    static void check(boolean condition) {
        // @method check(Z)V
        // @declaration a static method of `AssertDifferentConstructor`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!AssertDifferentConstructor.$assertionsDisabled) {
            if (!condition) {
                throw new java.lang.AssertionError(42);
            }
        }
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `AssertDifferentConstructor`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $assertionsDisabled = disabled();
    }
}
