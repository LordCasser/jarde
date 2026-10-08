package defpackage;

/* JADX INFO: loaded from: ArrayRawParam.jar:ArrayRawParam.class */
public class ArrayRawParam<T> {
    public T[] value;

    /* JADX WARN: Multi-variable type inference failed */
    public static void put(ArrayRawParam receiver, Object[] objArr) {
        receiver.value = objArr;
    }
}
