package defpackage;

/* JADX INFO: loaded from: BoxObject.class */
public class BoxObject {
    public static int boxed(Object obj) {
        return 5;
    }

    public static int boxed(Integer num) {
        return 6;
    }

    public static int run(int i) {
        return boxed((Object) Integer.valueOf(i));
    }
}
