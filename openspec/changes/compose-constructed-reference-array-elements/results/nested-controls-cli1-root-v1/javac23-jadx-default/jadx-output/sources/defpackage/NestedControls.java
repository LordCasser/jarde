package defpackage;

/* JADX INFO: loaded from: javac23.jar:NestedControls.class */
public class NestedControls {
    static String mark(String str) {
        System.out.print("mark:");
        System.out.println(str);
        return str;
    }

    static Object[] nested() {
        return new Object[]{new StringBuilder(new StringBuilder(mark("nested")))};
    }

    public static void main(String[] strArr) {
        System.out.println(nested()[0]);
    }
}
