public final class ExceptionOnlyLocalDriver {
    private static void check(String label, RuntimeException pending) {
        ExceptionOnlyLocal.trace = 0;
        Throwable observed = null;
        try {
            ExceptionOnlyLocal.run(pending);
        } catch (Throwable thrown) {
            observed = thrown;
        }
        if (ExceptionOnlyLocal.trace != 78) {
            throw new AssertionError(label + ": trace=" + ExceptionOnlyLocal.trace);
        }
        if (pending == null) {
            if (!(observed instanceof NullPointerException)) {
                throw new AssertionError(label + ": expected NullPointerException, got " + observed);
            }
            System.out.println(label + ":trace=78:type=" + observed.getClass().getName());
        } else {
            if (observed != pending) {
                throw new AssertionError(label + ": exception identity changed");
            }
            System.out.println(label + ":trace=78:identity=true:type="
                    + observed.getClass().getName() + ":message=" + observed.getMessage());
        }
    }

    public static void main(String[] args) {
        check("null", null);
        check("state", new IllegalStateException("local-state"));
        check("argument", new IllegalArgumentException("local-argument"));
    }
}
