public class Verify {
    public static void main(String[] args) throws Exception {
        Class.forName("jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls").getDeclaredMethods();
        System.out.println("verified");
    }
}
