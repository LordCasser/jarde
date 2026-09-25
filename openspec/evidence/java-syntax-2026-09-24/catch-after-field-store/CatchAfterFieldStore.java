public final class CatchAfterFieldStore {
    private static int calls;

    private static void fail() {
        throw new IllegalArgumentException("probe");
    }

    private static void maybeFail(boolean shouldThrow) {
        if (shouldThrow) {
            fail();
        }
    }

    public static int fieldStore(boolean shouldThrow) {
        calls = 7;
        try {
            maybeFail(shouldThrow);
            return calls;
        } catch (IllegalArgumentException ex) {
            return calls + 11;
        }
    }

    public static int localStore(boolean shouldThrow) {
        int value = 7;
        try {
            maybeFail(shouldThrow);
            return value;
        } catch (IllegalArgumentException ex) {
            return value + 11;
        }
    }
}
