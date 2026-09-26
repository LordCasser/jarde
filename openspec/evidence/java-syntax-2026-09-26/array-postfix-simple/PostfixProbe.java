public final class PostfixProbe {
    public static int readThenIncrement(int[] a, int i) {
        return a[i]++;
    }
    public static void main(String[] args) {
        int[] a = { 41 };
        int old = readThenIncrement(a, 0);
        System.out.println("old=" + old + ",new=" + a[0]);
    }
}
