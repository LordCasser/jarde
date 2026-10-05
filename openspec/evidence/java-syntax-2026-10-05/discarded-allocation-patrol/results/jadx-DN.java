package defpackage;

/* JADX INFO: loaded from: DN.class */
public class DN {
    static void discarded() {
        new DN.N();
        java.lang.System.out.println("after");
    }

    static void used() {
        new DN.N().hi();
    }

    static void chained() {
        new DN.N().hi();
    }

    static void argUse() {
        takes(new DN.N());
    }

    static void takes(DN.N n) {
        n.hi();
    }

    public static void main(java.lang.String[] strArr) {
        discarded();
        used();
        chained();
        argUse();
    }
}
