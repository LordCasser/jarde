package dt19;
public class Outer<T> {
  public class Inner {
    public T id(T value) { return value; }
  }
  public Inner make() { return new Inner(); }
}
