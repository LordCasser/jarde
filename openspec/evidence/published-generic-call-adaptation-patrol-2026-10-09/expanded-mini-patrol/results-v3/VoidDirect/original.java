public class VoidDirect<T> {
  Object seen;
  public void sink(T x) { seen = x; }
  public void relay(T x) { sink(x); }
}
