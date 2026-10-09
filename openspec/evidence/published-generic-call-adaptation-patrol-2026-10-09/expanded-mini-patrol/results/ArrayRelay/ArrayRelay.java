public class ArrayRelay<T> {
  public T[] id(T[] x) { return x; }
  public T[] relay(T[] x) { return id(x); }
  public static void main(String[] a) { Object[] m=new Object[]{new Object()}; ArrayRelay<Object> c=new ArrayRelay<Object>(); System.out.println("behavior.marker="+(c.relay(m)==m)); }
}
