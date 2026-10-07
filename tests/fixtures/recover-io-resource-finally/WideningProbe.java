/// The two argument positions the `java.io` widening rows serve, each one row away from the
/// refusal the table's closed set states.
///
/// `wrap` is the first row's own position — a `FileInputStream` value at `InputStreamReader`'s
/// `java.io.InputStream` parameter — and `buffer` is the second row's — an `InputStreamReader`
/// value at `BufferedReader`'s `java.io.Reader` parameter. The table's own unit test proves no
/// other row reaches either pair, so each member's presentation is attributable to its own row.
public final class WideningProbe {
    private WideningProbe() {}

    static java.io.InputStreamReader wrap(String path) throws java.io.IOException {
        return new java.io.InputStreamReader(new java.io.FileInputStream(path), "UTF-8");
    }

    static java.io.BufferedReader buffer(java.io.InputStreamReader reader) {
        return new java.io.BufferedReader(reader);
    }
}
