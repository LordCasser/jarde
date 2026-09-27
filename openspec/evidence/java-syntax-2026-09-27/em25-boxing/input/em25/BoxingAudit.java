package em25;

public final class BoxingAudit {
	private BoxingAudit() {
	}

	public static Object boxInteger() {
		return 1;
	}

	public static Object boxBoolean() {
		return true;
	}

	public static Object boxByte() {
		return (byte) 2;
	}

	public static Short boxShort() {
		return 3;
	}

	public static Character boxCharacter() {
		return 'c';
	}

	public static Long boxLong() {
		return 4L;
	}

	public static long unboxOrDefault(Long value) {
		if (value == null) {
			value = 0L;
		}
		return value;
	}

	public static boolean unbox(Boolean value) {
		return value;
	}
}
