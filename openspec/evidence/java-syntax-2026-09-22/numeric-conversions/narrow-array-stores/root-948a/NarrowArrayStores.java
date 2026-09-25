public class NarrowArrayStores {
    public static void storeByte(int[] array, int index, int value) {
        array[index] = value;
    }

    public static void storeChar(int[] array, int index, int value) {
        array[index] = value;
    }

    public static void storeShort(int[] array, int index, int value) {
        array[index] = value;
    }

    public static void storeBoolean(int[] array, int index, int value) {
        array[index] = value;
    }

    public static void storeByteProduced(int[] array, int index, int value, boolean fail) {
        array[index] = NarrowArrayStoreEffects.value(value, fail);
    }

    public static void storeCharProduced(int[] array, int index, int value, boolean fail) {
        array[index] = NarrowArrayStoreEffects.value(value, fail);
    }

    public static void storeShortProduced(int[] array, int index, int value, boolean fail) {
        array[index] = NarrowArrayStoreEffects.value(value, fail);
    }

    public static void storeBooleanProduced(int[] array, int index, int value, boolean fail) {
        array[index] = NarrowArrayStoreEffects.value(value, fail);
    }

    public static void ordinaryByte(byte[] array) {
        byte value = 1;
        array[0] = value;
        array[1] = 0;
        array[2] = 1;
    }

    public static void ordinaryChar(char[] array) {
        char value = 1;
        array[0] = value;
        array[1] = 0;
        array[2] = 1;
    }

    public static void ordinaryShort(short[] array) {
        short value = 1;
        array[0] = value;
        array[1] = 0;
        array[2] = 1;
    }

    public static void ordinaryBoolean(boolean[] array) {
        boolean value = true;
        array[0] = value;
        array[1] = false;
        array[2] = true;
    }
}
