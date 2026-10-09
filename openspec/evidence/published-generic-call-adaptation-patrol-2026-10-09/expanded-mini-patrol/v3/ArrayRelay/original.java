public class ArrayRelay<T> {
  public T[] id(T[] x) { return x; }
  public T[] relay(T[] x) { return id(x); }
}
