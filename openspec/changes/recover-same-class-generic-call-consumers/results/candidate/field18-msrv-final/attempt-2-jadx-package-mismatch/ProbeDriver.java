import java.lang.reflect.*;
import java.util.*;
public class ProbeDriver {
  public static void main(String[] a) throws Exception {
    Class<?> c=Class.forName(a[0]); Object o=c.getConstructor().newInstance();
    for(Method m:c.getDeclaredMethods()) if(m.getName().equals("put")) {
      Class<?>[] ps=m.getParameterTypes(); Object[] xs=new Object[ps.length];
      for(int i=0;i<ps.length;i++) {
        if(ps[i]==long.class) xs[i]=17L;
        else if(ps[i]==double.class) xs[i]=2.5;
        else if(ps[i].isArray()) xs[i]=new List[]{Collections.singletonList("array")};
        else if(List.class.isAssignableFrom(ps[i])) xs[i]=Collections.singletonList("list");
        else xs[i]="arg"+i;
      }
      m.invoke(o,xs);
    }
    Object v=c.getField("v").get(o);
    System.out.println("value="+(v instanceof Object[] ? Arrays.deepToString((Object[])v) : v));
    System.out.println("classvars="+c.getTypeParameters().length);
    System.out.println("fieldGenericType="+c.getField("v").getGenericType().getTypeName());
  }
}
