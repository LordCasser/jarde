public class ThrowingArgument extends IllegalArgumentException {
    public static final IllegalStateException ORIGINAL = new IllegalStateException("message");

    public ThrowingArgument(String message) { super(message); }

    @Override public String getMessage() { throw ORIGINAL; }
}
