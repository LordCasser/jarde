public final class RetryPrefixBeta {
    public int trace;
    public int outerPrefixCalls;
    public int retryPrefixCalls;
    public int protectedCalls;
    public int catchCalls;
    public int outerIterations;
    public RuntimeException lastCaught;
    public String lastCaughtMessage;

    public final IllegalStateException prefixEscape = new IllegalStateException("beta-prefix");
    public final AssertionError protectedEscape = new AssertionError("beta-protected-error");
    public final AssertionError catchEscape = new AssertionError("beta-catch-effect");

    private int injectedFailures;

    public RetryPrefixBeta() {
    }

    public int execute(int failures, int outerLimit, int mode) {
        injectedFailures = cap(failures, 0, 1);
        int limit = cap(outerLimit, 1, 2);

        outer: while (true) {
            beginOuter(mode);
            outerIterations++;
            int stage = 0;
            while (true) {
                beginRetry(mode);
                if (stage != 0) {
                    break outer;
                }
                stage += 7;
                if (outerIterations > limit) {
                    break outer;
                }
                try {
                    performProtected(mode);
                    break;
                } catch (IllegalStateException caught) {
                    catchCalls++;
                    rememberCatch(caught, mode);
                }
            }
        }
        append(5);
        return trace;
    }

    private void beginOuter(int mode) {
        outerPrefixCalls++;
        append(2);
        if ((mode & 1) != 0) {
            throw prefixEscape;
        }
    }

    private void beginRetry(int mode) {
        retryPrefixCalls++;
        append(1);
        if ((mode & 2) != 0) {
            throw prefixEscape;
        }
    }

    private void performProtected(int mode) {
        protectedCalls++;
        append(4);
        if (injectedFailures != 0) {
            injectedFailures--;
            throw new IllegalStateException("beta-again");
        }
        if ((mode & 4) != 0) {
            throw protectedEscape;
        }
    }

    private void rememberCatch(IllegalStateException caught, int mode) {
        lastCaught = caught;
        lastCaughtMessage = caught.getMessage();
        append(3);
        if ((mode & 8) != 0) {
            throw catchEscape;
        }
    }

    private void append(int event) {
        trace = trace * 10 + event;
    }

    private static int cap(int value, int lower, int upper) {
        return Math.max(lower, Math.min(value, upper));
    }
}
