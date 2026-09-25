/** Minimal Java 8 TWR core; exception identities are supplied by the runner. */
public final class TwrAudit implements AutoCloseable {
    private static int opens;
    private static int closes;
    private static int initializations;
    private static int trace;

    private final int id;
    private final Exception closeFailure;

    private TwrAudit(int id, Exception closeFailure) {
        this.id = id;
        this.closeFailure = closeFailure;
    }

    private static TwrAudit open(int id, Exception closeFailure) {
        opens++;
        return new TwrAudit(id, closeFailure);
    }

    private static TwrAudit failOpen(Exception failure) throws Exception {
        initializations++;
        throw failure;
    }

    private static void failBody(Exception failure) throws Exception {
        throw failure;
    }

    private static void event(int value) {
        trace = trace * 10 + value;
    }

    public static void reset() {
        opens = 0;
        closes = 0;
        initializations = 0;
        trace = 0;
    }

    public static int opens() {
        return opens;
    }

    public static int closes() {
        return closes;
    }

    public static int initializations() {
        return initializations;
    }

    public static int trace() {
        return trace;
    }

    public static void normalBodyAndClose() throws Exception {
        try (TwrAudit resource = open(1, null)) {
            event(9);
        }
    }

    public static void bodyAndFailingClose(Exception body, Exception closeFailure) throws Exception {
        try (TwrAudit resource = open(1, closeFailure)) {
            event(9);
            failBody(body);
        }
    }

    public static void failingInitialization(Exception failure) throws Exception {
        try (TwrAudit resource = failOpen(failure)) {
            event(8);
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
