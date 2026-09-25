/** The parent carries AutoCloseable; the child does not declare it directly. */
class NullResourceBase implements AutoCloseable {
    @Override public void close() { }
}
