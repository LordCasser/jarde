/** Source-only assertion that the verifier-valid unknown-array shape throws at bastore. */
public final class NarrowArrayStoreUnknownRunner {
    private NarrowArrayStoreUnknownRunner() {}

    public static void main(String[] args) {
        try {
            NarrowArrayStoreUnknownElement.unknown();
            throw new AssertionError("bastore completed for a null array");
        } catch (NullPointerException expected) {
            System.out.println("unknown:java.lang.NullPointerException");
        }
    }
}
