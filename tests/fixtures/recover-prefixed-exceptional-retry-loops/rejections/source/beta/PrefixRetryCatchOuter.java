public final class PrefixRetryCatchOuter {
    public int trace;
    public int outerPrefixCalls;
    public int retryPrefixCalls;
    public int protectedCalls;
    public int catchCalls;
    public int outerIterations;
    public RuntimeException lastCaught;
    public String lastCaughtMessage;

    public final IllegalArgumentException prefixEscape = new IllegalArgumentException("alpha-prefix");
    public final AssertionError protectedEscape = new AssertionError("alpha-protected-error");
    public final AssertionError catchEscape = new AssertionError("alpha-catch-effect");

    private int failuresRemaining;

    public PrefixRetryCatchOuter() {
    }

    public int execute(int failures, int outerLimit, int mode) {
        failuresRemaining = bounded(failures, 0, 1);
        int limit = bounded(outerLimit, 1, 2);

        outer: while (true) {
            f1(mode);
            outerIterations++;
            int stage = 0;
            while (true) {
                f2(mode);
                if (stage != 0) {
                    break outer;
                }
                stage += 3;
                if (outerIterations > limit) {
                    break outer;
                }
                try {
                    protectedStep(mode);
                    break;
                } catch (IllegalArgumentException failure) {
                    catchCalls++;
                    recordCatch(failure, mode);
                    continue outer;
                }
            }
        }
        record(5);
        return trace;
    }

    private void f1(int mode) {
        outerPrefixCalls++;
        record(1);
        if ((mode & 1) != 0) {
            throw prefixEscape;
        }
    }

    private void f2(int mode) {
        retryPrefixCalls++;
        record(2);
        if ((mode & 2) != 0) {
            throw prefixEscape;
        }
    }

    private void protectedStep(int mode) {
        protectedCalls++;
        record(3);
        if (failuresRemaining > 0) {
            failuresRemaining--;
            throw new IllegalArgumentException("alpha-retry");
        }
        if ((mode & 4) != 0) {
            throw protectedEscape;
        }
    }

    private void recordCatch(IllegalArgumentException failure, int mode) {
        lastCaught = failure;
        lastCaughtMessage = failure.getMessage();
        record(4);
        if ((mode & 8) != 0) {
            throw catchEscape;
        }
    }

    private void record(int event) {
        trace = trace * 10 + event;
    }

    private static int bounded(int value, int min, int max) {
        return Math.max(min, Math.min(value, max));
    }
}
