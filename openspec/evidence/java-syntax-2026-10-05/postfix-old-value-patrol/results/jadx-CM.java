package defpackage;

/* JADX INFO: loaded from: CM.class */
public class CM {
    static int a;
    static int b;

    static int compound() {
        return ((((10 + 5) - 3) * 2) / 4) % 4;
    }

    static int compoundInExpr() {
        int i = 10 + 5;
        return i + i;
    }

    static int compoundField() {
        a += 3;
        b--;
        return a + b;
    }

    static int incDec() {
        int i = 5 + 1 + 1;
        int i2 = (i - 1) - 1;
        return i2 + 5 + i + i + i2;
    }

    static int incField() {
        a++;
        a++;
        return a;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + compound() + "/" + compoundInExpr() + "/" + compoundField() + "/" + incDec() + "/" + incField());
    }
}
