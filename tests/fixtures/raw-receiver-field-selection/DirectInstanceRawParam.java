public class DirectInstanceRawParam<T> {
    public T value;

    public void put(DirectInstanceRawParam receiver, Object value) {
        receiver.value = value;
    }
}
