public class ShiftEffectOrder {
    private static int events;

    private static int step(int tag, int value) {
        events = events * 10 + tag;
        return value;
    }

    private static int fail(int tag) {
        events = events * 10 + tag;
        throw new IllegalArgumentException("shift operand");
    }

    public static int inline(int value) {
        events = 0;
        int result = step(1, value) << step(2, 2);
        return events * 1000 + result;
    }

    public static int separated(int value) {
        events = 0;
        int left = step(1, value);
        step(3, 0);
        int result = left << step(2, 2);
        return events * 1000 + result;
    }

    public static int throwLeft(int value) {
        events = 0;
        return fail(1) << step(2, value);
    }

    public static int throwRight(int value) {
        events = 0;
        return step(1, value) << fail(2);
    }

    public static int events() {
        return events;
    }
}
