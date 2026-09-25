
/* JADX INFO: loaded from: StaticEarlyReturn.class */
public class StaticEarlyReturn {
    static int value;

    static boolean early() {
        return Boolean.getBoolean("jarde.static.early");
    }

    public static int get() {
        return value;
    }

    static {
        if (early()) {
            value = 1;
        } else {
            value = 7;
        }
    }
}
