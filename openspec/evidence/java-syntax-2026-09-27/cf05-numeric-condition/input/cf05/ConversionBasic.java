package cf05;

public class ConversionBasic {
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
}
