public class C7 {
    interface Consts { int LIMIT = 10; String NAME = "c7"; long MASK = 0xFFL; }
    public static byte compoundByte(byte b) {
        b += 1; b -= 2; b *= 3; b /= 2; b %= 5; b &= 3; b |= 8; b ^= 1; b <<= 1; b >>= 1; b >>>= 1;
        return b;
    }
    public static short compoundShort(short s) { s += 100; s *= 2; return s; }
    public static char charArith(char c) {
        char r = (char) (c + 1);
        r += 2; r++;
        return r;
    }
    public static int useConsts() { return Consts.LIMIT + Consts.NAME.length() + (int) (Consts.MASK & 0xF); }
    public static void main(String[] a) {
        System.out.println(compoundByte((byte) 9));
        System.out.println(compoundShort((short) 7));
        System.out.println(charArith('a'));
        System.out.println(useConsts());
    }
}
