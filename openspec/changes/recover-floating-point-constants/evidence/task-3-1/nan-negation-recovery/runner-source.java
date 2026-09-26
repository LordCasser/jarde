import java.nio.file.*;
public class NanFoldRunner {
  public static void main(String[] args) throws Exception {
    Class<?> c = Class.forName("FloatingConstants");
    java.lang.reflect.Method f = c.getDeclaredMethod("floatNan");
    java.lang.reflect.Method d = c.getDeclaredMethod("doubleNan");
    System.out.println("floatNan=" + Integer.toHexString(Float.floatToRawIntBits((float) f.invoke(null))));
    System.out.println("doubleNan=" + Long.toHexString(Double.doubleToRawLongBits((double) d.invoke(null))));
  }
}
