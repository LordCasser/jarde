public class RawReceiver<T> {
    public T value;

    public static void rawParameter(RawReceiver raw, Object value) {
        raw.value = value;
    }

    public static void staticRawLocal(RawReceiver raw, Object value) {
        RawReceiver alias = raw;
        alias.value = value;
    }

    public void instanceRawLocal(Object value) {
        RawReceiver alias = this;
        alias.value = value;
    }

    public void thisReceiver(T value) {
        this.value = value;
    }

    public void parameterizedReceiver(RawReceiver<T> receiver, T value) {
        receiver.value = value;
    }

    public static <T> void shadowMethodT(RawReceiver<T> receiver, T value) {
        receiver.value = value;
    }

    public T read() {
        return value;
    }
}
