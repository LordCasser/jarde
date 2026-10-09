package defpackage;

/* JADX INFO: loaded from: Array2DRawParam.jar:Array2DRawParam.class */
public class Array2DRawParam<T> {
    public T[] value;

    /* JADX WARN: Multi-variable type inference failed */
    public static void put(Array2DRawParam receiver, Object[][] objArr) {
        receiver.value = objArr;
    }
}
