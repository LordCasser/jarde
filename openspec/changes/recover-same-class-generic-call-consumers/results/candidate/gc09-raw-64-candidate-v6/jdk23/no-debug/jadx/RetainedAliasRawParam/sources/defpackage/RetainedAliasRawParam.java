package defpackage;

/* JADX INFO: loaded from: RetainedAliasRawParam.jar:RetainedAliasRawParam.class */
public class RetainedAliasRawParam<T> {
    public T value;

    /* JADX WARN: Multi-variable type inference failed */
    public static Object put(RetainedAliasRawParam retainedAliasRawParam, Object obj) {
        retainedAliasRawParam.value = obj;
        return retainedAliasRawParam;
    }
}
