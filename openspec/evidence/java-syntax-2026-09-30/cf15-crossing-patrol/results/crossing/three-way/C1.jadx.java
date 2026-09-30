package defpackage;

/* JADX INFO: loaded from: C1.class */
public class C1 {
    public static int work(int i) {
        return i + 1;
    }

    public static String swallow() {
        try {
            work(1);
            return "ok";
        } catch (NoSuchFieldError e) {
            return "missed";
        }
    }

    public static String five() {
        StringBuilder sb = new StringBuilder();
        try {
            sb.append('a');
        } catch (NoSuchFieldError e) {
        }
        try {
            sb.append('b');
        } catch (NoSuchFieldError e2) {
        }
        try {
            sb.append('c');
        } catch (IllegalStateException e3) {
        }
        try {
            sb.append('d');
        } catch (NoSuchFieldError e4) {
        }
        try {
            sb.append('e');
        } catch (RuntimeException e5) {
        }
        return sb.toString();
    }

    public static void main(String[] strArr) {
        System.out.println(swallow());
        System.out.println(five());
    }
}
