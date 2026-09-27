package jadx.tests.integration.trycatch;

import static jadx.tests.api.utils.assertj.JadxAssertions.assertThat;

public class SwitchCatchMinimal {
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
		try {
			switch (testNumber) {
				case 1:
					test1(excType);
					break;
				case 2:
					test2(excType);
					break;
				case 3:
					test3(excType);
					break;
			}
		} catch (IllegalArgumentException e) {
			assertThat(excType).isEqualTo(2);
		}
		return sb.toString();
	}
}
