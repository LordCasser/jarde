public class AV {
    static int[][] p = new int[2][];
    static int[] f;
    static int calls;

    static int sum(int[] a) { return a.length; }
    static int mark(int v) { calls = calls + 1; return v; }

    // --- the change's own anchors: the two positions that used to refuse -----------
    static int elemStore() { p[0] = new int[]{7}; return p[0][0]; }
    static int immIdx() { return new int[]{9}[0]; }

    // --- the four positions that must stay byte-identical -------------------------
    static int localPos() { int[] x = new int[]{1, 2}; return x[0] + x[1]; }
    static int fieldPos() { AV.f = new int[]{3}; return AV.f[0]; }
    static int argPos() { return sum(new int[]{5}); }
    static int[][] outerPos() { return new int[][]{ new int[]{6}, new int[]{7, 8} }; }

    // --- bare immediate consumption, and the sawtooth family ----------------------
    static int bareIdx() { return new int[2].length; }
    static int[] bareRet() { return new int[3]; }
    static int[][] jagged() { return new int[][]{ new int[]{1}, new int[]{2, 3}, new int[]{4, 5, 6} }; }

    // --- the same consumption positions, one step further -------------------------
    static int immLen() { return new int[]{4, 5, 6}.length; }
    static int immIdxVar(int i) { return new int[]{9}[i]; }
    static int immIdxExpr(int i) { return new int[]{9, 8}[i + 1]; }
    static int immIdxSum() { return new int[]{9}[0] + 1; }
    static int twoIdx() { return new int[]{9}[0] + new int[]{8}[0]; }
    static int twoStores() { p[0] = new int[]{7}; p[1] = new int[]{8}; return p[1][0]; }
    static int nestedIdx() { return new int[][]{ new int[]{9} }[0][0]; }
    static int foreach() { int s = 0; for (int v : new int[]{1, 2}) s += v; return s; }
    static boolean condIdx() { return new int[]{9}[0] > 0; }
    static int immIdxInCall() { return sum(new int[]{9}) + new int[]{1}[0]; }
    static int order() { return new int[]{mark(1)}[mark(0)]; }

    public static void main(String[] a) {
        // One call per statement: the concatenation rule's own dependency closure is bounded, and a
        // single twenty-argument chain is past that bound for reasons this fixture is not about.
        System.out.print(elemStore()); System.out.print("/");
        System.out.print(immIdx()); System.out.print("/");
        System.out.print(localPos()); System.out.print("/");
        System.out.print(fieldPos()); System.out.print("/");
        System.out.print(argPos()); System.out.print("/");
        System.out.print(outerPos()[1][1]); System.out.print("/");
        System.out.print(bareIdx()); System.out.print("/");
        System.out.print(bareRet().length); System.out.print("/");
        System.out.print(jagged()[2][2]); System.out.print("/");
        System.out.print(immLen()); System.out.print("/");
        System.out.print(immIdxVar(0)); System.out.print("/");
        System.out.print(immIdxExpr(0)); System.out.print("/");
        System.out.print(immIdxSum()); System.out.print("/");
        System.out.print(twoIdx()); System.out.print("/");
        System.out.print(twoStores()); System.out.print("/");
        System.out.print(nestedIdx()); System.out.print("/");
        System.out.print(foreach()); System.out.print("/");
        System.out.print(condIdx()); System.out.print("/");
        System.out.print(immIdxInCall()); System.out.print("/");
        System.out.print(order()); System.out.print("/");
        System.out.println(calls);
    }
}
