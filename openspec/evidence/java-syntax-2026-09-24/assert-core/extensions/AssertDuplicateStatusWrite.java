public class AssertDuplicateStatusWrite {
    static boolean $assertionsDisabled;

    private static boolean disabled() {
        return !AssertDuplicateStatusWrite.class.desiredAssertionStatus();
    }

    static {
        $assertionsDisabled = disabled();
        $assertionsDisabled = false;
    }

    static void check(boolean condition) {
        if (!$assertionsDisabled && !condition) {
            throw new AssertionError();
        }
    }
}
