package em25;

/* JADX INFO: loaded from: input.jar:em25/BoxingAudit.class */
public final class BoxingAudit {
    private BoxingAudit() {
    }

    public static Object boxInteger() {
        return 1;
    }

    public static Integer integerMinimum() {
        return -128;
    }

    public static Integer integerMaximum() {
        return 127;
    }

    public static Integer integerBelowRange() {
        return -129;
    }

    public static Integer integerAboveRange() {
        return 128;
    }

    public static Number integerAsNumber() {
        return 1;
    }

    public static Integer integerConsumedByCall() {
        return retain(1);
    }

    private static Integer retain(Integer num) {
        return num;
    }

    public static Object boxBoolean() {
        return true;
    }

    public static Object boxFalse() {
        return false;
    }

    public static Object boxByte() {
        return (byte) 2;
    }

    public static Short boxShort() {
        return (short) 3;
    }

    public static Character boxCharacter() {
        return 'c';
    }

    public static Character characterAsciiMaximum() {
        return (char) 127;
    }

    public static Character characterAboveAscii() {
        return (char) 128;
    }

    public static Long boxLong() {
        return 4L;
    }

    public static long unboxOrDefault(Long l) {
        if (l == null) {
            l = 0L;
        }
        return l.longValue();
    }

    public static boolean unbox(Boolean bool) {
        return bool.booleanValue();
    }
}
