package defpackage;

/* JADX INFO: loaded from: IV1.jar:IV1.class */
public class IV1 {
    private int base = 4;
    private int bonus = 3;

    /* JADX INFO: loaded from: IV1.jar:IV1$Inner.class */
    class Inner {
        private int tag;

        Inner(int i) {
            this.tag = i;
        }

        int total() {
            return this.tag + defpackage.IV1.this.base + defpackage.IV1.this.bonus;
        }
    }

    IV1.Inner make(int i) {
        return new IV1.Inner(i);
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(new defpackage.IV1().make(5).total());
    }
}
