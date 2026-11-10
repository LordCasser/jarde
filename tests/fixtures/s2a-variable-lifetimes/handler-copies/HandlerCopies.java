public final class HandlerCopies {
    public static int copy(boolean fail, int input) {
        int saved = input;
        try {
            if (fail) throw new IllegalArgumentException();
            return saved;
        } catch (IllegalArgumentException caught) {
            return saved + input;
        }
    }

    public static int initializedByCall(boolean fail, int input) {
        int saved = Integer.parseInt("6");
        try {
            if (fail) throw new IllegalArgumentException();
            return saved;
        } catch (IllegalArgumentException caught) {
            return saved + input;
        }
    }
}
