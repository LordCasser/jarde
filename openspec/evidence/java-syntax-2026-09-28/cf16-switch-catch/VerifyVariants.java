package jadx.tests.integration.trycatch;

public final class VerifyVariants {
	public static void main(String[] args) throws Exception {
		for (String name : new String[] {
				"PartialSwitchCatch1", "PartialSwitchCatch2", "PartialSwitchCatch3", "TwrSwitchCatch"}) {
			Class<?> type = Class.forName("jadx.tests.integration.trycatch." + name);
			Object instance = type.getConstructor().newInstance();
			String result = (String) type.getMethod("runTest", int.class, int.class).invoke(instance, 1, 0);
			if (!"call-out-finally".equals(result)) {
				throw new AssertionError(name + " returned " + result);
			}
			System.out.println(name + "=verified,runTest(1,0)=" + result);
		}
	}
}
