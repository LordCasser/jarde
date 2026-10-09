public class MultiParam<T> {
  public T first(T x, T y) { return x; }
  public T relay(T x, T y) { return first(x, y); }
  public static void main(String[] a) { Object m=new Object(); System.out.println("behavior.marker="+(new MultiParam<Object>().relay(m,m)==m)); }
}
