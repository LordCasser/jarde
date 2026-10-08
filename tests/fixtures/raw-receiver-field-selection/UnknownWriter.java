public class UnknownWriter<T> {
    public T value;

    private static Object producedByUnknownCall() {
        return new Object();
    }

    public static void put(UnknownWriter receiver) {
        receiver.value = producedByUnknownCall();
    }
}
