public final class Runner {
    private static final int[][] CASES = {
        {9, 2},
        {10, 3},
        {11, 1},
        {8, 2},
        {12, 4}
    };

    public static void main(String[] args) {
        for (int i = 0; i < CASES.length; i++) {
            int[] c = CASES[i];
            System.out.println("case" + (i + 1) + "="
                    + ForAddStoreSimple.run(c[0], c[1]));
        }
    }
}
