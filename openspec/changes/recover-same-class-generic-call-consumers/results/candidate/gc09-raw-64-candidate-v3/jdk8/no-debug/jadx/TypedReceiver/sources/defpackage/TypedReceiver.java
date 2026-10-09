package defpackage;

/* JADX INFO: loaded from: TypedReceiver.jar:TypedReceiver.class */
public class TypedReceiver<T> {
    public T value;

    public void put(TypedReceiver<T> typedReceiver, T t) {
        typedReceiver.value = t;
    }

    public native void observe(TypedReceiver<T> typedReceiver);
}
