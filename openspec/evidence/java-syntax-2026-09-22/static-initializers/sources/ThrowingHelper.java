public class ThrowingHelper {
    public static int fail() {
        throw new IllegalStateException("external init");
    }
}
