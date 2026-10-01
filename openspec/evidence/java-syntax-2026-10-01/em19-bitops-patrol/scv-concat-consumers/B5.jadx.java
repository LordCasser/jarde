package defpackage;

/* JADX INFO: loaded from: B5.class */
public class B5 {
    public static String s1(int i) {
        return (((i & 1) == 0 || (i & 2) == 0) ? false : true) + ":";
    }

    public static String s2(int i) {
        return (((i & 1) == 0 || (i & 2) == 0) ? false : true) + ":" + ((i & 4) != 0);
    }

    public static boolean s3(int i) {
        return ((i & 1) == 0 || (i & 2) == 0) ? false : true;
    }

    public static void main(String[] strArr) {
        System.out.println(s1(3));
        System.out.println(s2(3));
        System.out.println(s3(3));
    }
}
