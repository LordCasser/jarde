package defpackage;

/* JADX INFO: loaded from: IV2.jar:IV2.class */
public class IV2 {
    private int count = 10;

    /* JADX INFO: loaded from: IV2.jar:IV2$Inner.class */
    class Inner {
        Inner() {
        }

        void bump() {
            defpackage.IV2.this.count++;
        }
    }

    int get() {
        return this.count;
    }

    public static void main(java.lang.String[] strArr) {
        defpackage.IV2 iv2 = new defpackage.IV2();
        java.util.Objects.requireNonNull(iv2);
        iv2.new Inner().bump();
        java.lang.System.out.println(iv2.get());
    }
}
