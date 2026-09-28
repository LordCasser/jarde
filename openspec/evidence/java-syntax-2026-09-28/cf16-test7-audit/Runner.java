package jadx.tests.integration.trycatch;

public class Runner {
	public static void main(String[] args) {
		Object instance;
		try {
			instance = Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls")
					.getDeclaredConstructor().newInstance();
		} catch (ReflectiveOperationException error) {
			throw new AssertionError(error);
		}
		for (Object input : new Object[] { null, "r", "ok" }) {
			try {
				boolean result = (Boolean) invoke(instance, input);
				System.out.println(String.valueOf(input) + ":return=" + result + ",f=" + readF(instance));
			} catch (Throwable error) {
				System.out.println(String.valueOf(input) + ":throw=" + error.getClass().getSimpleName() + ",f=" + readF(instance));
			}
			resetF(instance);
		}
	}

	private static Object invoke(Object instance, Object input) throws Exception {
		java.lang.reflect.Method method = instance.getClass().getDeclaredMethod("test", Object.class);
		method.setAccessible(true);
		try {
			return method.invoke(instance, input);
		} catch (java.lang.reflect.InvocationTargetException error) {
			Throwable cause = error.getCause();
			if (cause instanceof Exception) {
				throw (Exception) cause;
			}
			if (cause instanceof Error) {
				throw (Error) cause;
			}
			throw new AssertionError(cause);
		}
	}

	private static int readF(Object instance) {
		try {
			java.lang.reflect.Field field = instance.getClass().getDeclaredField("f");
			field.setAccessible(true);
			return field.getInt(instance);
		} catch (ReflectiveOperationException error) {
			throw new AssertionError(error);
		}
	}

	private static void resetF(Object instance) {
		try {
			java.lang.reflect.Field field = instance.getClass().getDeclaredField("f");
			field.setAccessible(true);
			field.setInt(instance, 0);
		} catch (ReflectiveOperationException error) {
			throw new AssertionError(error);
		}
	}
}
