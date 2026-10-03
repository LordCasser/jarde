package defpackage;

/* JADX INFO: loaded from: fam.jar:N1.class */
public class N1 {
    private int base = 4;

    /* JADX INFO: loaded from: fam.jar:N1$Inner.class */
    class Inner {
        private int tag;

        Inner(int i) {
            this.tag = i;
        }

        int total() {
            return this.tag + defpackage.N1.this.base;
        }
    }

    /* JADX INFO: loaded from: fam.jar:N1$Stat.class */
    static class Stat {
        Stat() {
        }

        int m() {
            return 1;
        }

        int use(defpackage.N1 n1) {
            java.util.Objects.requireNonNull(n1);
            return n1.new Inner(9).total();
        }
    }

    N1.Inner make(int i) {
        return new N1.Inner(i);
    }

    static int useStatic() {
        return new defpackage.N1().make(6).total();
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(useStatic());
        java.lang.System.out.println(new defpackage.N1().new Inner(3).total());
        java.lang.System.out.println(new N1.Stat().use(new defpackage.N1()));
    }
}
