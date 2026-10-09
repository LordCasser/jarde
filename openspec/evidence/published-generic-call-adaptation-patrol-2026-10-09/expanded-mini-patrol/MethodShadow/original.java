public class MethodShadow<T> {
  public <T extends Number> T relay(T x) { return this.<T>id(x); }
  public <U extends Number> U id(U x) { return x; }
  public static void main(String[] a) { Integer m=Integer.valueOf(19); System.out.println("behavior.marker="+(new MethodShadow<Object>().relay(m)==m)); }
}
