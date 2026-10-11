package cf11;
public final class Runner {
	private static final int[][] CASES = {
		{-1, 54, 4},
		{12, 54, 4},
		{13, 54, 4},
		{100, 54, 4},
		{12, 0, 4},
		{12, 8, 4},
		{12, 13, 4},
		{12, 54, 0},
		{12, 54, 1},
		{13, 54, 1}
	};

	public static void main(String[] args) {
		NestedLoopSideExitControls controls = new NestedLoopSideExitControls();
		for (int i = 0; i < CASES.length; i++) {
			int threshold = CASES[i][0];
			int bound = CASES[i][1];
			int window = CASES[i][2];
			int result = controls.testFor(threshold, bound, window);
			int counts = controls.runCounts(threshold, bound, window);
			System.out.println(i + ":" + threshold + "," + bound + "," + window + ":" + result + "," + counts);
		}
	}
}
