public class ReuseAfterForEachRunner {
    public static void main(String[] args) {
        System.out.println(ReuseAfterForEach.sumThenReuse(new int[] {1, 2, 3}));
        System.out.println(ReuseAfterForEach.sumThenReuse(new int[0]));
        try {
            ReuseAfterForEach.sumThenReuse(null);
            System.out.println("missing-exception");
        } catch (NullPointerException expected) {
            System.out.println("NullPointerException");
        }
    }
}
