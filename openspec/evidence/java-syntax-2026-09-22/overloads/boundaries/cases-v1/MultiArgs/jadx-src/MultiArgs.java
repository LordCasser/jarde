
/* JADX INFO: loaded from: MultiArgs.class */
public class MultiArgs {
    public static int pick(Object obj, Object obj2) {
        return 1;
    }

    public static int pick(String str, String str2) {
        return 2;
    }

    public static int run() {
        return pick((Object) null, "x");
    }
}
