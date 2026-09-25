/** Multi-resource TWR case for reverse close and suppressed order. */
public final class TwrMultiAudit implements AutoCloseable {
    private static int closes;
    private static int trace;

    private final int id;
    private final Exception closeFailure;

    private TwrMultiAudit(int id, Exception closeFailure) {
        this.id = id;
        this.closeFailure = closeFailure;
    }

    private static TwrMultiAudit open(int id, Exception closeFailure) {
        return new TwrMultiAudit(id, closeFailure);
    }

    private static void failBody(Exception failure) throws Exception {
        throw failure;
    }

    private static void event(int value) {
        trace = trace * 10 + value;
    }

    public static void reset() {
        closes = 0;
        trace = 0;
    }

    public static int closes() {
        return closes;
    }

    public static int trace() {
        return trace;
    }

    public static void bodyAndThreeFailingCloses(Exception body, Exception closeOne,
                                                 Exception closeTwo, Exception closeThree)
            throws Exception {
        try (TwrMultiAudit one = open(1, closeOne);
             TwrMultiAudit two = open(2, closeTwo);
             TwrMultiAudit three = open(3, closeThree)) {
            event(9);
            failBody(body);
        }
    }

    @Override
    public void close() throws Exception {
        closes++;
        event(id);
        if (closeFailure != null) {
            throw closeFailure;
        }
    }
}
