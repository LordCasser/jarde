package defpackage;

/* JADX INFO: loaded from: RawParam.jar:RawParam.class */
public class RawParam<T> {
    public T value;

    /* JADX WARN: Multi-variable type inference failed */
    public static void put(RawParam receiver, Object obj) {
        receiver.value = obj;
    }
}
