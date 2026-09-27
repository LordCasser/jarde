package dt29;

import java.io.Closeable;

public class InterfaceCast {
	public static Runnable asRunnable(Closeable value) {
		return (Runnable) value;
	}

	public static String choose(Closeable value) {
		return "closeable";
	}

	public static String choose(Runnable value) {
		return "runnable";
	}

	public static String chooseRunnable(Closeable value) {
		return choose((Runnable) value);
	}
}
