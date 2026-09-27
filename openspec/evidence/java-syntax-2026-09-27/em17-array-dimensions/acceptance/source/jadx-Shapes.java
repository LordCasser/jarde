package em17;

/* JADX INFO: loaded from: input.jar:em17/Shapes.class */
public class Shapes {
    private char[] payload;

    public Shapes(byte[] bArr) {
        char[] chars = toChars(bArr);
        this.payload = new char[chars.length];
        System.arraycopy(chars, 0, this.payload, 0, bArr.length);
    }

    private static char[] toChars(byte[] bArr) {
        return new char[bArr.length];
    }

    public int payloadLength() {
        return this.payload.length;
    }

    public static long[][] longRows(int i) {
        return new long[i][];
    }

    public static String[][] stringRows(int i) {
        return new String[i][];
    }

    public static int[][][] deep(int i) {
        return new int[i][][];
    }

    public static int[][] full(int i, int i2) {
        return new int[i][i2];
    }

    public static int[][] literal(int i, int i2) {
        return new int[][]{new int[]{1}, new int[]{i, i2}, new int[0]};
    }

    public static Object[] wrapped(byte[] bArr) {
        return new Object[]{bArr};
    }
}
