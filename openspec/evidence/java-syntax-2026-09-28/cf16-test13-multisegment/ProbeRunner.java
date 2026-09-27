package jadx.tests.integration.trycatch;

public class ProbeRunner {
	private static void run(String label, int input, int throwAt, String expected) {
		TestTryCatchFinally13$TestCls.TRACE.setLength(0);
		TestTryCatchFinally13$TestCls.THROW_AT = throwAt;
		TestTryCatchFinally13$TestCls.LAST_FAILURE = null;
		new TestTryCatchFinally13$TestCls().test(input);
		String actual = TestTryCatchFinally13$TestCls.TRACE.toString();
		if (!expected.equals(actual)) {
			throw new AssertionError(label + " expected=" + expected + " actual=" + actual);
		}
		int finallyCount = actual.split("finally,", -1).length - 1;
		if (finallyCount != 1) {
			throw new AssertionError(label + " finallyCount=" + finallyCount);
		}
		System.out.println(label + "=" + actual + " finallyCount=" + finallyCount);
	}

	public static void main(String[] args) {
		run("early-return", -12, -1, "do1,finally,");
		run("normal-then", 11, -1, "do1,do2,finally,");
		run("normal-else", 0, -1, "do1,finally,");
		run("normal-else-if", -1, -1, "do1,do3,finally,");
		run("catch-site-1", 0, 1, "do1,throw:IllegalArgumentException,catch:IllegalArgumentException,finally,");
		run("catch-site-2", 11, 2, "do1,do2,throw:IllegalStateException,catch:IllegalStateException,finally,");
		run("catch-site-3", -1, 3, "do1,do3,throw:UnsupportedOperationException,catch:UnsupportedOperationException,finally,");
	}
}
