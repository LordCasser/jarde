public class VoidControl {
    static int calls;

    public static void init() {
        calls = 1;
        return;
    }

    public static int get() {
        return calls;
    }
}
