package jadx.tests.integration.trycatch;

public class TestTryCatchFinally7 {
    public static class TestCls {
        private int f = 0;

        private boolean test(Object obj) {
            boolean res;
            try {
                res = exc(obj);
            } catch (Exception e) {
                res = false;
            } finally {
                f++;
            }
            return res;
        }

        private boolean exc(Object obj) throws Exception {
            if ("e".equals(obj)) {
                throw new Exception("typed");
            }
            if (obj instanceof AssertionError) {
                throw (AssertionError) obj;
            }
            return true;
        }
    }
}
