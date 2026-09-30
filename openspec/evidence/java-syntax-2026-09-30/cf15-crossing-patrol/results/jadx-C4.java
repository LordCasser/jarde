package defpackage;

/* JADX INFO: loaded from: C4.class */
public class C4 implements AutoCloseable {
    @Override // java.lang.AutoCloseable
    public void close() {
    }

    public static String twrNamed() throws Exception {
        try {
            C4 c4 = new C4();
            try {
                c4.toString();
                c4.close();
                return "done";
            } catch (Throwable th) {
                try {
                    c4.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught";
        }
    }

    public static String constructNamed() {
        StringBuilder sb = new StringBuilder();
        try {
            sb.append('a');
            return sb.toString();
        } catch (IllegalStateException e) {
            return "caught";
        }
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(twrNamed());
        System.out.println(constructNamed());
    }
}
