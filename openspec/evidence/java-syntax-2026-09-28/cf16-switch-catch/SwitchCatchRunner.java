package jadx.tests.integration.trycatch;

public final class SwitchCatchRunner {
	public static void main(String[] args) {
		String[] expected = {
				"call-out-finally", "call-npe-catch-out-finally", "call-iae-finally",
				"call-finally", "call-npe-catch-finally", "call-iae-finally",
				"call-finally", "call-npe-catch-finally", "call-iae-finally"
		};
		int i = 0;
		for (int test = 1; test <= 3; test++) {
			for (int exception = 0; exception <= 2; exception++) {
				String actual = new SwitchCatchMinimal().runTest(test, exception);
				if (!expected[i].equals(actual)) {
					throw new AssertionError(test + "," + exception + ": expected=" + expected[i] + " actual=" + actual);
				}
				System.out.println(test + "," + exception + "=" + actual);
				i++;
			}
		}
	}
}
