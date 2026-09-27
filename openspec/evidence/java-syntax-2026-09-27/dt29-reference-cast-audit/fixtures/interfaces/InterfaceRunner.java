package dt29;

import java.io.Closeable;

public class InterfaceRunner {
	public static void main(String[] args) {
		Both both = new Both();
		Closeable accepted = both;
		if (InterfaceCast.asRunnable(accepted) != both) {
			throw new AssertionError("cast identity");
		}
		if (!"runnable".equals(InterfaceCast.chooseRunnable(accepted))) {
			throw new AssertionError("overload target");
		}
		boolean failed = false;
		try {
			InterfaceCast.asRunnable(new CloseOnly());
		} catch (ClassCastException expected) {
			failed = true;
		}
		if (!failed) {
			throw new AssertionError("missing runtime cast failure");
		}
		System.out.println("runnable:ClassCastException");
	}
}
