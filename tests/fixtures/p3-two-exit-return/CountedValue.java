public final class CountedValue {
    public static int calls;
    private final String value;

    public CountedValue(String value) { this.value = value; }

    @Override public boolean equals(Object other) {
        calls++;
        return other instanceof CountedValue && value.equals(((CountedValue) other).value);
    }
}
