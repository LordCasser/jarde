package dt28;

public final class ByteConditionalCall {
	private ByteConditionalCall() {
	}

	public static byte run(long value, boolean condition) {
		return acceptByte(value, condition ? (byte) 1 : (byte) 0);
	}

	private static byte acceptByte(long ignored, byte value) {
		return value;
	}
}
