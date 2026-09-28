package jadx.tests.integration.trycatch;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
public class TestTryCatchFinally6$TestCls {
  public static void test() throws IOException {
    InputStream is = null;
    try {
      call();
      is = new FileInputStream("1.txt");
    } finally {
      if (is != null) {
        is.close();
      }
    }
  }
  private static void call() {}
}
