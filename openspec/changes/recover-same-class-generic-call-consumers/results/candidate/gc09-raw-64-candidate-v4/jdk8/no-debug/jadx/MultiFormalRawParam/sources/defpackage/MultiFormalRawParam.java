package defpackage;

/* JADX INFO: loaded from: MultiFormalRawParam.jar:MultiFormalRawParam.class */
public class MultiFormalRawParam<T> {
    public T value;

    /* JADX WARN: Multi-variable type inference failed */
    public static <K> void put(MultiFormalRawParam multiFormalRawParam, K k, Object obj) {
        k.hashCode();
        multiFormalRawParam.value = obj;
    }
}
