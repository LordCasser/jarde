interface Sink<T> { void put(T v); }
class IfaceNarrow implements Sink {
    public void put(String v) { }
}
