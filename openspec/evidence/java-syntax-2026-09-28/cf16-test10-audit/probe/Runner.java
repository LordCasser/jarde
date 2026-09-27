package trycatch;

import android.content.Context;
import android.content.res.Resources;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

public final class Runner {
	private static final StringBuilder EVENTS = new StringBuilder();

	private static final class ProbeStream extends ByteArrayInputStream {
		private final String mode;
		private boolean thrown;

		ProbeStream(String text, String mode) {
			super(text.getBytes(java.nio.charset.StandardCharsets.UTF_8));
			this.mode = mode;
		}

		@Override
		public void close() throws IOException {
			EVENTS.append("close;");
			if (!thrown && (mode.equals("close-io") || mode.equals("logger-runtime"))) {
				thrown = true;
				throw new IOException("close");
			}
			if (mode.equals("close-runtime")) {
				throw new IllegalStateException("close");
			}
			super.close();
		}
	}

	private static Context context(final String mode) {
		return new Context() {
			@Override
			public Resources getResources() {
				return new Resources() {
					@Override
				public InputStream openRawResource(int id) {
						EVENTS.append("open;");
						if (mode.equals("open-throw")) {
							throw new IllegalStateException("open");
						}
						return new ProbeStream(mode.equals("empty") ? "" : "payload", mode);
					}
				};
			}
		};
	}

	public static void main(String[] args) {
		String mode = args[0];
		EVENTS.setLength(0);
		ProbeState.events = EVENTS;
		ProbeState.mode = mode;
		try {
			String result = TestTryCatchFinally10.test(context(mode), 7);
			System.out.println(mode + ":return:" + result + ":" + EVENTS);
		} catch (Throwable th) {
			System.out.println(mode + ":throw:" + th.getClass().getSimpleName() + ":" + th.getMessage() + ":" + EVENTS);
		}
	}
}
