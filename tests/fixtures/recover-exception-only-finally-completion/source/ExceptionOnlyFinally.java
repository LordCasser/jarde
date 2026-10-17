public class ExceptionOnlyFinally {
    public static int trace = 0;
    public static final RuntimeException TRY_FAILURE = new IllegalArgumentException("try");

    public ExceptionOnlyFinally() {
    }

    private static int mark(int digit) {
        trace = trace * 10 + digit;
        System.out.print(digit);
        return digit;
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
