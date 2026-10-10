package discardprobe;

/** Separate negative fixture: javac emits invoke2; pop2 for the discarded long result. */
public final class ProbePop2 {
    private ProbePop2() {}

    public static long wide() {
        return 7L;
    }

    public static void discardWide() {
        wide();
    }
}
