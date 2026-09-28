package jadx.tests.integration.trycatch;

import java.io.IOException;
import java.io.InputStream;
import java.util.Scanner;

public class TestTryCatchFinally9$TestCls {
	public String test() throws IOException {
		InputStream input = null;
		try {
			input = this.getClass().getResourceAsStream("resource");
			Scanner scanner = new Scanner(input).useDelimiter("\\A");
			return scanner.hasNext() ? scanner.next() : "";
		} finally {
			if (input != null) {
				input.close();
			}
		}
	}
}
