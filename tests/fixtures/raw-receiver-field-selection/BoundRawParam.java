public class BoundRawParam<T extends Number & Comparable<T>> {
    public T value;

    public static void put(BoundRawParam receiver, Number value) {
        receiver.value = value;
    }
}
