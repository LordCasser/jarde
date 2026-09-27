package jadx.tests.integration.trycatch;
public class Runner {
    static void run(String label, int body, boolean fin) {
        TestTryCatchFinally17.TRACE.setLength(0);
        TestTryCatchFinally17.BODY_MODE = body;
        TestTryCatchFinally17.THROW_FINALLY = fin;
        String result;
        try { result = Integer.toString(new TestTryCatchFinally17.TestCls().test()); }
        catch (Throwable t) { result = t.getClass().getSimpleName() + ":" + t.getMessage(); }
        System.out.println(label + "=" + TestTryCatchFinally17.TRACE + result);
    }
    public static void main(String[] args) {
        run("normal", 0, false);
        run("unsupported", 1, false);
        run("null", 2, false);
        run("error", 3, false);
        run("finally", 0, true);
        run("unsupported-finally", 1, true);
        run("null-finally", 2, true);
        run("error-finally", 3, true);
    }
}
