public class MethodShadow<T> {
  public <T extends Number> T relay(T x) { return this.<T>id(x); }
  public <U extends Number> U id(U x) { return x; }
}
