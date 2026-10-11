package defpackage;

/* JADX INFO: loaded from: input.jar:NullThenBuilder.class */
public class NullThenBuilder {
    static Object run(boolean z) {
        int i = 0;
        if (z && 0 == 0) {
            i = 0 + 1;
        }
        return Integer.valueOf(i + new StringBuilder("second").length());
    }

    public static void main(String[] strArr) {
        System.out.println(run(true));
    }
}
