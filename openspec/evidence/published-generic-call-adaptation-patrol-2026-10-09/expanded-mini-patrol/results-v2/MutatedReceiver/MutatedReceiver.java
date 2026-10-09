public class MutatedReceiver<T> {
  @SuppressWarnings({"rawtypes","unchecked"}) public T relay(T x) {
    java.util.List<T> receiver = new java.util.ArrayList<T>();
    java.util.List raw = new java.util.ArrayList();
    raw.add(x);
    receiver = raw;
    return receiver.get(0);
  }
  public static void main(String[] a) { Object m=new Object(); System.out.println("behavior.marker="+(new MutatedReceiver<Object>().relay(m)==m)); }
}
