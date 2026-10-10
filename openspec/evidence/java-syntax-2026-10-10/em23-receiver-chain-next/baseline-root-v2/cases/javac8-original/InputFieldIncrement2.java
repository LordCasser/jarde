package em23;

public class InputFieldIncrement2 {
    private static class A {
        int f = 5;
    }

    public A a;

    public void test1(int n) {
        this.a.f = this.a.f + n;
    }

    public void test2(int n) {
        this.a.f *= n;
    }
}
