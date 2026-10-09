public class IncompatibleBinder<T extends Number> {
  public <U extends Number & Runnable> U sink(U x) { return x; }
  public T relay(T x) { return sink(x); }
}
