package jadx.tests.integration.trycatch;

import java.io.FileInputStream;
import java.io.IOException;

/* JADX INFO: loaded from: nodebug.jar:jadx/tests/integration/trycatch/TestTryCatchFinally6$TestCls.class */
public class TestTryCatchFinally6$TestCls {
    public static void test() throws IOException {
        FileInputStream fileInputStream = null;
        try {
            call();
            FileInputStream fileInputStream2 = new FileInputStream("1.txt");
            if (fileInputStream2 != null) {
            }
        } finally {
            if (fileInputStream != null) {
                fileInputStream.close();
            }
        }
    }

    private static void call() {
    }
}
