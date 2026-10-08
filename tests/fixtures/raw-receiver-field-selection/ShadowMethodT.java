public class ShadowMethodT<T> {
    public T value;
    public static <T> void put(ShadowMethodT<T> receiver, T value) { receiver.value = value; }
    public static native <T> void observe(ShadowMethodT<T> receiver, T value);
}
