// jarde: presentation of `AssertHandlerBoundary` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AssertHandlerBoundary extends java.lang.Object {
    static final boolean $assertionsDisabled;

    static int caught;

    public AssertHandlerBoundary() {
        // @method <init>()V
        // @declaration a constructor of `AssertHandlerBoundary`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean disabled() {
        // @method disabled()Z
        // @declaration a static method of `AssertHandlerBoundary`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return !AssertHandlerBoundary.class.desiredAssertionStatus();
    }

    private static boolean condition(boolean throwAssertion) {
        // @method condition(Z)Z
        // @declaration a static method of `AssertHandlerBoundary`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        if (throwAssertion) {
            throw new java.lang.AssertionError((java.lang.Object) "condition");
        } else {
            return false;
        }
    }

    static void check(boolean throwAssertion) {
        // jarde: not recovered: the recovery run for `check(Z)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method check(Z)V
        // @declaration a static method of `AssertHandlerBoundary`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 6 7 10 13 16 17 19 22 23 24 27 28 29 32
        // canonical block at BCI 32 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `AssertHandlerBoundary`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        $assertionsDisabled = disabled();
    }
}
