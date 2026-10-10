package defpackage;

/* JADX INFO: loaded from: input.jar:RequiredConversions.class */
public class RequiredConversions {
    static int field;

    public static String castPart(char c) {
        return ((int) c) + "!";
    }

    public static String castPartLast(String str, char c) {
        return str + ((int) c);
    }

    public static String intPart(int i) {
        return i + "!";
    }

    public static int widen(int i) {
        return i;
    }

    public static int argued(char c) {
        return widen(c);
    }

    public static int arguedByte(byte b) {
        return widen(b);
    }

    public static int returned(char c) {
        return c;
    }

    public static int returnedShort(short s) {
        return s;
    }

    public static char kept(char c) {
        return c;
    }

    public static int declared(char c) {
        return c;
    }

    public static int assigned(char c) {
        return c;
    }

    public static int written(char c) {
        field = c;
        return field;
    }
}
