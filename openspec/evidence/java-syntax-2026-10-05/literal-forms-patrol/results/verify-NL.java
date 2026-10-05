public class NL extends java.lang.Object {
    static long hexLong;

    static int binLit;

    static long under;

    static double hexFloat;

    static char hexChar;

    public NL() {
        super();
        return;
    }

    static int charArith(char arg0) {
        return arg0 * 2 + 1;
    }

    static java.lang.String bits(long arg0) {
        return java.lang.Long.toBinaryString(arg0);
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("").append(NL.hexLong).append("/").append(NL.binLit).append("/").append(NL.under).append("/").append(NL.hexFloat).append("/").append(NL.hexChar).append("/").append(charArith('A')).append("/").append((java.lang.String) bits(255L)).toString());
        return;
    }

    static {
        NL.hexLong = 3405691582L;
        NL.binLit = 170;
        NL.under = 1000000000L;
        NL.hexFloat = 0x1.91eb851eb851fp1d;
        NL.hexChar = '\u4e2d';
    }
}
