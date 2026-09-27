package jadx.tests.integration.trycatch;

public class TargetRunner {
	public static void main(String[] args) throws Exception {
		new TestTryCatchFinally4$TestCls().test();
		System.out.println("target.test() returned");
	}
}
