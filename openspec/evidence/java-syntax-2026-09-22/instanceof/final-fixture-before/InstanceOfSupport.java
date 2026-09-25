/** Source-only producer for the call and producer-failure instanceof cases. */
public final class InstanceOfSupport {
    public static int calls;
    public static boolean fail;

    private InstanceOfSupport() {}

    public static String value() {
        calls++;
        if (fail) {
            throw new IllegalStateException("producer");
        }
        return "value";
    }
}
