package em25;

public final class BoxingAudit {
	private BoxingAudit() {
	}

	public static Object boxInteger() {
		return 1;
	}

	public static Integer integerMinimum() {
		return -128;
	}

	public static Integer integerMaximum() {
		return 127;
	}

	public static Integer integerBelowRange() {
		return -129;
	}

	public static Integer integerAboveRange() {
		return 128;
	}

	public static Number integerAsNumber() {
		return Integer.valueOf(1);
	}

	public static Integer integerConsumedByCall() {
		return retain(Integer.valueOf(1));
	}

	private static Integer retain(Integer value) {
		return value;
	}

	public static Object boxBoolean() {
		return true;
	}

	public static Object boxFalse() {
		return false;
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

	public static Character characterAsciiMaximum() {
		return 127;
	}

	public static Character characterAboveAscii() {
		return 128;
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
