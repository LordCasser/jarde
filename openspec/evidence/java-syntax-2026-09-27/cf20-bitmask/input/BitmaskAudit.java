public class BitmaskAudit {
    private static final int MASK = 2;
    private static int reads;

    private static int observe(int value) {
        reads++;
        return value;
    }

    public static int select(int value) {
        if ((observe(value) & MASK) != 0) {
            return 11;
        }
        return 22;
    }

    public static int selectZeroMask(int value) {
        if ((observe(value) & MASK) == 0) {
            return 33;
        }
        return 44;
    }

    private static String measured(int value) {
        reads = 0;
        int selected = select(value);
        int count = reads;
        reads = 0;
        int zero = selectZeroMask(value);
        return selected + ":" + count + "," + zero + ":" + reads;
    }

    public static void main(String[] args) {
        int[] values = { 0, 1, 2, 3, Integer.MIN_VALUE, Integer.MIN_VALUE + 2 };
        for (int value : values) {
            System.out.println(value + "=" + measured(value));
        }
    }
}
