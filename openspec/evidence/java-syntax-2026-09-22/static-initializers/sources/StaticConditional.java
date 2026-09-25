public class StaticConditional {
    static int value;

    static {
        if (System.getProperty("jarde.static.missing") == null) {
            value = 1;
        } else {
            value = 2;
        }
    }

    public static int get() {
        return value;
    }
}
