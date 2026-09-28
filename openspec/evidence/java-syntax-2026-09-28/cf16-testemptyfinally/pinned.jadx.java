package jadx.tests.integration.trycatch;

import java.io.FileInputStream;
import java.io.IOException;

/* JADX INFO: loaded from: TestEmptyFinally$TestCls.class */
public class TestEmptyFinally$TestCls {
    public void test(FileInputStream f1) {
        try {
            f1.close();
        } catch (IOException e) {
        }
    }
}
