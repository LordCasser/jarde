package defpackage;

/* JADX INFO: loaded from: BF.class */
public class BF {
    private int flags = 0;

    void enable(int i) {
        this.flags |= 1 << i;
    }

    void disable(int i) {
        this.flags &= (1 << i) ^ (-1);
    }

    boolean isSet(int i) {
        return (this.flags & (1 << i)) != 0;
    }

    boolean isEmpty() {
        return this.flags == 0;
    }

    int count() {
        int i = 0;
        int i2 = this.flags;
        while (true) {
            int i3 = i2;
            if (i3 == 0) {
                return i;
            }
            i += i3 & 1;
            i2 = i3 >>> 1;
        }
    }

    public static void main(java.lang.String[] strArr) {
        defpackage.BF bf = new defpackage.BF();
        bf.enable(0);
        bf.enable(3);
        bf.enable(5);
        java.lang.System.out.println("" + bf.isSet(0) + "/" + bf.isSet(1) + "/" + bf.isSet(3) + "/" + bf.isEmpty() + "/" + bf.count());
        bf.disable(3);
        java.lang.System.out.println("" + bf.isSet(3) + "/" + bf.count());
    }
}
