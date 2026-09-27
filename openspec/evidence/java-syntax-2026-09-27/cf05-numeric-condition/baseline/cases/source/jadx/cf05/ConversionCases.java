package cf05;

/* JADX INFO: loaded from: ConversionCases.jar:cf05/ConversionCases.class */
public class ConversionCases {
    private byte myByte = 7;
    private short myShort = 9;

    public static int asInt(boolean z) {
        return z ? 1 : 0;
    }

    public static long asLong(boolean z) {
        return z ? 1L : 0L;
    }

    public static byte asByte(boolean z) {
        return z ? (byte) 1 : (byte) 0;
    }

    public static float asFloat(boolean z) {
        return z ? 1.0f : 0.0f;
    }

    public static double asDouble(boolean z) {
        return z ? 1.0d : 0.0d;
    }

    public int castByte(boolean z) {
        return write(z ? (byte) 0 : (byte) 1);
    }

    public int byteField(boolean z) {
        return write(z ? (byte) 0 : this.myByte);
    }

    public int castShort(boolean z) {
        return write(z ? (short) 0 : (short) 1);
    }

    public int shortField(boolean z) {
        return write(z ? this.myShort : (short) 0);
    }

    public int shortConstant(boolean z) {
        return write(z ? Short.MIN_VALUE : (short) 0);
    }

    private int write(byte b) {
        return 100 + b;
    }

    private int write(short s) {
        return 200 + s;
    }
}
