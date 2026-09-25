/** Main class compiled with int[] source signatures, then patched to real B/C/S stores. */
public final class NarrowArrayStoreOrder {
    public static void storeByte(int[] array, int index, int value,
            boolean failArray, boolean failIndex, boolean failValue) {
        NarrowArrayStoreOrderEffects.mark('p');
        NarrowArrayStoreOrderEffects.byteArray(array, failArray)[
                NarrowArrayStoreOrderEffects.index(index, failIndex)] =
                NarrowArrayStoreOrderEffects.value(value, failValue);
        NarrowArrayStoreOrderEffects.mark('q');
    }

    public static void storeChar(int[] array, int index, int value,
            boolean failArray, boolean failIndex, boolean failValue) {
        NarrowArrayStoreOrderEffects.mark('p');
        NarrowArrayStoreOrderEffects.charArray(array, failArray)[
                NarrowArrayStoreOrderEffects.index(index, failIndex)] =
                NarrowArrayStoreOrderEffects.value(value, failValue);
        NarrowArrayStoreOrderEffects.mark('q');
    }

    public static void storeShort(int[] array, int index, int value,
            boolean failArray, boolean failIndex, boolean failValue) {
        NarrowArrayStoreOrderEffects.mark('p');
        NarrowArrayStoreOrderEffects.shortArray(array, failArray)[
                NarrowArrayStoreOrderEffects.index(index, failIndex)] =
                NarrowArrayStoreOrderEffects.value(value, failValue);
        NarrowArrayStoreOrderEffects.mark('q');
    }
}
