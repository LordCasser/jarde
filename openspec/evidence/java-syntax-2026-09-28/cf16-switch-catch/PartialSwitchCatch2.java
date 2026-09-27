package jadx.tests.integration.trycatch;

import static jadx.tests.api.utils.assertj.JadxAssertions.assertThat;

public class PartialSwitchCatch2 {
	private StringBuilder sb;

	public void test1(int excType) {
		try {
			try {
				call(excType);
			} catch (NullPointerException e) {
				sb.append("-catch");
			}
		sb.append("-out");
		} finally {
			sb.append("-finally");
		}
	}

	public void test2(int excType) {
		try {
			try {
				call(excType);
			} catch (NullPointerException e) {
				sb.append("-catch");
			}
		} finally {
			sb.append("-finally");
		}
	}

	public void test3(int excType) {
		try {
			call(excType);
		} catch (NullPointerException e) {
			sb.append("-catch");
		} finally {
			sb.append("-finally");
		}
	}

	public void call(int excType) {
		sb.append("call");
		switch (excType) {
			case 1:
				sb.append("-npe");
				throw new NullPointerException();
			case 2:
				sb.append("-iae");
				throw new IllegalArgumentException();
		}
	}

	public String runTest(int testNumber, int excType) {
		sb = new StringBuilder();
		switch (testNumber) {
			case 1:
				test1(excType);
				break;
			case 2:
				try {
					test2(excType);
				} catch (IllegalArgumentException e) {
					assertThat(excType).isEqualTo(2);
				}
				break;
			case 3:
				test3(excType);
				break;
		}
		return sb.toString();
	}
}
