import java.lang.reflect.*;
public class NestedCallProbe {
  static int failures;
  static void check(boolean ok, String label) {
    System.out.println("check."+label+"="+ok);
    if (!ok) failures++;
  }
  public static void main(String[] args) throws Exception {
    Class<?> c=Class.forName(args[0]);
    TypeVariable<?>[] vars=c.getTypeParameters();
    check(vars.length==1, "one-class-formal");
    TypeVariable<?> t=vars[0];
    check(t.getGenericDeclaration()==c, "class-formal-owner");
    for(String name:new String[]{"first","second","relay"}) {
      Method m=c.getDeclaredMethod(name,Object.class);
      check(m.getGenericReturnType().equals(t), name+"-return-is-class-T");
      check(m.getGenericParameterTypes()[0].equals(t), name+"-parameter-is-class-T");
      check(((TypeVariable<?>)m.getGenericReturnType()).getGenericDeclaration()==c,
            name+"-return-declaration-is-class");
      System.out.println("method="+name+";return="+m.getGenericReturnType().getTypeName()
          +";parameter="+m.getGenericParameterTypes()[0].getTypeName());
    }
    Object receiver=c.getConstructor().newInstance();
    Object marker=new Object();
    Object returned=c.getDeclaredMethod("relay",Object.class).invoke(receiver,marker);
    check(returned==marker, "nested-result-marker-identity");
    System.out.println("behavior=nested-result-marker-identity");
    System.out.println("probe.failures="+failures);
    if(failures!=0) System.exit(1);
  }
}
