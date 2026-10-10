public class LongArrayLimits {

	// replaced constant break filled array creation
	private static final int ARRAY_SIZE = 4;

	public long[] test() {
		return new long[] { 0, 1, Long.MAX_VALUE, Long.MIN_VALUE + 1 };
	}
}
