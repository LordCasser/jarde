public class KnownBoundPositive {
    public String value() { return "bound"; }
    public java.util.function.Supplier<String> entryThis() { return this::value; }
    public static java.util.function.IntSupplier constantString() { return "value"::length; }
}
