public class StaticEarlyReturn {
    static int value;

    static boolean early() {
        return Boolean.getBoolean("jarde.static.early");
    }

    static {
        if (early()) {
            value = 1;
        }
        value = 7;
    }

    public static int get() {
        return value;
    }
}
