package jadx.tests.integration.trycatch;

import java.io.FileInputStream;
import java.io.IOException;

public class TestEmptyFinally$TestCls {
	public void test(FileInputStream f1) {
		try {
			f1.close();
		} catch (IOException e) {
			// do nothing
		} finally {
			// ignore
		}
	}
}
