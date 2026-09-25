public final class GridRunner {
    public static void main(String[] args) {
        for (int n : new int[] {0, 1, 3, 6, 9}) {
            System.out.println(n + ":" + Grid.nestedBreak(n) + ":"
                    + Grid.labeledContinue(n) + ":" + Grid.labeledBreak(n));
        }
    }
}
