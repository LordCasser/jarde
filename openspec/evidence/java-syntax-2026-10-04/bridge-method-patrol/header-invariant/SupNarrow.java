class BoxP<T> { void set(T v) { } }
class SupNarrow extends BoxP {
    void set(String v) { }
}
