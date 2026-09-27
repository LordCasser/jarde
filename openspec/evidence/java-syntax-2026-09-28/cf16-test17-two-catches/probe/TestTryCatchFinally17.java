package jadx.tests.integration.trycatch;

public class TestTryCatchFinally17 {
    public static final StringBuilder TRACE = new StringBuilder();
    public static int BODY_MODE;
    public static boolean THROW_FINALLY;
	public static class TestCls {
		public int test() {
			try {
				TCls.doSomething();
			} catch (UnsupportedOperationException e) {
				// do nothing
			} catch (NullPointerException e) {
				return 1;
			} finally {
				TCls.doFinally();
			}
			return 0;
		}

		private static class TCls {
			public static void doSomething() {
                TRACE.append("body,");
                if (BODY_MODE == 1) throw new UnsupportedOperationException("unsupported");
                if (BODY_MODE == 2) throw new NullPointerException("null");
                if (BODY_MODE == 3) throw new AssertionError("error");
			}

			public static void doFinally() {
                TRACE.append("finally,");
                if (THROW_FINALLY) throw new IllegalArgumentException("finally");
			}
		}
	}

}
