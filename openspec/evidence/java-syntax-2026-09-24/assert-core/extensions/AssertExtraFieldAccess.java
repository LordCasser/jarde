public class AssertExtraFieldAccess {
    static final boolean $assertionsDisabled = disabled();
    static int extraWrites;

    private static boolean disabled() {
        return !AssertExtraFieldAccess.class.desiredAssertionStatus();
    }

    static boolean statusProbe() {
        return $assertionsDisabled;
    }

    static void check(boolean condition) {
        if ($assertionsDisabled) {
            return;
        }
        extraWrites++;
        if (!condition) {
            throw new AssertionError();
        }
    }
}
