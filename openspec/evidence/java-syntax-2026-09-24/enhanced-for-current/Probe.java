public class Probe {
    public static int sumAndReturnIndex(int[] a) {
        int sum = 0;
        int i;
        for (i = 0; i < a.length; i++) {
            sum += a[i];
        }
        return i + sum;
    }

    public static int mismatchedArrays(int[] a, int[] b) {
        int sum = 0;
        for (int i = 0; i < a.length; i++) {
            sum += b[i];
        }
        return sum;
    }

    public static void main(String[] args) {
        System.out.println("cached=" + sumAndReturnIndex(new int[] {3, 4}));
        try {
            System.out.println("mismatch=" + mismatchedArrays(new int[] {1, 2}, new int[] {9}));
        } catch (RuntimeException e) {
            System.out.println("mismatch=" + e.getClass().getSimpleName());
        }
    }
}
