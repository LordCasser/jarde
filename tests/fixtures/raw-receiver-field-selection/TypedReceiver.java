public class TypedReceiver<T> {
    public T value;
    public void put(TypedReceiver<T> receiver, T value) { receiver.value = value; }
    public native void observe(TypedReceiver<T> receiver);
}
