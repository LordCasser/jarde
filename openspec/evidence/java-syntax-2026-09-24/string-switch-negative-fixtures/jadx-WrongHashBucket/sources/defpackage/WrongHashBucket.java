package defpackage;

/* JADX INFO: loaded from: WrongHashBucket.class */
public class WrongHashBucket {
    static int calls;

    static String read(String str) {
        calls++;
        return str;
    }

    public static int choose(String str) {
        String str2 = read(str);
        byte b = -1;
        switch (str2.hashCode()) {
            case 2112:
                if (str2.equals("abc")) {
                    b = 1;
                }
                break;
            case 96354:
                if (str2.equals("Aa")) {
                    b = 0;
                }
                break;
        }
        switch (b) {
            case 0:
                return 10;
            case 1:
                return 20;
            default:
                return 40;
        }
    }
}
