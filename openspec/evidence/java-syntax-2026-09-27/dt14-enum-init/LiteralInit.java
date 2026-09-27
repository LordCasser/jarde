package dt14;

public enum LiteralInit {
    FIRST(1),
    SECOND(20);

    private final int code;

    LiteralInit(int code) {
        this.code = code;
    }

    int code() {
        return code;
    }
}
