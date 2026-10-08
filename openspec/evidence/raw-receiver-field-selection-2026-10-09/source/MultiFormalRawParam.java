public class MultiFormalRawParam<T> {
    public T value;

    public static <K> void put(MultiFormalRawParam receiver, K key, Object value) {
        key.hashCode();
        receiver.value = value;
    }
}
