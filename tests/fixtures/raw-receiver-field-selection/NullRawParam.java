public class NullRawParam<T> {
    public T value;

    public static void put(NullRawParam receiver) {
        receiver.value = null;
    }
}
