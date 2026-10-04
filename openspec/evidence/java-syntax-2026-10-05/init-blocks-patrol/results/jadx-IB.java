package defpackage;

/* JADX INFO: loaded from: IB.class */
public class IB {
    static int sc = 41;
    int ic;

    static {
        sc++;
    }

    IB(int i) {
        this.ic = 10;
        this.ic += 5;
        this.ic += i;
    }

    IB() {
        this(1);
    }

    static int more() {
        sc += 100;
        return sc;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(sc + "/" + new defpackage.IB().ic + "/" + more());
    }
}
