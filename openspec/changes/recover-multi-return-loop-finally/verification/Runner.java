package jadx.tests.integration.trycatch;

import java.lang.reflect.InvocationHandler;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public final class Runner {
	private static final class Scenario implements InvocationHandler {
		final String name;
		final boolean cNull;
		final boolean first;
		final int iterations;
		final String failure;
		final boolean closeFails;
		final List<String> events = new ArrayList<>();
		final IllegalStateException failureError;
		final IllegalStateException closeError = new IllegalStateException("close-failure");
		int loaded;
		int nextCalls;
		Object d;

		Scenario(String name, boolean cNull, boolean first, int iterations, String failure, boolean closeFails) {
			this.name = name;
			this.cNull = cNull;
			this.first = first;
			this.iterations = iterations;
			this.failure = failure;
			this.closeFails = closeFails;
			this.failureError = new IllegalStateException(failure + "-failure");
		}

		Object proxy(Class<?>... interfaces) {
			return Proxy.newProxyInstance(targetClass().getClassLoader(), interfaces, this);
		}

		@Override public Object invoke(Object proxy, Method method, Object[] args) {
			String m = method.getName();
			if (m.equals("toString")) return name;
			if (m.equals("hashCode")) return System.identityHashCode(proxy);
			if (m.equals("equals")) return proxy == args[0];
			if (m.equals("p")) return cNull ? null : args[0];
			if (m.equals("f")) { events.add("f"); return d = proxy(find(targetClass().getDeclaredClasses(), "D")); }
			if (m.equals("first")) { events.add("first"); if ("first".equals(failure)) throw failureError; return first; }
			if (m.equals("load")) {
				events.add("load" + (loaded + 1));
				if ("load".equals(failure)) throw failureError;
				loaded++;
				return "v" + loaded;
			}
			if (m.equals("toNext")) { events.add("toNext" + (++nextCalls)); if ("toNext".equals(failure)) throw failureError; return nextCalls < iterations; }
			if (m.equals("close")) {
				events.add("close");
				if (closeFails) throw closeError;
				return null;
			}
			throw new AssertionError("unexpected method " + method);
		}
	}

	private static void run(String name, boolean cNull, boolean first, int iterations, String failure, boolean closeFails) throws Exception {
		Scenario s = new Scenario(name, cNull, first, iterations, failure, closeFails);
		Class<?>[] nested = targetClass().getDeclaredClasses();
		Class<?> a = find(nested, "A");
		Class<?> b = find(nested, "B");
		Object ac = cNull ? null : s.proxy(a, find(nested, "C"));
		Object bp = s.proxy(b);
		Method test = targetClass().getDeclaredMethod("test", a, b);
		test.setAccessible(true);
		Object target = targetClass().newInstance();
		try {
			Object result = test.invoke(target, ac, bp);
			System.out.println(name + " result=" + result + " events=" + s.events);
		} catch (InvocationTargetException e) {
			Throwable t = e.getCause();
			System.out.println(name + " throws=" + t.getClass().getName() + ":" + t.getMessage() + " identity=" + (t == s.closeError ? "close" : t == s.failureError ? "body" : "other") + " events=" + s.events);
		}
	}

	private static Class<?> find(Class<?>[] classes, String suffix) {
		for (Class<?> c : classes) if (c.getSimpleName().equals(suffix)) return c;
		try { return Class.forName(targetClass().getName() + "$" + suffix); }
		catch (ClassNotFoundException e) { throw new AssertionError("missing interface " + suffix + " in " + Arrays.toString(classes)); }
	}

	private static Class<?> targetClass() {
		try {
			return Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls");
		} catch (ClassNotFoundException e) {
			throw new AssertionError(e);
		}
	}

	public static void main(String[] args) throws Exception {
		run("c_null", true, true, 1, "", false);
		run("first_false", false, false, 1, "", false);
		run("one_iteration", false, true, 1, "", false);
		run("two_iterations", false, true, 2, "", false);
		run("first_exception", false, true, 1, "first", false);
		run("load_exception", false, true, 1, "load", false);
		run("toNext_exception", false, true, 1, "toNext", false);
		run("close_overrides_return", false, true, 1, "", true);
		run("close_overrides_body_error", false, true, 1, "load", true);
	}
}
