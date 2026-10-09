package defpackage;

/* JADX INFO: loaded from: ShadowMethodT.jar:ShadowMethodT.class */
public class ShadowMethodT<T> {
    public T value;

    public static native <T> void observe(ShadowMethodT<T> shadowMethodT, T t);

    public static <T> void put(ShadowMethodT<T> receiver, T value) {
        receiver.value = value;
    }
}
