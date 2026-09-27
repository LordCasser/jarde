import java.lang.reflect.Method;
public final class RunnerReflect {
  public static void main(String[] args) throws Exception {
    Class<?> c = Class.forName(args[0]);
    Object x = c.getDeclaredConstructor().newInstance();
    Method test = c.getDeclaredMethod("test", Object.class);
    test.setAccessible(true);
    System.out.println("normal=" + test.invoke(x, "a") + ",f=" + c.getField("f").get(x));
    System.out.println("exception=" + test.invoke(x, new Object[] { null }) + ",f=" + c.getField("f").get(x));
    c.getMethod("check").invoke(x);
    System.out.println("check=passed");
  }
}
