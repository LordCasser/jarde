package jadx.tests.integration.trycatch;

import java.io.IOException;
import java.io.InputStream;
import java.util.Scanner;

/* JADX INFO: loaded from: <java-class-input> */
public class TestTryCatchFinally9$TestCls {
    public String test() throws IOException {
        InputStream input = null;
        try {
            InputStream input2 = getClass().getResourceAsStream("resource");
            Scanner scanner = new Scanner(input2).useDelimiter("\\A");
            String next = scanner.hasNext() ? scanner.next() : "";
            if (input2 != null) {
            }
            return next;
        } finally {
            if (input != null) {
                input.close();
            }
        }
    }
}
