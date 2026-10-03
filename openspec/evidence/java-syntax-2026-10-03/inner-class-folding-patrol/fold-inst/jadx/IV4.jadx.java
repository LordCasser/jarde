package defpackage;

/* JADX INFO: loaded from: IV4.jar:IV4.class */
public class IV4 {
    private int base = 4;

    /* JADX INFO: loaded from: IV4.jar:IV4$Inner.class */
    class Inner {
        Inner() {
        }

        int total() {
            return defpackage.IV4.access$000(defpackage.IV4.this);
        }
    }

    static int access$000(defpackage.IV4 iv4) {
        return iv4.base + 100;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(new defpackage.IV4().new Inner().total());
    }
}
