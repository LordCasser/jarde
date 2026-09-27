package jadx.tests.integration.trycatch;

import java.lang.reflect.Field;

public final class PinnedFixedRunner {
	public static void main(String[] args) throws Exception {
		Class<?> type = Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally12$TestCls");
		Field sb = type.getDeclaredField("sb");
		sb.setAccessible(true);
		String[] expected = {
				"call-out-finally", "call-npe-catch-out-finally", "call-iae-finally",
				"call-finally", "call-npe-catch-finally", "call-iae-finally",
				"call-finally", "call-npe-catch-finally", "call-iae-finally"
		};
		int i = 0;
		for (int test = 1; test <= 3; test++) {
			for (int exception = 0; exception <= 2; exception++) {
				Object instance = type.getConstructor().newInstance();
				StringBuilder value = new StringBuilder();
				sb.set(instance, value);
				String actual = (String) type.getMethod("runTest", int.class, int.class).invoke(instance, test, exception);
				if (!expected[i].equals(actual)) throw new AssertionError(test + "," + exception + ": " + actual);
				System.out.println(test + "," + exception + "=" + actual);
				i++;
			}
		}
	}
}
