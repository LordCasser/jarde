package em23;

public class InputFieldMultiplyControls {
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

    public int multiplyDivide(int n) {
        this.a.f *= 8 / n;
        return this.a.f;
    }
}
