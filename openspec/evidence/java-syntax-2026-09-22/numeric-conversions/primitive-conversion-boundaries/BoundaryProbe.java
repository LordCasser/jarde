public class BoundaryProbe {
    public static long longField;
    public static double[] doubles = new double[4];
    public static byte[] bytes = new byte[4];

    public static int savedOldValue(int value) {
        int old = value;
        value = (int) (float) (value + 1);
        return old * 31 + value;
    }

    public static int grouped(int value) {
        return (int) ((float) (value + 1) * 2.0f);
    }

    public static long fieldConsumer(int value) {
        longField = (long) (float) value;
        return longField;
    }

    public static double arrayConsumer(int index, long value) {
        doubles[index] = (double) (float) value;
        return doubles[index];
    }

    public static int arrayNarrowConsumer(int index, int value) {
        bytes[index] = (byte) value;
        return bytes[index];
    }

    public static int discardConvertedProducer() {
        BoundaryEffects.discard((byte) BoundaryEffects.next());
        return BoundaryEffects.trace;
    }

    public static int repeatConvertedValue() {
        int converted = (int) (float) BoundaryEffects.next();
        return converted + converted;
    }
}
