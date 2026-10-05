package defpackage;

/* JADX INFO: loaded from: CH.class */
public class CH {
    static int a;
    static int b;
    static int c;
    static int[] arr = new int[3];
    static int idx = 0;

    static void chain() {
        c = 5;
        b = 5;
        a = 5;
    }

    static int chainLocal() {
        return 7 + 7;
    }

    static void sideIdx() {
        int[] iArr = arr;
        int i = idx;
        idx = i + 1;
        iArr[i] = 10;
        int[] iArr2 = arr;
        int i2 = idx;
        idx = i2 + 1;
        iArr2[i2] = 20;
    }

    static int sideRead() {
        int[] iArr = arr;
        int i = idx;
        idx = i - 1;
        return iArr[i];
    }

    public static void main(java.lang.String[] strArr) {
        chain();
        sideIdx();
        java.lang.System.out.println("" + a + "/" + b + "/" + c + "/" + chainLocal() + "/" + arr[0] + "/" + arr[1] + "/" + idx + "/" + sideRead() + "/" + idx);
    }
}
