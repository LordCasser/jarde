import java.lang.reflect.*;
import java.util.*;
public class HeaderProbe {
  public static void main(String[] args) throws Exception {
    Class<?> c = Class.forName(args[0]);
    Method[] ms = c.getDeclaredMethods();
    Arrays.sort(ms, new Comparator<Method>() { public int compare(Method a, Method b) { return a.toString().compareTo(b.toString()); }});
    for (Method m : ms) {
      if (m.getName().equals("main")) continue;
      System.out.print("header=" + m.getName() + "<");
      for (TypeVariable<Method> v : m.getTypeParameters()) System.out.print(v.getName() + ":" + Arrays.toString(v.getBounds()) + "@" + v.getGenericDeclaration() + ";");
      System.out.print(">( ");
      for (Type t : m.getGenericParameterTypes()) System.out.print(t.getTypeName() + ";");
      System.out.println(") -> " + m.getGenericReturnType().getTypeName());
    }
    String probe = args[1];
    Object receiver = c.getConstructor().newInstance();
    Object marker = new Object();
    if (probe.equals("VoidDirect")) {
      c.getMethod("relay", Object.class).invoke(receiver, marker);
      System.out.println("behavior.marker=" + (c.getField("seen").get(receiver) == marker));
    } else if (probe.equals("ArrayRelay")) {
      Object[] value = new Object[]{marker};
      System.out.println("behavior.marker=" + (c.getMethod("relay", Object[].class).invoke(receiver, (Object)value) == value));
    } else if (probe.equals("NumberBoundRelay") || probe.equals("MethodShadow")) {
      Integer value = Integer.valueOf(17);
      System.out.println("behavior.marker=" + (c.getMethod("relay", Number.class).invoke(receiver, value) == value));
    } else if (probe.equals("IndependentCallee")) {
      System.out.println("behavior.marker=" + (c.getMethod("relay", Object.class).invoke(receiver, marker) == marker));
    } else if (probe.equals("MultiParam")) {
      System.out.println("behavior.marker=" + (c.getMethod("relay", Object.class, Object.class).invoke(receiver, marker, marker) == marker));
    } else if (probe.equals("RawReceiver") || probe.equals("MutatedReceiver")) {
      System.out.println("behavior.marker=" + (c.getMethod("relay", Object.class).invoke(receiver, marker) == marker));
    } else if (probe.equals("SameNameOverload")) {
      c.getMethod("relay", Object.class).invoke(receiver, marker);
      System.out.println("behavior.selected=" + c.getField("selected").get(receiver));
    } else if (probe.equals("SameErasureBinder")) {
      System.out.println("behavior.null=" + (c.getMethod("relay", Number.class).invoke(receiver, new Object[]{null}) == null));
    }
  }
}
