public class RawOwnerChild<T> extends RawOwnerBase<T> {
    public static void put(RawOwnerChild receiver, Object value) {
        receiver.value = value;
    }
}
