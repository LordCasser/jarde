public class StaticExternalException {
    static int value = ThrowingHelper.fail();

    public static int get() {
        return value;
    }
}
