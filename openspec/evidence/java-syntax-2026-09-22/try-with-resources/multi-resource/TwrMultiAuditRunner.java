/** Runtime oracle for reverse close order and each suppressed exception's identity. */
public final class TwrMultiAuditRunner {
    private static void check(boolean condition, String message) {
        if (!condition) {
            throw new AssertionError(message);
        }
    }

    public static void main(String[] args) throws Exception {
        TwrMultiAudit.reset();
        Exception body = new Exception("body");
        Exception closeOne = new Exception("close-one");
        Exception closeTwo = new Exception("close-two");
        Exception closeThree = new Exception("close-three");
        Exception primary = null;
        try {
            TwrMultiAudit.bodyAndThreeFailingCloses(body, closeOne, closeTwo, closeThree);
        } catch (Exception failure) {
            primary = failure;
        }
        check(primary == body, "body failure must remain primary by identity");
        Throwable[] suppressed = primary.getSuppressed();
        check(suppressed.length == 3, "all close failures must be suppressed");
        check(suppressed[0] == closeThree, "close-three must be first suppressed");
        check(suppressed[1] == closeTwo, "close-two must be second suppressed");
        check(suppressed[2] == closeOne, "close-one must be third suppressed");
        check(TwrMultiAudit.closes() == 3, "every resource must close once");
        check(TwrMultiAudit.trace() == 9321, "body then close-three, close-two, close-one");
        System.out.println("suppressed-order-3-2-1:closes=" + TwrMultiAudit.closes()
                + ",trace=" + TwrMultiAudit.trace());
    }
}
