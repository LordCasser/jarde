public class B1 {
    public static String bitOps(int v) {
        StringBuilder b = new StringBuilder();
        if ((v & 0xF0) != 0) b.append('h');
        if ((v | 0x01) == v + 1) b.append('o');
        if ((v ^ 0x0F) == 0xF0) b.append('x');
        int shifted = (v << 2) | (v >> 3) | (v >>> 1);
        b.append(':').append(shifted & 0xFF);
        b.append(':').append(~v & 0xFF);
        int nonlocal = shifted & getMask();
        b.append(':').append(nonlocal);
        return b.toString();
    }
    static int getMask() { return 0x33; }
    public static void main(String[] a) {
        System.out.println(bitOps(0x5A));
        System.out.println(bitOps(0x03));
    }
}
