public class KnownBound {
    public String value() { return "bound"; }
    public java.util.function.Supplier<String> entryThis() { return this::value; }
    public static java.util.function.IntSupplier constantString() { return "value"::length; }
    public static java.util.function.Supplier<String> constantClass() { return KnownBound.class::getName; }
}
