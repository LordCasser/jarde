/** Runtime oracle for body, close, suppression identity, and failed initialization. */
public final class TwrAuditRunner {
    private static void check(boolean condition, String message) {
        if (!condition) {
            throw new AssertionError(message);
        }
    }

    private static void state(String scenario, int opens, int closes, int initializations, int trace) {
        check(TwrAudit.opens() == opens, scenario + " opens=" + TwrAudit.opens());
        check(TwrAudit.closes() == closes, scenario + " closes=" + TwrAudit.closes());
        check(TwrAudit.initializations() == initializations,
                scenario + " initializations=" + TwrAudit.initializations());
        check(TwrAudit.trace() == trace, scenario + " trace=" + TwrAudit.trace());
        System.out.println(scenario + ":opens=" + opens + ",closes=" + closes
                + ",initializations=" + initializations + ",trace=" + trace);
    }

    public static void main(String[] args) throws Exception {
        TwrAudit.reset();
        TwrAudit.normalBodyAndClose();
        state("normal", 1, 1, 0, 91);

        TwrAudit.reset();
        Exception body = new Exception("body");
        Exception closing = new Exception("close");
        Exception primary = null;
        try {
            TwrAudit.bodyAndFailingClose(body, closing);
        } catch (Exception failure) {
            primary = failure;
        }
        check(primary == body, "body failure must remain primary by identity");
        Throwable[] suppressed = primary.getSuppressed();
        check(suppressed.length == 1, "one close failure must be suppressed");
        check(suppressed[0] == closing, "the original close failure object must be suppressed");
        state("body-and-close-fail", 1, 1, 0, 91);

        TwrAudit.reset();
        Exception initializer = new Exception("initializer");
        Exception initializationFailure = null;
        try {
            TwrAudit.failingInitialization(initializer);
        } catch (Exception failure) {
            initializationFailure = failure;
        }
        check(initializationFailure == initializer, "initializer failure identity must be preserved");
        state("initializer-fails-before-resource", 0, 0, 1, 0);
    }
}
