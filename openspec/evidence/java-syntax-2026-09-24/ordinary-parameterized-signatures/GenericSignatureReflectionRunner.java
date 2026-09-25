import java.lang.reflect.Method;
import java.util.List;
public class GenericSignatureReflectionRunner {
  public static void main(String[] args) throws Exception {
    Method m = RawBodyMismatch.class.getMethod("body", List.class);
    System.out.println("generic-parameter=" + m.getGenericParameterTypes()[0].getTypeName()
        + " generic-return=" + m.getGenericReturnType().getTypeName());
  }
}
