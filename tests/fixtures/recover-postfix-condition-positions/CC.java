public class CC {
    // The three positions again, each answering with the count its increment reached: the do-while
    // scan's answer is its iteration count exactly (the absorbed `iinc` runs once per iteration),
    // and the other two answer the increment's own value at the point the bytecode read it. The
    // compound chain control answers its count too, so the whole class's behavior is compared and
    // not only its text.
    static int scan(int[] xs){ int i = 0, last = -1; do { last = xs[i]; } while (xs[i++] != 0 && i < xs.length); return i; }
    static int find(int[] xs, int t){ int i = 0; while (i < xs.length && xs[i++] != t) { } return i; }
    static int cond(int[] a){ int i = 0; if (a[i++] > 0 && i < a.length) { return i; } return -i; }
    static int chain(int a, int b){ int n = 0; do { n++; a--; b--; } while (a > 0 && b > 0); return n; }
    public static void main(String[] x){
        int[] xs = {5, 7, 0, 9};
        int[] ys = {5, 7, 9};
        int[] zs = {3, 1};
        System.out.println(scan(xs) + "/" + find(ys, 7) + "/" + cond(zs) + "/" + chain(3, 2));
    }
}
