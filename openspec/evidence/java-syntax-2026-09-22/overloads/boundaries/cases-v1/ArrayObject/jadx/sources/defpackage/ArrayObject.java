package defpackage;

/* JADX INFO: loaded from: ArrayObject.class */
public class ArrayObject {
    public static int arr(Object obj) {
        return 3;
    }

    public static int arr(String[] strArr) {
        return 4;
    }

    public static int run(String[] strArr) {
        return arr((Object) strArr);
    }
}
