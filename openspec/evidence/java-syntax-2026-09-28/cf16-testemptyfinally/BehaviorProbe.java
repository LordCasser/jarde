package jadx.tests.integration.trycatch;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

public final class BehaviorProbe {
	private static final class ProbeStream extends FileInputStream {
		private final String outcome;
		private final IOException io = new IOException("close-io");
		private final IllegalStateException runtime = new IllegalStateException("close-runtime");
		private int closeCalls;

		ProbeStream(String outcome) throws IOException {
			super(File.createTempFile("cf16-empty-finally", ".tmp"));
			this.outcome = outcome;
		}

		@Override
		public void close() throws IOException {
			closeCalls++;
			super.close();
			if ("io".equals(outcome)) {
				throw io;
			}
			if ("runtime".equals(outcome)) {
				throw runtime;
			}
		}
	}

	public static void main(String[] args) throws Exception {
		Class<?> type = Class.forName(args[0]);
		Object target = type.getConstructor().newInstance();
		Method test = type.getMethod("test", FileInputStream.class);
		for (String mode : new String[] { "success", "io", "runtime" }) {
			ProbeStream stream = new ProbeStream(mode);
			String result = "return";
			boolean sameThrowable = true;
			try {
				test.invoke(target, stream);
			} catch (InvocationTargetException ex) {
				Throwable cause = ex.getCause();
				result = cause.getClass().getSimpleName() + ":" + cause.getMessage();
				sameThrowable = cause == ("runtime".equals(mode) ? stream.runtime : stream.io);
			}
			System.out.println(mode + " result=" + result + " closeCalls=" + stream.closeCalls
					+ " sameThrowable=" + sameThrowable);
		}
	}
}
