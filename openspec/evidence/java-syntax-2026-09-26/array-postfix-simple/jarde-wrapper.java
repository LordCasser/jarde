public final class PostfixProbe {
    public static int readThenIncrement(int[] arg0, int arg1) {
        return arg0[arg1]++;
    }
    public static void main(String[] args) {
        int[] a = { 41 };
        int old = readThenIncrement(a, 0);
        System.out.println("old=" + old + ",new=" + a[0]);
    }
}
