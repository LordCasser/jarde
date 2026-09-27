package jadx.tests.api.utils.assertj;

public final class JadxAssertions {
	private JadxAssertions() {
	}

	public static Assertion assertThat(Object actual) {
		return new Assertion(actual);
	}

	public static final class Assertion {
		private final Object actual;

		private Assertion(Object actual) {
			this.actual = actual;
		}

		public void isEqualTo(Object expected) {
			if (actual == null ? expected != null : !actual.equals(expected)) {
				throw new AssertionError("expected=" + expected + " actual=" + actual);
			}
		}
	}
}
