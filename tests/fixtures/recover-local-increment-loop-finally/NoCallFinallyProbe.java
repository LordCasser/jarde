public class NoCallFinallyProbe {
    public static void main(String[] args) {
        int[][] cases = { {}, {1}, {1,2}, {3}, {1,3}, {1,2,3}, {3,1}, {3,3}, {1,2,1}, null };
        String[] labels = { "empty", "one", "two", "hit", "late", "later", "first", "twice", "none", "null" };
        for (int i = 0; i < cases.length; i++) {
            try {
                System.out.println(labels[i] + ":" + NoCallFinally.noCall(cases[i]));
            } catch (NullPointerException expected) {
                System.out.println(labels[i] + ":null");
            }
        }
    }
}
