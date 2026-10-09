public class VoidDirect<T> {
  public Object seen;
  public void sink(T x) { seen = x; }
  public void relay(T x) { sink(x); }
}
