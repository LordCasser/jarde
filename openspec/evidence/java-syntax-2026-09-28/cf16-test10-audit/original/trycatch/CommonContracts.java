package trycatch;

public final class CommonContracts {
	private CommonContracts() {
	}

	public static void requireNonNull(Object value) {
		if (value == null) {
			throw new NullPointerException();
		}
	}
}
