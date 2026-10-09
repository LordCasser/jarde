package defpackage;

/* JADX INFO: loaded from: TypedReceiver.jar:TypedReceiver.class */
public class TypedReceiver<T> {
    public T value;

    public native void observe(TypedReceiver<T> typedReceiver);

    public void put(TypedReceiver<T> receiver, T value) {
        receiver.value = value;
    }
}
