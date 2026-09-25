public final class ConstructorPairProbe {
    private final String first;
    private final String second;

    public ConstructorPairProbe(String text, int mode) {
        this(mode == 1 ? text : "", mode == 0 ? "" : text);
    }

    public ConstructorPairProbe(String first, String second) {
        this.first = first;
        this.second = second;
    }

    public String first() {
        return first;
    }

    public String second() {
        return second;
    }
}
