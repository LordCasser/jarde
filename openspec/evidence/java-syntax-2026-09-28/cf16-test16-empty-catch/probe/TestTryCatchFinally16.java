package jadx.tests.integration.trycatch;

public class TestTryCatchFinally16 {
    public static final StringBuilder TRACE = new StringBuilder();
    public static int BODY_MODE;
    public static boolean THROW_FINALLY;
	public static class TestCls {
		public void test() {
			try {
				TCls.doSomething();
			} catch (Exception e) {
				// do nothing
			} finally {
				TCls.doFinally();
			}
		}

		private static class TCls {
			public static void doSomething() {
                TRACE.append("body,");
                if (BODY_MODE == 1) throw new IllegalStateException("body");
                if (BODY_MODE == 2) throw new AssertionError("body");
			}

			public static void doFinally() {
                TRACE.append("finally,");
                if (THROW_FINALLY) throw new IllegalArgumentException("finally");
			}
		}
	}

}
