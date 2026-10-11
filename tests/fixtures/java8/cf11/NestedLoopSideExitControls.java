package cf11;
public class NestedLoopSideExitControls {
	public int testFor(int returnThreshold, int bound, int window) {
		int tmp = 1;
		for (int i = 10; i > -1; i--) {
			if (i > tmp) {
				for (int j = 0; j < bound; j += 4) {
					if (i < j) {
						for (int k = j; k < j + window; k++) {
							if (returnThreshold > k) {
								return 0;
							}
						}
						break;
					}
				}
			}
			tmp++;
		}
		return tmp;
	}

	public int runCounts(int returnThreshold, int bound, int window) {
		int tmp = 1;
		int ticks = 0;
		for (int i = 10; i > -1; i--) {
			if (i > tmp) {
				for (int j = 0; j < bound; j += 4) {
					if (i < j) {
						for (int k = j; k < j + window; k++) {
							ticks++;
							if (returnThreshold > k) {
								return ticks;
							}
						}
						break;
					}
				}
			}
			tmp++;
		}
		return tmp * 1000 + ticks;
	}
}
