// default package

/* JADX INFO: loaded from: ArrayLiteralBoundaryTargets.jar:LongArrayLimits.class */
public class LongArrayLimits {
    private static final int ARRAY_SIZE = 4;

    public long[] test() {
        return new long[]{0, 1, Long.MAX_VALUE, -9223372036854775807L};
    }
}
