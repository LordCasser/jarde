package defpackage;

/* JADX INFO: loaded from: MV2.jar:MV2.class */
public class MV2 {

    /* JADX INFO: loaded from: MV2.jar:MV2$Inner.class */
    class Inner {
        int tag;

        Inner(int i) {
            this.tag = i;
        }

        int total() {
            return this.tag * 2;
        }
    }

    /* JADX INFO: loaded from: MV2.jar:MV2$Stat.class */
    static class Stat {
        Stat() {
        }

        int m() {
            return 11;
        }
    }

    public static void main(String[] strArr) {
        System.out.println(new Stat().m());
        System.out.println(new MV2().new Inner(4).total());
    }
}
