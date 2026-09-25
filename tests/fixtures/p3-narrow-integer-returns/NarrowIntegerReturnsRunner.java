public class NarrowIntegerReturnsRunner {
    public static void main(String[] args) {
        int[] values = { -2147483648, -129, -128, -1, 0, 1, 127, 128, 32767, 32768, 65535, 2147483647 };
        for (int value : values) {
            System.out.println("direct:" + value + ":" + NarrowIntegerReturns.directByte(null, value) + ":" + (int) NarrowIntegerReturns.directChar(null, value) + ":" + NarrowIntegerReturns.directShort(false, value) + ":" + NarrowIntegerReturns.integerControl((byte) value, false));
            System.out.println("local:" + value + ":" + NarrowIntegerReturns.byteLocal((byte) value) + ":" + (int) NarrowIntegerReturns.charLocal((char) value) + ":" + NarrowIntegerReturns.shortLocal((short) value));
        }

        int[] fields = { -129, -128, 127, 128, 32768, 65535 };
        for (int value : fields) {
            NarrowIntegerReturns probe = new NarrowIntegerReturns();
            probe.value = value;
            System.out.println("post:" + value + ":" + probe.postByte(value) + ":" + probe.value);
            probe.value = value;
            System.out.println("pre:" + value + ":" + (int) probe.preChar(value) + ":" + probe.value);
            probe.value = value;
            System.out.println("postShort:" + value + ":" + probe.postShort(value) + ":" + probe.value);
        }

        for (int value : fields) {
            System.out.println("sync:" + value + ":" + NarrowIntegerReturns.syncByte(new Object(), value) + ":" + (int) NarrowIntegerReturns.syncChar("lock", value) + ":" + NarrowIntegerReturns.syncShort(new Object(), value, value));
        }
        try {
            NarrowIntegerReturns.syncByte(null, 128);
            System.out.println("null:returned");
        } catch (Throwable error) {
            System.out.println("null:" + error.getClass().getName());
        }
    }
}
