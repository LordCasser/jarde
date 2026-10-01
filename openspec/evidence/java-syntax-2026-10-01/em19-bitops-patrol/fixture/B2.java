public class B2 {
    static long seed = 0x123456789AL;
    public static long longOps(long v) {
        long r = v;
        r ^= 0xFFFFL;
        r |= (v & 0xFFL) << 8;
        r &= ~(0xFL);
        r >>>= 2;
        return r;
    }
    public static String compound(int v) {
        int m = v;
        m |= 0x10;
        m &= 0x3C;
        m ^= 0x08;
        m <<= 1;
        m >>= 2;
        boolean hasA = (v & 0x01) != 0 && (v & 0x02) != 0;
        boolean hasB = (v & 0x04) != 0 || (v & 0x08) != 0;
        return m + ":" + hasA + ":" + hasB;
    }
    public static void main(String[] a) {
        System.out.println(longOps(seed));
        System.out.println(compound(0x1B));
    }
}
