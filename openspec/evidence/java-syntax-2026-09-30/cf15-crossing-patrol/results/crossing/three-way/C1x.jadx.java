package defpackage;

/* JADX INFO: loaded from: C1x.class */
public class C1x {
    public static void ns(int i) {
        if (i != 0) {
            throw new NoSuchFieldError("injected");
        }
    }

    public static void is(int i) {
        if (i != 0) {
            throw new IllegalStateException("injected");
        }
    }

    public static void re(int i) {
        if (i != 0) {
            throw new RuntimeException("injected");
        }
    }

    public static int work(int i) {
        return i + 1;
    }

    public static String swallow(int i) {
        try {
            ns(i);
            work(1);
            return "ok";
        } catch (NoSuchFieldError e) {
            return "missed";
        }
    }

    public static String five(int i) {
        StringBuilder sb = new StringBuilder();
        try {
            ns(i);
            sb.append('a');
        } catch (NoSuchFieldError e) {
        }
        try {
            sb.append('b');
        } catch (NoSuchFieldError e2) {
        }
        try {
            is(i);
            sb.append('c');
        } catch (IllegalStateException e3) {
        }
        try {
            sb.append('d');
        } catch (NoSuchFieldError e4) {
        }
        try {
            re(i);
            sb.append('e');
        } catch (RuntimeException e5) {
        }
        return sb.toString();
    }

    public static void main(String[] strArr) {
        System.out.println(swallow(strArr.length));
        System.out.println(five(strArr.length));
    }
}
