import java.lang.reflect.*;
import java.util.*;

public final class ConstructorDriver {
  private static Object value(Class<?> c, int index) {
    if (c == long.class) return Long.valueOf(17L);
    if (c == double.class) return Double.valueOf(2.5);
    if (c == boolean.class) return Boolean.TRUE;
    if (c == int.class) return Integer.valueOf(3);
    if (c == short.class) return Short.valueOf((short)2);
    if (c == byte.class) return Byte.valueOf((byte)1);
    if (c == char.class) return Character.valueOf('x');
    if (c == float.class) return Float.valueOf(1.5f);
    if (c.isArray()) return new String[] {"a", "b"};
    if (Number.class.isAssignableFrom(c)) return Integer.valueOf(9);
    return "arg" + index;
  }
  private static String type(Type t) { return t.getTypeName(); }
  private static String valueText(Object value) {
    if (value == null) return "null";
    if (!value.getClass().isArray()) return String.valueOf(value);
    int n=Array.getLength(value);
    StringBuilder b=new StringBuilder("[");
    for (int i=0;i<n;i++) { if (i>0) b.append(", "); b.append(valueText(Array.get(value,i))); }
    return b.append("]").toString();
  }
  private static String binder(Type t, Class<?> owner) {
    if (t instanceof GenericArrayType) return binder(((GenericArrayType)t).getGenericComponentType(), owner) + "[]";
    if (!(t instanceof TypeVariable)) return "non-variable";
    GenericDeclaration d = ((TypeVariable<?>)t).getGenericDeclaration();
    if (d == owner) return "class";
    if (d instanceof Constructor) return "constructor";
    return d.toString();
  }
  public static void main(String[] args) throws Exception {
    Class<?> c = Class.forName(args[0]);
    Constructor<?>[] constructors = c.getDeclaredConstructors();
    Arrays.sort(constructors, new Comparator<Constructor<?>>() {
      public int compare(Constructor<?> a, Constructor<?> b) { return a.toGenericString().compareTo(b.toGenericString()); }
    });
    Constructor<?> chosen = null;
    for (Constructor<?> k : constructors) if (chosen == null || k.getParameterTypes().length > chosen.getParameterTypes().length) chosen=k;
    for (int ci=0; ci<constructors.length; ci++) {
      Constructor<?> k=constructors[ci];
      Type[] parameters=k.getGenericParameterTypes();
      TypeVariable<?>[] ctorVars=k.getTypeParameters();
      System.out.println("REFLECT|ctor["+ci+"].formalCount="+parameters.length+";typeVariableCount="+ctorVars.length);
      for (int i=0; i<parameters.length; i++) {
        Type p=parameters[i];
        System.out.println("REFLECT|ctor["+ci+"].param["+i+"]="+type(p)+";binder="+binder(p,c));
      }
      for (int i=0; i<ctorVars.length; i++) {
        StringBuilder b=new StringBuilder();
        for (Type bound:ctorVars[i].getBounds()) { if (b.length()>0) b.append('&'); b.append(type(bound)); }
        System.out.println("REFLECT|ctor["+ci+"].typeVar["+i+"]="+ctorVars[i].getName()+";bounds="+b+";binder="+binder(ctorVars[i],c));
      }
    }
    Object[] actual = new Object[chosen.getParameterTypes().length];
    for (int i=0; i<actual.length; i++) actual[i]=value(chosen.getParameterTypes()[i], i);
    Object instance=chosen.newInstance(actual);
    if (c.getName().equals("RawNewHold")) {
      Object factory=c.getMethod("make", Object.class).invoke(null, "factory");
      Field field=c.getDeclaredField("v"); field.setAccessible(true);
      System.out.println("BEHAVIOR|rawFactory.field="+valueText(field.get(factory)));
    }
    if (c.getName().equals("PeerNewHold")) {
      Constructor<?> oneArg=null;
      for (Constructor<?> k:constructors) if (k.getParameterTypes().length==1) oneArg=k;
      if (oneArg==null) throw new AssertionError("missing one-argument constructor");
      Object peer=oneArg.newInstance(value(oneArg.getParameterTypes()[0], 0));
      Field field=c.getDeclaredField("v"); field.setAccessible(true);
      System.out.println("BEHAVIOR|peerNew.field="+valueText(field.get(peer)));
    }
    TypeVariable<?>[] classVars=c.getTypeParameters();
    System.out.println("REFLECT|classTypeVariableCount="+classVars.length);
    for (int i=0;i<classVars.length;i++) {
      TypeVariable<?> v=classVars[i];
      StringBuilder b=new StringBuilder();
      for (Type bound:v.getBounds()) { if (b.length()>0) b.append('&'); b.append(type(bound)); }
      System.out.println("REFLECT|classVar["+i+"]="+v.getName()+";bounds="+b);
    }
    Field[] fields=c.getDeclaredFields();
    Arrays.sort(fields,new Comparator<Field>() { public int compare(Field a,Field b){return a.getName().compareTo(b.getName());} });
    for (Field f:fields) {
      if (Modifier.isStatic(f.getModifiers())) continue;
      f.setAccessible(true);
      Type t=f.getGenericType();
      System.out.println("REFLECT|field["+f.getName()+"]="+type(t)+";binder="+binder(t,c));
      System.out.println("BEHAVIOR|field["+f.getName()+"]="+valueText(f.get(instance)));
    }
    System.out.println("BEHAVIOR|constructed="+c.getName());
  }
}
