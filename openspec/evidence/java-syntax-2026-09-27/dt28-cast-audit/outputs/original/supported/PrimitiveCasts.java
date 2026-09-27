package dt28;

public final class PrimitiveCasts {
	private PrimitiveCasts() {
	}

	public static long widenChar(char value) {
		return (long) value << 32;
	}

	public static int truncateLong(long value) {
		return (int) value >> 2;
	}

	public static byte narrowLong(long value) {
		return (byte) value;
	}

	public static short narrowInt(int value) {
		return (short) value;
	}

	public static char narrowChar(int value) {
		return (char) value;
	}

	public static byte byteConditional(boolean condition) {
		return condition ? (byte) 1 : (byte) 0;
	}

	// Control: binary numeric promotion gives this conditional type long without casts.
	public static long promotedConditional(boolean condition, int narrow, long wide) {
		return condition ? narrow : wide;
	}
}
