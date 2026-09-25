
/* JADX INFO: loaded from: NullObject.class */
public class NullObject {
    public static int choose(Object obj) {
        return 1;
    }

    public static int choose(String str) {
        return 2;
    }

    public static int run() {
        return choose((Object) null);
    }
}
