package jadx.tests.integration.trycatch;

public class TestTryCatchFinally17 {
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
			}

			public static void doFinally() {
			}
		}
	}

}
