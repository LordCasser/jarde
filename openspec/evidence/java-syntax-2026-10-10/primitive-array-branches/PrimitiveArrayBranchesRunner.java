import java.util.Arrays;

public class PrimitiveArrayBranchesRunner {
	private static void require(boolean condition, String message) {
		if (!condition) {
			throw new AssertionError(message);
		}
	}

	private static void checkArray(int type, Class<?> arrayType, String values) {
		Object result = PrimitiveArrayBranches.choose(type);
		require(result != null, "type " + type + " returned null");
		require(result.getClass() == arrayType,
				"type " + type + " returned " + result.getClass().getName());
		switch (type) {
			case 1:
				require(Arrays.equals((int[]) result, new int[] { 1, 2 }), "int values");
				break;
			case 2:
				require(Arrays.equals((float[]) result, new float[] { 1, 2 }), "float values");
				break;
			case 3:
				require(Arrays.equals((short[]) result, new short[] { 1, 2 }), "short values");
				break;
			case 4:
				require(Arrays.equals((byte[]) result, new byte[] { 1, 2 }), "byte values");
				break;
			default:
				throw new AssertionError("unexpected primitive branch: " + type);
		}
		String rendered;
		if (result instanceof int[]) {
			rendered = Arrays.toString((int[]) result);
		} else if (result instanceof float[]) {
			rendered = Arrays.toString((float[]) result);
		} else if (result instanceof short[]) {
			rendered = Arrays.toString((short[]) result);
		} else {
			rendered = Arrays.toString((byte[]) result);
		}
		require(values.equals(rendered), "type " + type + " rendered " + rendered);
		System.out.println("type=" + type + ";class=" + result.getClass().getName() + ";values=" + rendered);
	}

	private static void checkNull(int type) {
		Object result = PrimitiveArrayBranches.choose(type);
		require(result == null, "type " + type + " expected null, got " + result);
		System.out.println("type=" + type + ";null");
	}

	public static void main(String[] args) {
		checkArray(1, int[].class, "[1, 2]");
		checkArray(2, float[].class, "[1.0, 2.0]");
		checkArray(3, short[].class, "[1, 2]");
		checkArray(4, byte[].class, "[1, 2]");
		checkNull(0);
		checkNull(5);
		checkNull(-1);
		checkNull(Integer.MIN_VALUE);
		checkNull(Integer.MAX_VALUE);
	}
}
