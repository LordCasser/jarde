package defpackage;

/* JADX INFO: loaded from: nl.jar:NL.class */
public class NL {
    static final java.lang.Object LOCK = new java.lang.Object();

    /* JADX INFO: loaded from: nl.jar:NL$Box.class */
    static class Box {
        Box() {
        }

        public java.lang.String toString() {
            return java.lang.Thread.holdsLock(defpackage.NL.class) ? "Y" : "N";
        }
    }

    static java.lang.String probe(java.lang.Object obj) {
        java.lang.String str;
        synchronized (LOCK) {
            synchronized (defpackage.NL.class) {
                str = "n" + obj;
            }
        }
        return str;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(probe(new NL.Box()));
    }
}
