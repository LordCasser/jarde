package defpackage;

/* JADX INFO: loaded from: IV3.jar:IV3.class */
public class IV3 {
    private int seed = 2;

    /* JADX INFO: loaded from: IV3.jar:IV3$B.class */
    class B {

        /* JADX INFO: loaded from: IV3.jar:IV3$B$C.class */
        class C {
            C() {
            }

            int c() {
                return IV3.B.this.b() + 3;
            }
        }

        B() {
        }

        int b() {
            return defpackage.IV3.this.seed + 1;
        }
    }

    public static void main(java.lang.String[] strArr) {
        defpackage.IV3 iv3 = new defpackage.IV3();
        java.io.PrintStream printStream = java.lang.System.out;
        java.util.Objects.requireNonNull(iv3);
        printStream.println(iv3.new B().new C().c());
    }
}
