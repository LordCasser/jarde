public class NestedCallArgument<T> {
    public T first(T x) { return x; }
    public T second(T x) { return x; }
    public T relay(T x) { return second(first(x)); }
}
