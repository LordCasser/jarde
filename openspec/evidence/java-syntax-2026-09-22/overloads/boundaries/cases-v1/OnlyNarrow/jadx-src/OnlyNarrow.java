
/* JADX INFO: loaded from: OnlyNarrow.class */
public class OnlyNarrow {
    public static int onlyByte(byte b) {
        return 1;
    }

    public static int onlyShort(short s) {
        return 3;
    }

    public static int runByte() {
        return onlyByte((byte) 3);
    }

    public static int runShort() {
        return onlyShort((short) 3);
    }
}
