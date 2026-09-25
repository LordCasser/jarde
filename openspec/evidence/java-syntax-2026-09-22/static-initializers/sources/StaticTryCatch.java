public class StaticTryCatch {
    static int value;

    static {
        try {
            value = Integer.parseInt("7");
        } catch (NumberFormatException error) {
            value = 9;
        }
    }

    public static int get() {
        return value;
    }
}
