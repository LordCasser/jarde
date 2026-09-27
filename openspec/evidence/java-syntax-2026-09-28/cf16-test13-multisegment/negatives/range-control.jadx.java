package jadx.tests.integration.trycatch;

/* JADX INFO: loaded from: TestTryCatchFinally13$TestCls.class */
public class TestTryCatchFinally13$TestCls {
    public static final StringBuilder TRACE = new StringBuilder();
    public static int THROW_AT = -1;
    public static int THROW_ERROR_AT = -1;
    public static boolean THROW_FINALLY_ONCE;
    public static RuntimeException LAST_FAILURE;

    public void test(int i) {
        try {
            doSomething1();
            if (i == -12) {
                return;
            }
            if (i > 10) {
                doSomething2();
            } else if (i == -1) {
                doSomething3();
            }
        } catch (Exception e) {
            logError();
        } finally {
            doSomething4();
        }
    }

    void logError() {
        TRACE.append("catch:").append(LAST_FAILURE.getClass().getSimpleName()).append(',');
    }

    void doSomething1() {
        invoke(1, "do1");
    }

    void doSomething2() {
        invoke(2, "do2");
    }

    void doSomething3() {
        invoke(3, "do3");
    }

    void doSomething4() {
        TRACE.append("finally,");
        if (THROW_FINALLY_ONCE) {
            THROW_FINALLY_ONCE = false;
            TRACE.append("throw-finally,");
            throw new AssertionError("injected cleanup failure");
        }
    }

    private static void invoke(int site, String name) {
        TRACE.append(name).append(',');
        if (THROW_ERROR_AT == site) {
            TRACE.append("throw:AssertionError,");
            throw new AssertionError("injected original failure");
        }
        if (THROW_AT == site) {
            if (site == 1) {
                LAST_FAILURE = new IllegalArgumentException();
            } else if (site == 2) {
                LAST_FAILURE = new IllegalStateException();
            } else {
                LAST_FAILURE = new UnsupportedOperationException();
            }
            TRACE.append("throw:").append(LAST_FAILURE.getClass().getSimpleName()).append(',');
            throw LAST_FAILURE;
        }
    }
}
