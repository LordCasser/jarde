public class UnsafeOtherWriter<T> {
    public T value;

    public static void putRaw(UnsafeOtherWriter receiver, Object value) {
        receiver.value = value;
    }

    @SuppressWarnings("unchecked")
    public void putThroughUnprovedCast(Object value) {
        this.value = (T) value;
    }
}
