package jadx.tests.integration.trycatch;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStream;

public class TestTryCatchFinally4$TestCls {
	public void test() throws IOException {
		File file = File.createTempFile("test", "txt");
		OutputStream outputStream = new FileOutputStream(file);
		try {
			outputStream.write(1);
		} finally {
			try {
				outputStream.close();
				file.delete();
			} catch (IOException ignored) {
			}
		}
	}
}
