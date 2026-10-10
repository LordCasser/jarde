package em23;

/* JADX INFO: loaded from: InputFieldMultiplyControls-family.jar:em23/InputFieldMultiplyControls.class */
public class InputFieldMultiplyControls {
    public A a;

    /* JADX INFO: loaded from: InputFieldMultiplyControls-family.jar:em23/InputFieldMultiplyControls$A.class */
    private static class A {
        int f = 5;

        private A() {
        }
    }

    public void test1(int i) {
        this.a.f += i;
    }

    public void test2(int i) {
        this.a.f *= i;
    }

    public int multiplyDivide(int i) {
        this.a.f *= 8 / i;
        return this.a.f;
    }
}
