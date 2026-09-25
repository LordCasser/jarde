
/* JADX INFO: loaded from: ShiftEffectOrder.class */
public class ShiftEffectOrder {
    private static int events;

    private static int step(int i, int i2) {
        events = (events * 10) + i;
        return i2;
    }

    private static int fail(int i) {
        events = (events * 10) + i;
        throw new IllegalArgumentException("shift operand");
    }

    public static int inline(int i) {
        events = 0;
        return (events * 1000) + (step(1, i) << step(2, 2));
    }

    public static int separated(int i) {
        events = 0;
        int iStep = step(1, i);
        step(3, 0);
        return (events * 1000) + (iStep << step(2, 2));
    }

    public static int throwLeft(int i) {
        events = 0;
        return fail(1) << step(2, i);
    }

    public static int throwRight(int i) {
        events = 0;
        return step(1, i) << fail(2);
    }

    public static int events() {
        return events;
    }
}
