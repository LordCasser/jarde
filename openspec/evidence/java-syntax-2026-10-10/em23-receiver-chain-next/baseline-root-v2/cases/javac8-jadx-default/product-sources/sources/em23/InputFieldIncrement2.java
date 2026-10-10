package em23;

/* JADX INFO: loaded from: InputFieldIncrement2-family.jar:em23/InputFieldIncrement2.class */
public class InputFieldIncrement2 {
    public A a;

    /* JADX INFO: loaded from: InputFieldIncrement2-family.jar:em23/InputFieldIncrement2$A.class */
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
}
