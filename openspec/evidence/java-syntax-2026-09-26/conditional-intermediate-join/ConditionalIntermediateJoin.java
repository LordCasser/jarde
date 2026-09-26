public final class ConditionalIntermediateJoin {
    private static String trace = "";
    private static int fail;

    private static int f1() {
        trace += "f1,";
        if (fail == 1) throw new IllegalStateException("f1");
        return 10;
    }

    private static int f2() {
        trace += "f2,";
        if (fail == 2) throw new IllegalStateException("f2");
        return 20;
    }

    private static int f3() {
        trace += "f3,";
        if (fail == 3) throw new IllegalStateException("f3");
        return 30;
    }

    public static int choose(int a) {
        return a > 0 ? ((a > 1 ? f1() : f2()) + 3) : f3();
    }

    public static void reset(int nextFail) {
        trace = "";
        fail = nextFail;
    }

    public static String trace() {
        return trace;
    }
}
