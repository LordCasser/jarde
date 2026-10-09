public class NumberBoundRelay<T extends Number> {
  public T id(T x) { return x; }
  public T relay(T x) { return id(x); }
}
