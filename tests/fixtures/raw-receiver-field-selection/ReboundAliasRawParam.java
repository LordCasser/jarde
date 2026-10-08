public class ReboundAliasRawParam<T> {
    public T value;

    public static Object put(
            ReboundAliasRawParam receiver,
            ReboundAliasRawParam replacement,
            Object value,
            boolean flag) {
        ReboundAliasRawParam alias = receiver;
        if (flag) alias = replacement;
        alias.value = value;
        return alias;
    }
}
