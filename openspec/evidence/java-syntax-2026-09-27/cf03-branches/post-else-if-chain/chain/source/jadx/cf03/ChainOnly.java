package cf03;

/* JADX INFO: loaded from: chain.jar:cf03/ChainOnly.class */
public class ChainOnly {
    public static int hits;

    private static boolean matches(String str, String str2) {
        hits++;
        return str.equals(str2);
    }

    public static int chain(String str) {
        int i;
        hits = 0;
        if (matches(str, "a")) {
            i = 1;
        } else if (matches(str, "b")) {
            i = 2;
        } else if (matches(str, "3")) {
            i = 3;
        } else if (matches(str, "$")) {
            i = 4;
        } else {
            i = -1;
            hits += 10;
        }
        return Math.abs(i * 10);
    }
}
