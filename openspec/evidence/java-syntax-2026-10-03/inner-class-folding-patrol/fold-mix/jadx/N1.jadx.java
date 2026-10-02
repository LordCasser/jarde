package defpackage;

import java.util.Objects;

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
            return this.tag + N1.this.base;
        }
    }

    /* JADX INFO: loaded from: fam.jar:N1$Stat.class */
    static class Stat {
        Stat() {
        }

        int m() {
            return 1;
        }

        int use(N1 n1) {
            Objects.requireNonNull(n1);
            return n1.new Inner(9).total();
        }
    }

    Inner make(int i) {
        return new Inner(i);
    }

    static int useStatic() {
        return new N1().make(6).total();
    }

    public static void main(String[] strArr) {
        System.out.println(useStatic());
        System.out.println(new N1().new Inner(3).total());
        System.out.println(new Stat().use(new N1()));
    }
}
