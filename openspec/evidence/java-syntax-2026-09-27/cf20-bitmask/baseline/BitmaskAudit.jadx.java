
/* JADX INFO: loaded from: input.jar:BitmaskAudit.class */
public class BitmaskAudit {
    private static final int MASK = 2;
    private static int reads;

    private static int observe(int i) {
        reads++;
        return i;
    }

    public static int select(int i) {
        return (observe(i) & MASK) != 0 ? 11 : 22;
    }

    public static int selectZeroMask(int i) {
        return (observe(i) & MASK) == 0 ? 33 : 44;
    }

    private static String measured(int i) {
        reads = 0;
        int iSelect = select(i);
        int i2 = reads;
        reads = 0;
        return iSelect + ":" + i2 + "," + selectZeroMask(i) + ":" + reads;
    }

    public static void main(String[] strArr) {
        for (int i : new int[]{0, 1, MASK, 3, Integer.MIN_VALUE, -2147483646}) {
            System.out.println(i + "=" + measured(i));
        }
    }
}
