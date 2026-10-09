public class RawReceiver<T> {
  @SuppressWarnings("rawtypes") public java.util.List box = new java.util.ArrayList();
  @SuppressWarnings("unchecked") public T relay(T x) { box.add(x); return (T) box.get(0); }
}
