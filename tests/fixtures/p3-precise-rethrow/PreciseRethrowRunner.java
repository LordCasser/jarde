public final class PreciseRethrowRunner {
    private static void precise(String label, int mode) {
        PreciseRethrowProbe.trace = "";
        try {
            int value = PreciseRethrowProbe.precise(mode);
            System.out.println(label + "=return:" + value + ":trace=" + PreciseRethrowProbe.trace);
        } catch (Throwable failure) {
            System.out.println(label + "=throw:" + failure.getClass().getName() + ":"
                    + failure.getMessage() + ":trace=" + PreciseRethrowProbe.trace);
        }
    }

    public static void main(String[] args) {
        precise("precise-normal", 0);
        precise("precise-parse", 1);
        precise("precise-io", 2);
    }
}
