package defpackage;

/* JADX INFO: loaded from: MV3.jar:MV3.class */
public class MV3 {
    private int base = 4;

    /* JADX INFO: loaded from: MV3.jar:MV3$Inner.class */
    class Inner {
        int tag;

        Inner(int i) {
            this.tag = i;
        }

        int total() {
            return this.tag + MV3.this.base;
        }
    }

    /* JADX INFO: loaded from: MV3.jar:MV3$StatA.class */
    static class StatA {
        StatA() {
        }

        int a() {
            return 5;
        }
    }

    /* JADX INFO: loaded from: MV3.jar:MV3$StatB.class */
    static class StatB extends StatA {
        StatB() {
        }

        int b() {
            return a() + 1;
        }
    }

    Inner make(int i) {
        return new Inner(i);
    }

    public static void main(String[] strArr) {
        System.out.println(new StatB().b());
        System.out.println(new MV3().make(2).total());
    }
}
