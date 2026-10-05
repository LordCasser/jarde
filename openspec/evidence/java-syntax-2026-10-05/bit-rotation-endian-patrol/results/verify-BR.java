public class BR extends java.lang.Object {
    static final char[] HEX = "0123456789abcdef".toCharArray();

    public BR() {
        super();
        return;
    }

    static java.lang.String toHex(int arg0) {
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder(8);
        for (local2 = 7; local2 >= 0; local2 = local2 - 1) {
            local1.append(BR.HEX[arg0 >>> local2 * 4 & 15]);
        }
        return local1.toString();
    }

    static int rotl(int arg0, int arg1) {
        return java.lang.Integer.rotateLeft(arg0, arg1);
    }

    static int revBytes(int arg0) {
        return java.lang.Integer.reverseBytes(arg0);
    }

    static int bitCount(int arg0) {
        return java.lang.Integer.bitCount(arg0);
    }

    static java.lang.String bytes(byte[] arg0) {
        java.lang.StringBuilder local1;
        byte[] local2;
        local1 = new java.lang.StringBuilder();
        local2 = arg0;
        for (byte local5 : local2) {
            local1.append(java.lang.Character.forDigit(local5 >> 4 & 15, 16)).append(java.lang.Character.forDigit(local5 & 15, 16));
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        java.io.PrintStream saved0 = java.lang.System.out;
        java.lang.StringBuilder saved1 = new java.lang.StringBuilder().append("").append((java.lang.String) toHex(-559038737)).append("/").append(rotl(16711935, 8)).append("/").append(revBytes(287454020)).append("/").append(bitCount(-16711921)).append("/");
        saved0.println((java.lang.String) saved1.append((java.lang.String) bytes(new byte[]{-54, -2})).toString());
        return;
    }
}
