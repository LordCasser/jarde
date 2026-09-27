package dt14;

public enum AlternatingStringInit {
    FIRST(choose() ? "1" : "A"),
    SECOND(choose() ? "2" : "B"),
    THIRD(choose() ? "3" : "C");

    static int calls;
    private final String value;

    AlternatingStringInit(String value) {
        this.value = value;
    }

    String value() {
        return value;
    }

    static boolean choose() {
        calls++;
        return calls % 2 == 1;
    }
}
