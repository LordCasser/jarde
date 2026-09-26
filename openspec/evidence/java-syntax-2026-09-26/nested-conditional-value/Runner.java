public final class Runner {
    private Runner() {}

    public static void main(String[] args) {
        int[] boundaries = {9, 10, 11, 99, 100, 101};
        for (int a : boundaries) {
            System.out.println("nested(" + a + ")=" + NestedConditional.nested(a));
        }
        int[] effectCases = {0, 5, 10, 11, 12, 50, 100, 101, 150};
        for (int a : effectCases) {
            StringBuilder events = new StringBuilder();
            try {
                int value = NestedConditional.nestedEffects(a, events);
                System.out.println("effects(" + a + ")=" + value + ";events=" + events);
            } catch (RuntimeException error) {
                System.out.println("effects(" + a + ")=" + error.getClass().getSimpleName()
                        + ";message=" + error.getMessage() + ";events=" + events);
            }
        }
    }
}
