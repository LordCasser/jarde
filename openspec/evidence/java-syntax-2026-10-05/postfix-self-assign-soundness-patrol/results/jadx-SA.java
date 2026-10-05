package defpackage;

/* JADX INFO: loaded from: SA.class */
public class SA {
    int foo;

    static int postSelf() {
        int i = 5 + 1;
        return 5;
    }

    static int preSelf() {
        return 5 + 1;
    }

    static int postOther() {
        return ((5 + 1) * 10) + 5;
    }

    static int seqInc() {
        return 5 + 1 + 1;
    }

    int foo() {
        return this.foo + 1;
    }

    int useBoth() {
        return foo() + this.foo;
    }

    static java.lang.String uni() {
        return "aAb";
    }

    static char charArith(char c) {
        return (char) (c + 1);
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + postSelf() + "/" + preSelf() + "/" + postOther() + "/" + seqInc() + "/" + new defpackage.SA().useBoth() + "/" + uni() + "/" + charArith('x'));
    }
}
