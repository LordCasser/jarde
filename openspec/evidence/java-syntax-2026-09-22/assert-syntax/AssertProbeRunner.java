public final class AssertProbeRunner {
    private AssertProbeRunner() {
    }

    public static void main(String[] args) {
        System.out.println("assertions=" + AssertProbe.class.desiredAssertionStatus());

        AssertProbe.reset();
        String success;
        try {
            AssertProbe.succeeds();
            success = "returned";
        } catch (Throwable thrown) {
            success = thrown.getClass().getName();
        }
        print("success", success, "-", "-");

        AssertProbe.reset();
        String failure;
        String failureMessage = "-";
        try {
            AssertProbe.fails();
            failure = "returned";
        } catch (Throwable thrown) {
            failure = thrown.getClass().getName();
            if (thrown instanceof AssertionError) {
                failureMessage = thrown.getMessage();
            }
        }
        print("failure", failure, failureMessage, "-");

        AssertProbe.reset();
        RuntimeException conditionSentinel = new IllegalStateException("condition-sentinel");
        String conditionOutcome;
        boolean conditionIdentity = false;
        try {
            AssertProbe.conditionException(conditionSentinel);
            conditionOutcome = "returned";
        } catch (Throwable thrown) {
            conditionOutcome = thrown.getClass().getName();
            conditionIdentity = thrown == conditionSentinel;
        }
        print("condition-exception", conditionOutcome, "-", String.valueOf(conditionIdentity));

        AssertProbe.reset();
        RuntimeException messageSentinel = new IllegalArgumentException("message-sentinel");
        String messageOutcome;
        boolean messageIdentity = false;
        try {
            AssertProbe.messageException(messageSentinel);
            messageOutcome = "returned";
        } catch (Throwable thrown) {
            messageOutcome = thrown.getClass().getName();
            messageIdentity = thrown == messageSentinel;
        }
        print("message-exception", messageOutcome, "-", String.valueOf(messageIdentity));
    }

    private static void print(String name, String outcome, String detail, String same) {
        System.out.println(name + ": outcome=" + outcome
                + ", detail=" + detail
                + ", same=" + same
                + ", conditionCalls=" + AssertProbe.conditionCalls
                + ", messageCalls=" + AssertProbe.messageCalls
                + ", trace=" + AssertProbe.trace);
    }
}
