package defpackage;

/* JADX INFO: loaded from: C4x.class */
public class C4x implements AutoCloseable {
    @Override // java.lang.AutoCloseable
    public void close() {
    }

    public static void is(int i) {
        if (i != 0) {
            throw new IllegalStateException("injected");
        }
    }

    public static String constructNamed(int i) {
        StringBuilder sb = new StringBuilder();
        try {
            is(i);
            sb.append('a');
            return sb.toString();
        } catch (IllegalStateException e) {
            return "caught";
        }
    }

    public static void main(String[] strArr) {
        System.out.println(constructNamed(strArr.length));
    }
}
