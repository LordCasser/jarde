package defpackage;

/* JADX INFO: loaded from: ConditionalBoundarySwitch.class */
public class ConditionalBoundarySwitch {
    private static int trace;

    private static int left() {
        trace = (trace * 10) + 1;
        return 101;
    }

    private static int middle() {
        trace = (trace * 10) + 2;
        return 202;
    }

    private static int fallback() {
        trace = (trace * 10) + 3;
        return 303;
    }

    public static void reset() {
        trace = 0;
    }

    public static int trace() {
        return trace;
    }

    public static int choose(int i) {
        int iFallback;
        switch (i) {
            case 0:
                iFallback = left();
                break;
            case 1:
                iFallback = middle();
                break;
            default:
                iFallback = fallback();
                break;
        }
        return iFallback;
    }
}
