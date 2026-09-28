package jadx.tests.integration.trycatch;

/* JADX INFO: loaded from: debug.jar:jadx/tests/integration/trycatch/TestTryCatchFinally7.class */
public class TestTryCatchFinally7 {

    /* JADX INFO: loaded from: debug.jar:jadx/tests/integration/trycatch/TestTryCatchFinally7$TestCls.class */
    public static class TestCls {
        private int f = 0;

        private boolean test(Object obj) {
            boolean res;
            try {
                res = exc(obj);
            } catch (Exception e) {
                res = false;
            } finally {
                this.f++;
            }
            return res;
        }

        private boolean exc(Object obj) throws Exception {
            if ("r".equals(obj)) {
                throw new AssertionError();
            }
            return true;
        }
    }
}
