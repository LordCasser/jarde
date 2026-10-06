/// The definition `Arity` is compiled against: one type parameter, so the class's own
/// `Signature` states exactly one argument.
public interface ArityApi<T> {
    void put(T value);
}
