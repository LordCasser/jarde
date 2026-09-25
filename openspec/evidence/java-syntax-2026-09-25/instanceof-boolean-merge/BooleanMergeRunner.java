public final class BooleanMergeRunner {
    public static void main(String[] args) {
        for (Object x : new Object[] { null, "s", Integer.valueOf(4), new Object() }) {
            BooleanMergeControls.calls = 0;
            boolean positive = BooleanMergeControls.positive(x);
            System.out.println("positive:" + positive + ":" + BooleanMergeControls.calls);
            BooleanMergeControls.calls = 0;
            boolean negative = BooleanMergeControls.negative(x);
            System.out.println("negative:" + negative + ":" + BooleanMergeControls.calls);
        }
    }
}
