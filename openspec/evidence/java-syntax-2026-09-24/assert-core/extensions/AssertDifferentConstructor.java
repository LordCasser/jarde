public class AssertDifferentConstructor {
    static final boolean $assertionsDisabled = disabled();

    private static boolean disabled() {
        return !AssertDifferentConstructor.class.desiredAssertionStatus();
    }

    static void check(boolean condition) {
        if (!$assertionsDisabled && !condition) {
            throw new AssertionError(42);
        }
    }
}
