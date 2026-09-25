public class BoundaryRunner {
    private static String doubleBits(double value) {
        return Long.toHexString(Double.doubleToRawLongBits(value));
    }

    public static void main(String[] args) {
        System.out.println("locals:" + BoundaryProbe.savedOldValue(16777216));
        System.out.println("grouped:" + BoundaryProbe.grouped(16777216));
        System.out.println("field:" + BoundaryProbe.fieldConsumer(16777217));
        System.out.println("array:" + doubleBits(BoundaryProbe.arrayConsumer(1, 9007199254740993L)));
        System.out.println("narrow-array:" + BoundaryProbe.arrayNarrowConsumer(2, 257));
        BoundaryEffects.trace = 0;
        System.out.println("discard:" + BoundaryProbe.discardConvertedProducer() + ":" + BoundaryEffects.trace);
        BoundaryEffects.trace = 0;
        System.out.println("repeat:" + BoundaryProbe.repeatConvertedValue() + ":" + BoundaryEffects.trace);
    }
}
