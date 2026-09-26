public class AssertHandlerBoundary {
    static final boolean $assertionsDisabled = disabled();
    static int caught;

    private static boolean disabled() {
        return !AssertHandlerBoundary.class.desiredAssertionStatus();
    }

    private static boolean condition(boolean throwAssertion) {
        if (throwAssertion) {
            throw new AssertionError("condition");
        }
        return false;
    }

    static void check(boolean throwAssertion) {
        if (!$assertionsDisabled && !condition(throwAssertion)) {
            try {
                throw new AssertionError("generated");
            } catch (AssertionError error) {
                caught++;
            }
        }
    }
}
