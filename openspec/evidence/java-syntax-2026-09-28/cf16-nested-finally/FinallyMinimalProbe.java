package jadx.tests.integration.trycatch;
public class FinallyMinimalProbe {
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

	}
