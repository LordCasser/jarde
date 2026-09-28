package jadx.tests.integration.trycatch;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
public class Runner {
  public static void main(String[] args) throws Exception {
    Path file = Paths.get("1.txt");
    Files.write(file, new byte[]{1});
    try { TestTryCatchFinally6$TestCls.test(); System.out.println("normal:ok"); }
    finally { Files.deleteIfExists(file); }
    try { TestTryCatchFinally6$TestCls.test(); System.out.println("missing:ok"); }
    catch (java.io.FileNotFoundException expected) { System.out.println("missing:FileNotFoundException"); }
  }
}
