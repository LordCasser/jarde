package cf05;

public class ConversionCases {
    private byte myByte = 7;
    private short myShort = 9;

    public static int asInt(boolean flag) {
        return flag ? 1 : 0;
    }

    public static long asLong(boolean flag) {
        return flag ? 1L : 0L;
    }

    public static byte asByte(boolean flag) {
        return flag ? (byte) 1 : (byte) 0;
    }

    public static float asFloat(boolean flag) {
        return flag ? 1.0f : 0.0f;
    }

    public static double asDouble(boolean flag) {
        return flag ? 1.0 : 0.0;
    }

    public int castByte(boolean flag) {
        return write(flag ? (byte) 0 : 1);
    }

    public int byteField(boolean flag) {
        return write(flag ? 0 : myByte);
    }

    public int castShort(boolean flag) {
        return write(flag ? (short) 0 : 1);
    }

    public int shortField(boolean flag) {
        return write(flag ? myShort : 0);
    }

    public int shortConstant(boolean flag) {
        return write(flag ? Short.MIN_VALUE : 0);
    }

    private int write(byte value) {
        return 100 + value;
    }

    private int write(short value) {
        return 200 + value;
    }
}
