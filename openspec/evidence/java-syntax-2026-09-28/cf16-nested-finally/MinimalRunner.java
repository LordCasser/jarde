package jadx.tests.integration.trycatch;
import java.lang.reflect.*;
public class MinimalRunner {
 public static void main(String[] args) throws Exception {
  Class<?> c=Class.forName("jadx.tests.integration.trycatch.FinallyMinimalProbe");
  Constructor<?> ctor=c.getDeclaredConstructor(); Field sb=c.getDeclaredField("sb"); sb.setAccessible(true);
  String[] expected={"call-out-finally","call-npe-catch-out-finally","call-iae-finally","call-finally","call-npe-catch-finally","call-iae-finally","call-finally","call-npe-catch-finally","call-iae-finally"};
  int k=0;
  for(int n=1;n<=3;n++) for(int e=0;e<=2;e++) {
   Object t=ctor.newInstance(); sb.set(t,new StringBuilder());
   try { c.getMethod("test"+n,int.class).invoke(t,e); } catch(InvocationTargetException x) { if(!(x.getCause() instanceof IllegalArgumentException)) throw x; }
   String actual=((StringBuilder)sb.get(t)).toString();
   if(!expected[k].equals(actual)) throw new AssertionError(n+","+e+" expected="+expected[k]+" actual="+actual);
   System.out.println(n+","+e+"="+actual); k++;
  }
 }
}
