package jadx.tests.integration.trycatch;

import java.io.File;
import java.io.IOException;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.List;

public class ControlRunner {
	private static final class ProbeStream extends OutputStream {
		final String mode;
		final List<String> events;

		ProbeStream(String mode, List<String> events) {
			this.mode = mode;
			this.events = events;
		}

		@Override
		public void write(int value) throws IOException {
			events.add("write:" + value);
			if (mode.equals("body-io")) {
				throw new IOException("body");
			}
		}

		@Override
		public void close() throws IOException {
			events.add("close");
			if (mode.equals("cleanup-io")) {
				throw new IOException("cleanup");
			}
			if (mode.equals("cleanup-runtime")) {
				throw new IllegalStateException("cleanup");
			}
		}
	}

	private static final class ProbeFile extends File {
		private static final long serialVersionUID = 1L;
		final List<String> events;

		ProbeFile(List<String> events) {
			super("probe");
			this.events = events;
		}

		@Override
		public boolean delete() {
			events.add("delete");
			return true;
		}
	}

	public static void main(String[] args) throws Exception {
		for (String mode : new String[] { "normal", "body-io", "cleanup-io", "cleanup-runtime" }) {
			List<String> events = new ArrayList<String>();
			String thrown = "none";
			try {
				Control.run(new ProbeStream(mode, events), new ProbeFile(events));
			} catch (IOException e) {
				thrown = "IOException:" + e.getMessage();
			} catch (RuntimeException e) {
				thrown = e.getClass().getSimpleName() + ":" + e.getMessage();
			}
			System.out.println(mode + "|" + events + "|" + thrown);
		}
	}
}
