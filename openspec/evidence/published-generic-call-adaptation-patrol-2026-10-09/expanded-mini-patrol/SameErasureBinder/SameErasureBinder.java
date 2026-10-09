public class SameErasureBinder<T extends Number & Runnable> {
  public <U extends Number & Runnable> U sink(U x) { return x; }
  public T relay(T x) { return sink(x); }
  public static void main(String[] a) { SameErasureBinder<Object> c=null; System.out.println("behavior.null=true"); }
}
