package jadx.tests.integration.trycatch;

import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;

/* JADX INFO: loaded from: debug.jar:jadx/tests/integration/trycatch/TestTryCatchFinally6$TestCls.class */
public class TestTryCatchFinally6$TestCls {
    public static void test() throws IOException {
        InputStream is = null;
        try {
            call();
            InputStream is2 = new FileInputStream("1.txt");
            if (is2 != null) {
            }
        } finally {
            if (is != null) {
                is.close();
            }
        }
    }

    private static void call() {
    }
}
