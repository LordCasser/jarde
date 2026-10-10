public class ConstantIntArray {
	public static final int CONST_INT = 0xffff;

	public int[] test() {
		return new int[] { 127, 129, CONST_INT };
	}
}
