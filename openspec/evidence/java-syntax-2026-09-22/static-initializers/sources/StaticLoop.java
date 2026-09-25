public class StaticLoop {
    static int value;

    static {
        for (int i = 0; i < 3; i++) {
            value += i;
        }
    }

    public static int get() {
        return value;
    }
}
