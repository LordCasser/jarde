import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
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
	private static final RuntimeException LOOKUP_FAILURE = new RuntimeException("lookup-failure");
	private static final RuntimeException READ_FAILURE = new RuntimeException("read-failure");
	private static final IOException CLOSE_FAILURE = new IOException("close-failure");
	private static int closeCount;
	private static int readFailureCount;

	private Runner() {
	}

	public static void main(String[] args) throws Exception {
		Path classes = Paths.get(args[0]).toAbsolutePath();
		for (String scenario : new String[] {
			"present", "missing", "lookup-failure", "read-failure", "close-failure", "read-and-close-failure"
		}) {
			run(scenario, classes);
		}
	}

	private static void run(String scenario, Path classes) throws Exception {
		closeCount = 0;
		readFailureCount = 0;
		URLStreamHandler handler = new URLStreamHandler() {
			@Override
			protected URLConnection openConnection(URL url) {
				if ("lookup-failure".equals(scenario)) {
					throw LOOKUP_FAILURE;
				}
				return new URLConnection(url) {
					@Override
				public void connect() {
				}

					@Override
				public InputStream getInputStream() {
					ByteArrayInputStream content = new ByteArrayInputStream("resource-data".getBytes(StandardCharsets.UTF_8));
					return new InputStream() {
						@Override
						public int read() {
							if (scenario.startsWith("read")) {
								readFailureCount++;
								throw READ_FAILURE;
							}
							return content.read();
						}

						@Override
						public int read(byte[] bytes, int offset, int length) {
							if (scenario.startsWith("read")) {
								readFailureCount++;
								throw READ_FAILURE;
							}
							return content.read(bytes, offset, length);
						}

						@Override
						public void close() throws IOException {
							closeCount++;
							if (scenario.contains("close-failure")) {
								throw CLOSE_FAILURE;
							}
							content.close();
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
					if ("missing".equals(scenario)) {
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
				System.out.println(scenario + " result=" + result + " close=" + closeCount + " exception=none identity=none readFailure=" + readFailureCount);
			} catch (InvocationTargetException e) {
				Throwable cause = e.getCause();
				String identity = cause == LOOKUP_FAILURE ? "lookup" : cause == READ_FAILURE ? "read"
					: cause == CLOSE_FAILURE ? "close" : "other";
				System.out.println(scenario + " result=<threw> close=" + closeCount
					+ " exception=" + cause.getClass().getName() + " identity=" + identity + " readFailure=" + readFailureCount);
			}
		}
	}
}
