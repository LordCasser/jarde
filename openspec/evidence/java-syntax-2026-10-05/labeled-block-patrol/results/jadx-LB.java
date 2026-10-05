package defpackage;

/* JADX INFO: loaded from: LB.class */
public class LB {
    static int labeledBlock(int i) {
        int i2 = 0;
        if (i != 1) {
            if (i != 2) {
                i2 = 0 + 1;
            }
            i2 += 10;
        }
        return i2 + 100;
    }

    static int infBreak(int i) {
        int i2 = 0;
        do {
            i2++;
        } while (i2 <= i);
        return i2;
    }

    static int infReturn(int i) {
        int i2 = 0;
        do {
            i2++;
        } while (i2 <= i);
        return i2;
    }

    static int doInf(int i) {
        int i2 = 0;
        do {
            i2++;
        } while (i2 <= 100);
        return i2 + i;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + labeledBlock(1) + "/" + labeledBlock(2) + "/" + labeledBlock(3) + "/" + infBreak(5) + "/" + infReturn(4) + "/" + doInf(3));
    }
}
