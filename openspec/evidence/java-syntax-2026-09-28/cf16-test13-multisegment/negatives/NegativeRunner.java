package jadx.tests.integration.trycatch;

public class NegativeRunner {
	public static void main(String[] args) {
		if (args.length != 1) {
			throw new IllegalArgumentException("scenario required");
		}
		String scenario = args[0];
		int input;
		String expectedTrace;
		String expectedThrown;
		TestTryCatchFinally13$TestCls.THROW_AT = -1;
		TestTryCatchFinally13$TestCls.THROW_ERROR_AT = -1;
		TestTryCatchFinally13$TestCls.THROW_FINALLY_ONCE = false;
		TestTryCatchFinally13$TestCls.LAST_FAILURE = null;
		switch (scenario) {
			case "cleanup-target":
				input = -12;
				expectedTrace = "do1,do3,";
				expectedThrown = "none";
				break;
			case "branch-bypass":
				input = 0;
				expectedTrace = "do1,";
				expectedThrown = "none";
				break;
			case "range-expanded":
				input = -12;
				TestTryCatchFinally13$TestCls.THROW_FINALLY_ONCE = true;
				expectedTrace = "do1,finally,throw-finally,finally,";
				expectedThrown = "java.lang.AssertionError";
				break;
			case "range-control":
				input = -12;
				TestTryCatchFinally13$TestCls.THROW_FINALLY_ONCE = true;
				expectedTrace = "do1,finally,throw-finally,";
				expectedThrown = "java.lang.AssertionError";
				break;
			case "rethrow-changed":
				input = 0;
				TestTryCatchFinally13$TestCls.THROW_ERROR_AT = 1;
				expectedTrace = "do1,throw:AssertionError,finally,";
				expectedThrown = "java.lang.NullPointerException";
				break;
			default:
				throw new IllegalArgumentException("unknown scenario: " + scenario);
		}

		TestTryCatchFinally13$TestCls.TRACE.setLength(0);
		Throwable thrown = null;
		try {
			new TestTryCatchFinally13$TestCls().test(input);
		} catch (Throwable ex) {
			thrown = ex;
		}
		String actualTrace = TestTryCatchFinally13$TestCls.TRACE.toString();
		String actualThrown = thrown == null ? "none" : thrown.getClass().getName();
		if (!expectedTrace.equals(actualTrace) || !expectedThrown.equals(actualThrown)) {
			throw new AssertionError(scenario + " expected=" + expectedTrace + "/" + expectedThrown
					+ " actual=" + actualTrace + "/" + actualThrown);
		}
		int finallyCount = 0;
		for (String event : actualTrace.split(",", -1)) {
			if ("finally".equals(event)) {
				finallyCount++;
			}
		}
		System.out.println(scenario + " trace=" + actualTrace + " thrown=" + actualThrown
				+ " finallyCount=" + finallyCount);
	}
}
