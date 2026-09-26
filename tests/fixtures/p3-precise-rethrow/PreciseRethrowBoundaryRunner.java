public final class PreciseRethrowBoundaryRunner {
    private static void anyFinally(String label, int mode) {
        PreciseRethrowBoundaryProbe.trace = "";
        try {
            int value = PreciseRethrowBoundaryProbe.anyFinally(mode);
            System.out.println(label + "=return:" + value + ":trace=" + PreciseRethrowBoundaryProbe.trace);
        } catch (Throwable failure) {
            System.out.println(label + "=throw:" + failure.getClass().getName() + ":"
                    + failure.getMessage() + ":trace=" + PreciseRethrowBoundaryProbe.trace);
        }
    }

    private static void multiCatch(String label, int mode) {
        PreciseRethrowBoundaryProbe.trace = "";
        try {
            int value = PreciseRethrowBoundaryProbe.multiCatch(mode);
            System.out.println(label + "=return:" + value + ":trace=" + PreciseRethrowBoundaryProbe.trace);
        } catch (Throwable failure) {
            System.out.println(label + "=throw:" + failure.getClass().getName() + ":"
                    + failure.getMessage() + ":trace=" + PreciseRethrowBoundaryProbe.trace);
        }
    }

    private static void changedValue(String label, int mode) {
        PreciseRethrowBoundaryProbe.trace = "";
        try {
            int value = PreciseRethrowBoundaryProbe.changedValue(mode);
            System.out.println(label + "=return:" + value + ":trace=" + PreciseRethrowBoundaryProbe.trace);
        } catch (Throwable failure) {
            System.out.println(label + "=throw:" + failure.getClass().getName() + ":"
                    + failure.getMessage() + ":trace=" + PreciseRethrowBoundaryProbe.trace);
        }
    }

    public static void main(String[] args) {
        anyFinally("finally-normal", 0);
        anyFinally("finally-throw", 1);
        multiCatch("multi-parse", 1);
        multiCatch("multi-io", 2);
        changedValue("changed-normal", 0);
        changedValue("changed-replacement", 1);
    }
}
