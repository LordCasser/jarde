public class RawReceiver<T> {
  public static class Box<U> { public U id(U x) { return x; } }
  @SuppressWarnings("rawtypes") public Box box = new Box();
  @SuppressWarnings("unchecked") public T relay(T x) { return (T) box.id(x); }
  public static void main(String[] a) { Object m=new Object(); RawReceiver<Object> c=new RawReceiver<Object>(); System.out.println("behavior.marker="+(c.relay(m)==m)); }
}
