package jadx.tests.integration.trycatch;
public class Runner {
    static void run(String label, int body, boolean fin) {
        TestTryCatchFinally16.TRACE.setLength(0);
        TestTryCatchFinally16.BODY_MODE = body;
        TestTryCatchFinally16.THROW_FINALLY = fin;
        String result = "ok";
        try { new TestTryCatchFinally16.TestCls().test(); }
        catch (Throwable t) { result = t.getClass().getSimpleName() + ":" + t.getMessage(); }
        System.out.println(label + "=" + TestTryCatchFinally16.TRACE + result);
    }
    public static void main(String[] args) {
        run("normal", 0, false);
        run("caught", 1, false);
        run("error", 2, false);
        run("finally", 0, true);
        run("caught-finally", 1, true);
        run("error-finally", 2, true);
    }
}
