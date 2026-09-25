public class FinallyCompletion {
    public static int trace;
    public static final RuntimeException TRY_FAILURE = new IllegalArgumentException("try");
    public static final RuntimeException FINALLY_FAILURE = new IllegalStateException("finally");

    private static int mark(int digit) {
        trace = trace * 10 + digit;
        return digit;
    }

    public static int normalReturn() {
        try {
            return mark(1);
        } finally {
            mark(2);
        }
    }

    public static int finallyReturns() {
        try {
            return mark(3);
        } finally {
            return mark(4);
        }
    }

    public static int finallyThrows() {
        try {
            return mark(5);
        } finally {
            mark(6);
            throw FINALLY_FAILURE;
        }
    }

    public static int tryThrowsFinallyRuns() {
        try {
            mark(7);
            throw TRY_FAILURE;
        } finally {
            mark(8);
        }
    }
}
