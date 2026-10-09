package defpackage;

/* JADX INFO: loaded from: PrimitiveArrayRawParam.jar:PrimitiveArrayRawParam.class */
public class PrimitiveArrayRawParam<T> {
    public T value;

    /* JADX WARN: Multi-variable type inference failed */
    public static void put(PrimitiveArrayRawParam receiver, int[] iArr) {
        receiver.value = iArr;
    }
}
