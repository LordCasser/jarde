public class EnumSwitchSubject {
    private static int trace;

    private static int mark(int digit) {
        trace = trace * 10 + digit;
        return digit;
    }

    public static int trace() {
        return trace;
    }

    public static void reset() {
        trace = 0;
    }

    public static int choose(Hue hue) {
        switch (hue) {
            case RED:
                return mark(1);
            case BLUE:
                return mark(2);
            default:
                return mark(3);
        }
    }
}
