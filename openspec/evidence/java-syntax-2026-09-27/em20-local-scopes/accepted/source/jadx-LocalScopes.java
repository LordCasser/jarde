package em20;

/* JADX INFO: loaded from: input.jar:em20/LocalScopes.class */
public class LocalScopes {
    public static int joined(boolean z, int i) {
        return z ? i + 1 : i - 1;
    }

    public static int loop(int i) {
        int i2 = 0;
        for (int i3 = 0; i3 < i; i3++) {
            i2 += i3 + 1;
        }
        return i2;
    }

    public static int synchronizedLoop(int i) {
        int i2;
        synchronized (LocalScopes.class) {
            i2 = i + 1;
        }
        int i3 = 0;
        for (int i4 = 0; i4 < i2; i4++) {
            i3 += i4;
        }
        return i3;
    }

    public static int caught(boolean z) {
        int i;
        if (z) {
            try {
                throw new IllegalArgumentException("requested");
            } catch (IllegalArgumentException e) {
                i = 3;
            }
        } else {
            i = 2;
        }
        return i;
    }
}
