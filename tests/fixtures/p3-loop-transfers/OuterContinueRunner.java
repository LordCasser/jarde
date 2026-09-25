final class OuterContinueRunner {
    public static void main(String[] args) {
        for (int n : new int[] {0, 1, 2, 3, 5}) {
            System.out.println(n + ":" + OuterContinue.run(n));
        }
    }
}
