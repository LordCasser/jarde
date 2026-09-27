package jadx.tests.integration.trycatch;

public class TestTryCatchFinally14 {
    public static final StringBuilder TRACE = new StringBuilder();
    public static boolean CLEAR_T;
    public static boolean THROW_BODY;
    public static boolean THROW_FINALLY;

    public static class TestCls {
        private TCls t;

        public TestCls(boolean present) {
            if (present) {
                t = new TCls(this);
            }
        }

        public void test() {
            try {
                if (t != null) {
                    t.doSomething();
                }
            } finally {
                if (t != null) {
                    t.doFinally();
                }
            }
        }

        private static class TCls {
            private final TestCls owner;

            TCls(TestCls owner) {
                this.owner = owner;
            }

            public void doSomething() {
                TRACE.append("body,");
                if (CLEAR_T) {
                    owner.t = null;
                }
                if (THROW_BODY) {
                    throw new IllegalStateException("body");
                }
            }

            public void doFinally() {
                TRACE.append("finally,");
                if (THROW_FINALLY) {
                    throw new IllegalArgumentException("finally");
                }
            }
        }
    }
}
