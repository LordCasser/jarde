public enum DelegatingEnum {
    ZERO,
    ONE(1);

    private final int value;

    DelegatingEnum() {
        this(0);
    }

    DelegatingEnum(int value) {
        ConstructorEffects.record(value);
        this.value = value;
    }

    public int value() {
        return value;
    }
}
