public class Array2DRawParam<T> {
    public T[] value;

    public static void put(Array2DRawParam receiver, Object[][] value) {
        receiver.value = value;
    }
}
