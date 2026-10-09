public class SameNameOverload<T> {
  public String selected;
  public void pick(T x) { selected="generic"; }
  public void pick(String x) { selected="string"; }
  public void relay(T x) { pick(x); }
  public static void main(String[] a) { Object m=new Object(); SameNameOverload<Object> c=new SameNameOverload<Object>(); c.relay(m); System.out.println("behavior.selected="+c.selected); }
}
