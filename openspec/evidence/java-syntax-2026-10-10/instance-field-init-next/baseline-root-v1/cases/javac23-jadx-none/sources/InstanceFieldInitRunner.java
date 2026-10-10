import java.util.Arrays;

public class InstanceFieldInitRunner {
	private static void require(boolean condition, String message) {
		if (!condition) {
			throw new AssertionError(message);
		}
	}

	private static void requireArray(byte[] actual, byte... expected) {
		require(Arrays.equals(actual, expected), "array: " + Arrays.toString(actual));
	}

	public static void main(String[] args) {
		CommonDirectSuperByteArray.trace = "";
		CommonDirectSuperByteArray common0 = new CommonDirectSuperByteArray();
		requireArray(common0.bytes, (byte) 10, (byte) 20);
		require("eval:10;eval:20;body:noarg;".equals(CommonDirectSuperByteArray.trace),
				"common noarg order: " + CommonDirectSuperByteArray.trace);

		CommonDirectSuperByteArray.trace = "";
		CommonDirectSuperByteArray common1 = new CommonDirectSuperByteArray(4);
		requireArray(common1.bytes, (byte) 10, (byte) 20);
		require("eval:10;eval:20;body:int:4;".equals(CommonDirectSuperByteArray.trace),
				"common int order: " + CommonDirectSuperByteArray.trace);
		require(common0.bytes != common1.bytes, "each instance gets its own array");
		System.out.println("common=" + Arrays.toString(common0.bytes) + ";"
				+ Arrays.toString(common1.bytes) + ";distinct=" + (common0.bytes != common1.bytes));

		ThisDelegatingByteArray.trace = "";
		ThisDelegatingByteArray delegated0 = new ThisDelegatingByteArray();
		requireArray(delegated0.bytes, (byte) 21, (byte) 22);
		require("eval:21;eval:22;body:target:7;body:delegate;".equals(ThisDelegatingByteArray.trace),
				"delegating order/count: " + ThisDelegatingByteArray.trace);
		String firstDelegationTrace = ThisDelegatingByteArray.trace;
		ThisDelegatingByteArray.trace = "";
		ThisDelegatingByteArray delegated1 = new ThisDelegatingByteArray();
		requireArray(delegated1.bytes, (byte) 21, (byte) 22);
		require(firstDelegationTrace.equals(ThisDelegatingByteArray.trace),
				"each delegation initializes once: " + ThisDelegatingByteArray.trace);
		require(delegated0.bytes != delegated1.bytes, "delegated instances get distinct arrays");
		System.out.println("delegated=" + Arrays.toString(delegated0.bytes) + ";trace="
				+ firstDelegationTrace + ";distinct=" + (delegated0.bytes != delegated1.bytes));

		DifferentRhsByteArray.trace = "";
		DifferentRhsByteArray different0 = new DifferentRhsByteArray();
		requireArray(different0.bytes, (byte) 31);
		require("eval:31;body:noarg;".equals(DifferentRhsByteArray.trace),
				"different noarg path: " + DifferentRhsByteArray.trace);
		DifferentRhsByteArray.trace = "";
		DifferentRhsByteArray different1 = new DifferentRhsByteArray(5);
		requireArray(different1.bytes, (byte) 32);
		require("eval:32;body:int:5;".equals(DifferentRhsByteArray.trace),
				"different int path: " + DifferentRhsByteArray.trace);
		System.out.println("different=" + Arrays.toString(different0.bytes) + ";"
				+ Arrays.toString(different1.bytes));
	}
}
