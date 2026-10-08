public class RawOtherWriter<T> {
    public T v;
    public static void put(RawOtherWriter raw, Object value) {
        raw.v = value;
    }
}
