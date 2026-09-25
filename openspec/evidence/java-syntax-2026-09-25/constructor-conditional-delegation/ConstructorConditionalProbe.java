public final class ConstructorConditionalProbe {
    private final int value;

    public ConstructorConditionalProbe(String text, int input) {
        this(text == null ? 0 : input);
    }

    public ConstructorConditionalProbe(int value) {
        this.value = value;
    }

    public int value() {
        return value;
    }
}
