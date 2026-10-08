public class NestedAliasRawParam<T> {
    public T value;

    public static Object put(NestedAliasRawParam receiver, Object value) {
        {
            NestedAliasRawParam alias = receiver;
            alias.value = value;
        }
        {
            NestedAliasRawParam alias = receiver;
            alias.value = value;
        }
        return receiver;
    }
}
