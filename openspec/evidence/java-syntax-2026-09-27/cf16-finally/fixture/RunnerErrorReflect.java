import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

public final class RunnerErrorReflect {
  public static void main(String[] args) throws Exception {
    Class<?> type = Class.forName(args[0]);
    Object instance = type.getDeclaredConstructor().newInstance();
    Method test = type.getDeclaredMethod("test", Object.class);
    test.setAccessible(true);
    Error marker = new AssertionError("catchall-marker");
    try {
      test.invoke(instance, marker);
      throw new AssertionError("Error was swallowed");
    } catch (InvocationTargetException caught) {
      if (caught.getCause() != marker) throw new AssertionError("original Error identity lost", caught);
    }
    if (!type.getField("f").getBoolean(instance)) throw new AssertionError("finally field was not set");
    System.out.println("catchall=original-error,f=true");
  }
}
