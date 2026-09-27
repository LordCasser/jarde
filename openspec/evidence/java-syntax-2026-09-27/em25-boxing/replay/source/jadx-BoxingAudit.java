package em25;

/* JADX INFO: loaded from: input.jar:em25/BoxingAudit.class */
public final class BoxingAudit {
    private BoxingAudit() {
    }

    public static Object boxInteger() {
        return 1;
    }

    public static Object boxBoolean() {
        return true;
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
