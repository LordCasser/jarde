package cf11;

/* JADX INFO: loaded from: sequential-loops2.jar:cf11/UpstreamSequentialLoops2.class */
public final class UpstreamSequentialLoops2 {
    private static char[] lowercases = {'a'};

    public static String asciiToLowerCase(String s) {
        char c2;
        char[] c = null;
        int i = s.length();
        while (true) {
            int i2 = i;
            i--;
            if (i2 > 0) {
                char c1 = s.charAt(i);
                if (c1 <= 127 && c1 != (c2 = lowercases[c1])) {
                    c = s.toCharArray();
                    c[i] = c2;
                    break;
                }
            } else {
                break;
            }
        }
        while (true) {
            int i3 = i;
            i--;
            if (i3 <= 0) {
                break;
            }
            if (c[i] <= 127) {
                c[i] = lowercases[c[i]];
            }
        }
        return c == null ? s : new String(c);
    }
}
