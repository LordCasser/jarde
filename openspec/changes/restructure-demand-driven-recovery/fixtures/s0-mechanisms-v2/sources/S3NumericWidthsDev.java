public class S3NumericWidthsDev {
    static String trace = "";
    static int observe(int value) { trace += "o"; return value; }
    public static String run(int n) {
        trace = "";
        int base = n == 9 ? Integer.MAX_VALUE : (n == -2 ? Integer.MIN_VALUE : Integer.MAX_VALUE - n - 1);
        int old = base++;
        int updated = ++base;
        int oldFirst = observe(old);
        int oldSecond = old + n;
        int updatedFirst = observe(updated);
        int updatedSecond = updated ^ n;
        int wide = n * 100003 + 0x1ffff;
        byte narrowByte = (byte) wide;
        short narrowShort = (short) (wide + 32769);
        char narrowChar = (char) (wide + 65535);
        int overflow = Integer.MAX_VALUE + n + 1;
        trace += ":" + old + ":" + updated + ":" + overflow;
        return oldFirst + ":" + oldSecond + ":" + updatedFirst + ":" + updatedSecond + ":" +
                narrowByte + ":" + narrowShort + ":" + (int) narrowChar + ":" + overflow + ":" + trace;
    }
}
