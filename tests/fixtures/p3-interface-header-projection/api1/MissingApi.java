/// The definition `Unresolved` is compiled against. It is deliberately **not** part of the
/// frozen environment: the environment ships only `Unresolved.class`, so the interface the
/// class's own `Signature` names has no definition to prove the type arguments against.
public interface MissingApi<T> {
    void put(T value);
}
