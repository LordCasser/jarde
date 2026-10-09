public class IndependentCallee<T> {
  public <U> U id(U x) { return x; }
  public T relay(T x) { return this.<T>id(x); }
}
