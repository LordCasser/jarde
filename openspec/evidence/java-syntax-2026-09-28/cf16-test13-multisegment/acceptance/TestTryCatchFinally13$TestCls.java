package jadx.tests.integration.trycatch;

public class TestTryCatchFinally13$TestCls {
	public static final StringBuilder TRACE = new StringBuilder();
	public static int THROW_AT = -1;
	public static RuntimeException LAST_FAILURE;

	public void test(int i) {
		try {
			doSomething1();
			if (i == -12) {
				return;
			}
			if (i > 10) {
				doSomething2();
			} else if (i == -1) {
				doSomething3();
			}
		} catch (Exception ex) {
			logError();
		} finally {
			doSomething4();
		}
	}

	void logError() {
		TRACE.append("catch:").append(LAST_FAILURE.getClass().getSimpleName()).append(',');
	}

	void doSomething1() {
		TRACE.append("do1,");
		if (THROW_AT == 1) {
			LAST_FAILURE = new IllegalArgumentException();
			TRACE.append("throw:IllegalArgumentException,");
			throw LAST_FAILURE;
		}
	}
	void doSomething2() {
		TRACE.append("do2,");
		if (THROW_AT == 2) {
			LAST_FAILURE = new IllegalStateException();
			TRACE.append("throw:IllegalStateException,");
			throw LAST_FAILURE;
		}
	}
	void doSomething3() {
		TRACE.append("do3,");
		if (THROW_AT == 3) {
			LAST_FAILURE = new UnsupportedOperationException();
			TRACE.append("throw:UnsupportedOperationException,");
			throw LAST_FAILURE;
		}
	}
	void doSomething4() {
		TRACE.append("finally,");
	}
}
