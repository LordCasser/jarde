/** Source-only input for a legal byte-array store with a direct boolean operand. */
public final class NarrowArrayStoreBooleanOperand {
    private NarrowArrayStoreBooleanOperand() {}

    public static void storeByteOperand(boolean[] array, int index, boolean value) {
        array[index] = value;
    }
}
