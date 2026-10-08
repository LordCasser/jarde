public class ArrayRawParam<T> {
    public T[] value;

    public static void put(ArrayRawParam receiver, Object[] value) {
        receiver.value = value;
    }
}
