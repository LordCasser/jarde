package defpackage;

/* JADX INFO: loaded from: IBRInst.class */
public class IBRInst {
    int x = 4;
    int y;

    IBRInst() {
        if (this.x < 0) {
            throw new IllegalStateException("never");
        }
        this.y = this.x * 2;
    }

    IBRInst(int i) {
        if (this.x < 0) {
            throw new IllegalStateException("never");
        }
        this.y = this.x * 2;
        this.y += i;
    }

    public static void main(String[] strArr) {
        IBRInst iBRInst = new IBRInst();
        IBRInst iBRInst2 = new IBRInst(10);
        System.out.println(iBRInst.x + ":" + iBRInst.y + ":" + iBRInst2.x + ":" + iBRInst2.y);
    }
}
