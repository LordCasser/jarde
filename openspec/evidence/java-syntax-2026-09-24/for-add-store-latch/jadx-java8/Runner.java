public final class Runner {
    private static final int[][] CASES = {
        {9, 2, 2},
        {10, 3, 3},
        {11, 1, 4},
        {8, 2, -1},
        {12, 4, 8}
    };

    public static void main(String[] args) {
        for (int i = 0; i < CASES.length; i++) {
            int[] c = CASES[i];
            System.out.println("case" + (i + 1) + "="
                    + ForAddStoreLatch.run(c[0], c[1], c[2]));
        }
    }
}
