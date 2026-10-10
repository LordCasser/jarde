public class PrimitiveArrayBranches {
	private static Object test4(int type) {
		if (type == 1) {
			return new int[] { 1, 2 };
		} else if (type == 2) {
			return new float[] { 1, 2 };
		} else if (type == 3) {
			return new short[] { 1, 2 };
		} else if (type == 4) {
			return new byte[] { 1, 2 };
		} else {
			return null;
		}
	}

	public static Object choose(int type) {
		return test4(type);
	}
}
