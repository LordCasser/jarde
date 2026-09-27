package jadx.tests.integration.trycatch;

public class TestTryCatchFinally16 {
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
			}

			public static void doFinally() {
			}
		}
	}

}
