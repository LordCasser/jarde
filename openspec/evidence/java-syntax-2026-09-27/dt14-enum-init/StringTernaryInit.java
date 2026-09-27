package dt14;

public enum StringTernaryInit {
    FIRST(useNumber() ? "1" : "A"),
    SECOND(useNumber() ? "2" : "B"),
    ANY(useNumber() ? "1" : "2");

    private final String value;

    StringTernaryInit(String value) {
        this.value = value;
    }

    String value() {
        return value;
    }

    static boolean useNumber() {
        return false;
    }
}
