public final class CarryUnprovedConsumerProbe {
    public static String trace = "";
    public static int effects;

    public CarryUnprovedConsumerProbe() {
    }

    public static int dimension() {
        trace += "D";
        return 1;
    }

    public static boolean test(boolean f) {
        trace += "T";
        return f;
    }

    public static int arm() {
        trace += "A";
        return 2;
    }

    public static void consume(int[][] a) {
        trace += "C";
        effects = a.length * 100 + a[0].length;
    }

    public static void candidate(boolean f) {
        consume(new int[dimension()][test(f) ? arm() : 3]);
    }
}
