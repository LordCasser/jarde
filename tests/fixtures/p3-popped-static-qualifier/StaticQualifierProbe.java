public final class StaticQualifierProbe {
    public static int selects;
    public static int rightCalls;
    public static int value;
    public static boolean failSelect;

    public static void reset() {
        selects = 0;
        rightCalls = 0;
        value = 5;
        failSelect = false;
    }

    public static StaticQualifierProbe receiver() {
        selects++;
        if (failSelect) {
            throw new IllegalStateException("select");
        }
        return null;
    }

    public static int rhs() {
        rightCalls++;
        return 7;
    }

    public static int read() {
        return receiver().value;
    }

    public static int write() {
        receiver().value = rhs();
        return value;
    }

    public static int writeSum() {
        receiver().value += rhs();
        return value;
    }
}
