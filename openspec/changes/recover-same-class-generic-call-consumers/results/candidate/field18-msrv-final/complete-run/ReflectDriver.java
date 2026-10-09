import java.lang.reflect.*;
import java.util.*;
public class ReflectDriver {
  static String show(Object x) {
    if (x == null) return "null";
    if (x.getClass().isArray() && x instanceof Object[]) return Arrays.deepToString((Object[])x);
    return String.valueOf(x);
  }
  static Object argument(Class<?> parameter, String method, Class<?> owner) {
    if (parameter==boolean.class) return Boolean.TRUE;
    if (Map.class.isAssignableFrom(parameter)) return new HashMap<String,String>(Collections.singletonMap("raw","value"));
    if (parameter.isArray()) return new String[]{method};
    if (List.class.isAssignableFrom(parameter)) return new ArrayList<String>(Collections.singletonList("raw"));
    if (owner.getSimpleName().equals("NullSetter")) return null;
    if (method.equals("putObject")) return "object";
    return "put";
  }
  static Field field(Class<?> c) throws Exception {
    try { return c.getDeclaredField("v"); }
    catch (NoSuchFieldException ignored) {
      try { return c.getDeclaredField("index"); }
      catch (NoSuchFieldException ignoredAgain) { return null; }
    }
  }
  public static void main(String[] a) throws Exception {
    Class<?> c=Class.forName((a[2].equals("defpackage") ? "defpackage." : "")+a[0]);
    Object o=a[1].equals("ctor") ? c.getConstructor(Object.class).newInstance("ctor") : c.getConstructor().newInstance();
    for (int i=3;i<a.length;i++) {
      String method=a[i];
      Method m=null;
      for (Method candidate:c.getDeclaredMethods()) if (candidate.getName().equals(method)) { m=candidate; break; }
      if (m==null) throw new NoSuchMethodException(method);
      if (m.getParameterTypes().length==0) m.invoke(o);
      else { Class<?>[] ps=m.getParameterTypes(); Object[] xs=new Object[ps.length]; for(int k=0;k<ps.length;k++) xs[k]=argument(ps[k],method,c); m.invoke(o,xs); }
    }
    Field f=field(c);
    if (f==null) System.out.println("fieldValue=<none>\nfieldGenericType=<none>");
    else { f.setAccessible(true); System.out.println("fieldValue="+show(f.get(o))); System.out.println("fieldGenericType="+f.getGenericType().getTypeName()); }
    System.out.println("classvars="+c.getTypeParameters().length);
    Constructor<?>[] cs=c.getDeclaredConstructors();
    Arrays.sort(cs, new Comparator<Constructor<?>>() { public int compare(Constructor<?> x, Constructor<?> y) { return x.toString().compareTo(y.toString()); }});
    for (Constructor<?> k:cs) for (Type t:k.getGenericParameterTypes()) System.out.println("ctorParameterGenericType="+t.getTypeName());
    Method[] ms=c.getDeclaredMethods();
    Arrays.sort(ms, new Comparator<Method>() { public int compare(Method x, Method y) { return x.getName().compareTo(y.getName()); }});
    for (Method k:ms) {
      Type[] p=k.getGenericParameterTypes();
      if (p.length==0) System.out.println("method="+k.getName()+" methodvars="+k.getTypeParameters().length+" parameterGenericType=<none>");
      else for (Type t:p) System.out.println("method="+k.getName()+" methodvars="+k.getTypeParameters().length+" parameterGenericType="+t.getTypeName());
    }
  }
}
