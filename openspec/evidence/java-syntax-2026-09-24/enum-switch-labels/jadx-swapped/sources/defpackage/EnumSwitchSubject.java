package defpackage;

/* JADX INFO: loaded from: enum-switch-swapped.jar:EnumSwitchSubject.class */
public class EnumSwitchSubject {
    private static int trace;

    private static int mark(int i) {
        trace = (trace * 10) + i;
        return i;
    }

    public static int trace() {
        return trace;
    }

    public static void reset() {
        trace = 0;
    }

    public static int choose(Hue hue) {
        switch (hue) {
            case BLUE:
                return mark(1);
            case RED:
                return mark(2);
            default:
                return mark(3);
        }
    }
}
