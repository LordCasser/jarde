package jadx.tests.integration.trycatch;

public class Runner {
    private static void run(String name, boolean present, boolean clear, boolean bodyThrow, boolean finallyThrow) {
        TestTryCatchFinally14.TRACE.setLength(0);
        TestTryCatchFinally14.CLEAR_T = clear;
        TestTryCatchFinally14.THROW_BODY = bodyThrow;
        TestTryCatchFinally14.THROW_FINALLY = finallyThrow;
        Throwable thrown = null;
        try {
            new TestTryCatchFinally14.TestCls(present).test();
        } catch (Throwable error) {
            thrown = error;
        }
        System.out.println(name + "=" + TestTryCatchFinally14.TRACE + (thrown == null ? "ok" : thrown.getClass().getSimpleName() + ":" + thrown.getMessage()));
    }

    public static void main(String[] args) {
        run("null", false, false, false, false);
        run("normal", true, false, false, false);
        run("clear", true, true, false, false);
        run("body-throw", true, false, true, false);
        run("clear-throw", true, true, true, false);
        run("finally-throw", true, false, false, true);
        run("both-throw", true, false, true, true);
    }
}
