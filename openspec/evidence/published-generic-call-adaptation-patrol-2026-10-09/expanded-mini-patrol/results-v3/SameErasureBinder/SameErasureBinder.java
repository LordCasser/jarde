public class SameErasureBinder<T extends Number & Runnable> {
  public <U extends Number & Runnable> U sink(U x) { return x; }
  public T relay(T x) { return sink(x); }
}
