public class RawReceiver<T> {
  @SuppressWarnings("rawtypes") public java.util.List box = new java.util.ArrayList();
  @SuppressWarnings("unchecked") public T relay(T x) { box.add(x); return (T) box.get(0); }
  public static void main(String[] a) { Object m=new Object(); RawReceiver<Object> c=new RawReceiver<Object>(); System.out.println("behavior.marker="+(c.relay(m)==m)); }
}
