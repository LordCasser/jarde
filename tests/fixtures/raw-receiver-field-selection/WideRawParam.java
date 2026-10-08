public class WideRawParam<T> {
    public T value;

    public static void put(long prefix, WideRawParam receiver, double middle, Object value) {
        receiver.value = value;
    }
}
