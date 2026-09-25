public class CatchEscape {
    static Object read(Runnable action) {
        Object x = action;
        try {
            action.run();
        } catch (IllegalArgumentException e) {
            x = e;
        }
        return x;
    }
}
