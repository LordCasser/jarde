public final class ExceptionChain {
    static boolean result;
    static boolean rhsValue;
    static boolean throwRhs;
    static int calls;
    static int caught;

    static boolean rhs() {
        calls++;
        if (throwRhs) {
            throw new IllegalStateException("rhs");
        }
        return rhsValue;
    }

    static boolean assign(boolean extra, boolean left) {
        try {
            result = extra || left || rhs();
        } catch (RuntimeException ex) {
            caught++;
        }
        return result;
    }

    private static void run(String label, boolean extra, boolean left, boolean rhs, boolean throwsRhs) {
        result = false;
        rhsValue = rhs;
        throwRhs = throwsRhs;
        calls = 0;
        caught = 0;
        boolean value = assign(extra, left);
        System.out.println(label + ":result=" + value + ",calls=" + calls + ",caught=" + caught);
    }

    public static void main(String[] args) {
        run("extra", true, false, false, false);
        run("left", false, true, false, false);
        run("rhs-true", false, false, true, false);
        run("rhs-false", false, false, false, false);
        run("rhs-throws", false, false, false, true);
    }
}
