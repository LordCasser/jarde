public final class ForBoundariesRunner {
    public static void main(String[] args) {
        for (int n : new int[] {-1, 0, 1, 2, 5}) {
            System.out.println(n + ":" + ForBoundaries.counted(n)
                    + ":" + ForBoundaries.observedAfter(n)
                    + ":" + ForBoundaries.twoUpdates(n)
                    + ":" + ForBoundaries.varyingBound(n)
                    + ":" + ForBoundaries.noUpdate(n));
        }
    }
}
