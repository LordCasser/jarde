public class StaticHelper {
    static int value = helper();

    static int helper() {
        return 11;
    }

    public static int get() {
        return value;
    }
}
