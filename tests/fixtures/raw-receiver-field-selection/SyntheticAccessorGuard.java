public class SyntheticAccessorGuard<T> {
    public T value;

    public static void access$set(SyntheticAccessorGuard receiver, Object value) {
        receiver.value = value;
    }

    public void write(Object value) {
        access$set(this, value);
    }
}
