public class DependentArrayStores {
	public int[] test() {
		int[] arr = new int[3];
		arr[0] = 1;
		arr[1] = arr[0] + 1;
		arr[2] = arr[1] + 1;
		return arr;
	}
}
