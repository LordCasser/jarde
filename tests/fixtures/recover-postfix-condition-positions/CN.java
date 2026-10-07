public class CN {
    // The two negatives this slice states (`recover-postfix-condition-positions`): a second
    // variable's position in one condition, and a position in the middle of a short-circuit chain.
    // Each keeps the refusal it had.
    static int twoVariables(int[] xs, int[] ys){ int i = 0, j = 0; while (xs[i++] != 0 && ys[j++] != 0) { } return i + j; }
    static int midChain(int[] xs){ int i = 0; int a = 1; while (a != 0 && xs[i++] != 0 && i < xs.length) { } return i; }
    // The control: the compound do-while chain the region layer already presents, with no postfix
    // position in it. Its text must stay byte-identical.
    static int chain(int a, int b){ int n = 0; do { n++; a--; b--; } while (a > 0 && b > 0); return n; }
    public static void main(String[] x){
        int[] xs = {1, 0};
        int[] ys = {1, 2};
        System.out.println(chain(3, 2) + "/" + twoVariables(xs, ys) + "/" + midChain(xs));
    }
}
