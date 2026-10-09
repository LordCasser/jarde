public class NumberBoundRelay<T extends Number> {
  public T id(T x) { return x; }
  public T relay(T x) { return id(x); }
  public static void main(String[] a) { Integer m=Integer.valueOf(17); NumberBoundRelay<Integer> c=new NumberBoundRelay<Integer>(); System.out.println("behavior.marker="+(c.relay(m)==m)); }
}
