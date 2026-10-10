package defpackage;

import java.util.Arrays;

public final class ArrayLiteralBoundariesRunner {
	private ArrayLiteralBoundariesRunner() {
	}

	public static void main(String[] args) {
		LongArrayLimits limits = new LongArrayLimits();
		long[] longs = limits.test();
		long[] longsAgain = limits.test();
		check(longs != longsAgain, "LongArrayLimits.test must return a fresh array");
		check(Arrays.equals(longs, new long[] { 0L, 1L, Long.MAX_VALUE, Long.MIN_VALUE + 1 }),
				"long literal values must preserve all 64 bits");
		System.out.println("longs=" + longs.length + ":" + longs[0] + ":" + longs[1] + ":" + longs[2] + ":" + longs[3]);

		ConstantIntArray constants = new ConstantIntArray();
		int[] ints = constants.test();
		int[] intsAgain = constants.test();
		check(ints != intsAgain, "ConstantIntArray.test must return a fresh array");
		check(ConstantIntArray.CONST_INT == 65535, "CONST_INT keeps its exact positive value");
		check(Arrays.equals(ints, new int[] { 127, 129, 65535 }), "constant int array values");
		System.out.println("ints=" + ints.length + ":" + ints[0] + ":" + ints[1] + ":" + ints[2]);

		DependentArrayStores dependent = new DependentArrayStores();
		int[] values = dependent.test();
		int[] valuesAgain = dependent.test();
		check(values != valuesAgain, "DependentArrayStores.test must return a fresh array");
		check(Arrays.equals(values, new int[] { 1, 2, 3 }), "dependent stores use previous elements");
		System.out.println("dependent=" + values.length + ":" + values[0] + ":" + values[1] + ":" + values[2]);
	}

	private static void check(boolean condition, String message) {
		if (!condition) {
			throw new AssertionError(message);
		}
	}
}
