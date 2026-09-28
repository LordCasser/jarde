import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.net.URLConnection;
import java.net.URLStreamHandler;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.nio.file.Paths;

public final class Runner {
	private static int closeCount;

	private Runner() {
	}

	public static void main(String[] args) throws Exception {
		Path classes = Paths.get(args[0]).toAbsolutePath();
		run("present", classes, true);
		run("missing", classes, false);
	}

	private static void run(String label, Path classes, boolean resourcePresent) throws Exception {
		closeCount = 0;
		URLStreamHandler handler = new URLStreamHandler() {
			@Override
			protected URLConnection openConnection(URL url) {
				return new URLConnection(url) {
					@Override
				public void connect() {
				}

					@Override
				public java.io.InputStream getInputStream() {
						return new ByteArrayInputStream("resource-data".getBytes(StandardCharsets.UTF_8)) {
							@Override
						public void close() throws IOException {
							closeCount++;
							super.close();
						}
						};
					}
				};
			}
		};
		try (URLClassLoader loader = new URLClassLoader(new URL[] { classes.toUri().toURL() }, Runner.class.getClassLoader()) {
			@Override
			public URL getResource(String name) {
				if ("jadx/tests/integration/trycatch/resource".equals(name)) {
					if (!resourcePresent) {
						return null;
					}
					try {
						return new URL(null, "track://fixture/resource", handler);
					} catch (java.net.MalformedURLException e) {
						throw new IllegalStateException(e);
					}
				}
				return super.getResource(name);
			}
		}) {
			Class<?> cls = Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally9$TestCls", true, loader);
			Object instance = cls.getConstructor().newInstance();
			Method test = cls.getMethod("test");
			try {
				Object result = test.invoke(instance);
				System.out.println(label + " result=" + result + " close=" + closeCount + " exception=none");
			} catch (InvocationTargetException e) {
				Throwable cause = e.getCause();
				System.out.println(label + " result=<threw> close=" + closeCount + " exception=" + cause.getClass().getName());
			}
		}
	}
}
