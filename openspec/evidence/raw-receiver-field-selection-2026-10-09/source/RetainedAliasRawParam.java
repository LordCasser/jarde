public class RetainedAliasRawParam<T> {
    public T value;

    public static Object put(RetainedAliasRawParam receiver, Object value) {
        RetainedAliasRawParam alias = receiver;
        alias.value = value;
        return alias;
    }
}
