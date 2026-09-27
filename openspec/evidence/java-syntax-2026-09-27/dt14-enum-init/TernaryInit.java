package dt14;

public enum TernaryInit {
    FIRST(useNumber() ? 1 : 10),
    SECOND(useNumber() ? 2 : 20),
    ANY(useNumber() ? 1 : 2);

    static int calls;
    private final int code;

    TernaryInit(int code) {
        this.code = code;
    }

    int code() {
        return code;
    }

    public static boolean useNumber() {
        calls++;
        return calls % 2 == 1;
    }
}
