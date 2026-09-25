
/* JADX INFO: loaded from: NarrowOverloads.class */
public class NarrowOverloads {
    public static int onlyByte(byte b) {
        return 1;
    }

    public static int onlyByte(int i) {
        return 2;
    }

    public static int onlyShort(short s) {
        return 3;
    }

    public static int onlyShort(int i) {
        return 4;
    }

    public static int runByte() {
        return onlyByte((byte) 3);
    }

    public static int runShort() {
        return onlyShort((short) 3);
    }
}
