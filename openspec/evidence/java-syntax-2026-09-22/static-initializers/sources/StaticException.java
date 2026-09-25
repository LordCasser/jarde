public class StaticException {
    static int value = fail();

    static int fail() {
        throw new IllegalStateException("init");
    }

    public static int get() {
        return value;
    }
}
