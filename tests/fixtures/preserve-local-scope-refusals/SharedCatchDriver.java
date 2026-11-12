public final class SharedCatchDriver {
    static int calls;
    static final StringBuilder trace = new StringBuilder();
    static Object alias;
    static void require(boolean ok, String detail) {
        if (!ok) throw new AssertionError(detail);
    }
    static final class ArgumentChild extends IllegalArgumentException {
        ArgumentChild() { super((String) null); }
    }
    static final class StateChild extends IllegalStateException {
        StateChild() { super((String) null); }
    }
    static void one(final Throwable thrown, boolean caught, int index) {
        final Throwable cause = new Throwable("cause " + index);
        final Throwable first = new Throwable("first " + index);
        final Throwable second = new Throwable("second " + index);
        if (thrown != null) {
            thrown.initCause(cause);
            thrown.addSuppressed(first);
            thrown.addSuppressed(second);
        }
        final Object marker = new Object();
        Runnable action = new Runnable() {
            public void run() {
                ++calls;
                trace.append("run:");
                alias = marker;
                trace.append("alias:");
                if (thrown instanceof RuntimeException) {
                    trace.append("throw:");
                    throw (RuntimeException) thrown;
                }
                if (thrown instanceof Error) {
                    trace.append("throw:");
                    throw (Error) thrown;
                }
                trace.append("done:");
            }
        };
        calls = 0; trace.setLength(0); alias = null;
        Object result = null;
        Throwable escaped = null;
        try { result = ScopeRefusalsEscape.sharedHandler(action); }
        catch (Throwable value) { escaped = value; }
        trace.append("after");
        require(calls == 1 && alias == marker, "one invocation and alias before completion");
        require(trace.toString().equals(thrown == null ? "run:alias:done:after" : "run:alias:throw:after"), "event order");
        if (thrown == null) require(result == action && escaped == null, "normal runnable identity");
        else {
            require(caught ? result == thrown && escaped == null : escaped == thrown && result == null, "exception identity");
            require(thrown.getCause() == cause, "cause identity");
            Throwable[] suppressed = thrown.getSuppressed();
            require(suppressed.length == 2 && suppressed[0] == first && suppressed[1] == second, "suppressed identities/order");
            require(thrown.getMessage() == null, "null message unchanged");
        }
    }
    public static void main(String[] args) {
        int count = 0;
        for (int repetition = 0; repetition < 12; ++repetition) {
            one(null, false, count++);
            one(new IllegalArgumentException((String) null), true, count++);
            one(new RuntimeException((String) null), false, count++);
            one(new IllegalStateException((String) null), true, count++);
            one(new NullPointerException(), false, count++);
            one(new ArgumentChild(), true, count++);
            one(new AssertionError(), false, count++);
            one(new StateChild(), true, count++);
            Throwable escaped = null;
            try { ScopeRefusalsEscape.sharedHandler(null); }
            catch (Throwable value) { escaped = value; }
            require(escaped instanceof NullPointerException, "null action throws NPE");
            require(escaped.getCause() == null && escaped.getSuppressed().length == 0, "null action metadata");
            ++count;
        }
        System.out.println("shared-catch cases=" + count + " identity/cause/suppression/order=OK");
    }
}
