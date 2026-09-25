public final class CatchAfterFieldAssignment {
    private static int field;

    private static void fail() {
        throw new IllegalArgumentException("probe");
    }

    private static void maybeFail(boolean shouldThrow) {
        if (shouldThrow) {
            fail();
        }
    }

    public static int fieldAssignment(boolean shouldThrow) {
        field = 7;
        try {
            maybeFail(shouldThrow);
        } catch (IllegalArgumentException ex) {
            field = 18;
        }
        return field;
    }

    public static int localAssignment(boolean shouldThrow) {
        int local = 7;
        try {
            maybeFail(shouldThrow);
        } catch (IllegalArgumentException ex) {
            local = 18;
        }
        return local;
    }
}
