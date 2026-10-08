public class PrimitiveArrayRawParam<T> {
    public T value;

    public static void put(PrimitiveArrayRawParam receiver, int[] value) {
        receiver.value = value;
    }
}
