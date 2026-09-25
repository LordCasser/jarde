/** Source-only helper for FinalStaticProbe's observable <clinit> effects. */
public final class FinalStaticSupport {
    private static int count;
    private static final StringBuilder order = new StringBuilder();

    private FinalStaticSupport() {}

    public static int next(String name) {
        if (order.length() != 0) {
            order.append(',');
        }
        order.append(name);
        return ++count;
    }

    public static boolean branch() {
        return Boolean.parseBoolean(System.getProperty("jarde.final-static.branch"));
    }

    public static String order() {
        return order.toString();
    }
}
