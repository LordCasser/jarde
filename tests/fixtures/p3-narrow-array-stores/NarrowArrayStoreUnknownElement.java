/** Source-only legal-JVM shape whose bastore array operand has verifier type null. */
public final class NarrowArrayStoreUnknownElement {
    private NarrowArrayStoreUnknownElement() {}

    public static void unknown() {
        byte[] array = null;
        array[0] = 1;
    }
}
