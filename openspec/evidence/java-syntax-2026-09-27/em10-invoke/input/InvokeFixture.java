package em10;

import java.io.IOException;

public class InvokeFixture extends InvokeBase {
	private int receivers;

	private InvokeWorker receiver() {
		receivers++;
		return new InvokeWorker();
	}

	private void fail() throws IOException {
		throw new IOException("io");
	}

	long run() {
		return receiver().combine(1, 2L, 3.0d, 4)
				+ InvokeTools.combine(5, 6L, 7.0d, 8)
				+ InvokeFixture.inherited(9L, 10)
				+ runCatch();
	}

	private long runCatch() {
		try {
			fail();
		} catch (IOException exception) {
			return receiver().combine(11, 12L, 13.0d, 14)
					+ InvokeTools.caught(exception.getMessage()).length();
		}
		return 0;
	}

	public static void main(String[] args) {
		System.out.println(new InvokeFixture().run());
	}
}
