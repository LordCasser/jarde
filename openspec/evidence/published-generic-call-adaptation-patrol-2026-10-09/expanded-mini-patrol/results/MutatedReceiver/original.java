public class MutatedReceiver<T> {
  public static class Box<U> { public U id(U x) { return x; } }
  @SuppressWarnings({"rawtypes","unchecked"}) public T relay(T x) {
    Box<T> receiver = new Box<T>();
    Box raw = new Box();
    receiver = raw;
    return receiver.id(x);
  }
  public static void main(String[] a) { Object m=new Object(); System.out.println("behavior.marker="+(new MutatedReceiver<Object>().relay(m)==m)); }
}
