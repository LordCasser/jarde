public final class IntermediateJoinNegative {
    private static int effects;

    private static int f1() { return 10; }
    private static int f2() { return 20; }
    private static int f3() { return 30; }
    private static int side() { effects++; return 3; }

    public static int independentEffect(int a) {
        return a > 0 ? ((a > 1 ? f1() : f2()) + side()) : f3();
    }

    public static int throwingBridge(int a) {
        return a > 0 ? ((a > 1 ? f1() : f2()) / 3) : f3();
    }

    public static int caughtBridge(int a) {
        try {
            return a > 0 ? ((a > 1 ? f1() : f2()) / (a - 1)) : f3();
        } catch (ArithmeticException error) {
            return -1;
        }
    }

    public static int externalEntry(int a) {
        return a > 0 ? ((a == 1 ? 7 : (a > 1 ? f1() : f2())) + 3) : f3();
    }
}
