package defpackage;

/* JADX INFO: loaded from: StringSwitchMiddleDefault.class */
public class StringSwitchMiddleDefault {
    static int calls;
    static int score;

    static String read(String str) {
        calls++;
        if ("!".equals(str)) {
            throw new IllegalStateException("selector");
        }
        return str;
    }

    static void add(int i) {
        score = (score * 100) + i;
    }

    /* JADX WARN: Failed to restore switch over string. Please report as a decompilation issue */
    public static int choose(String str) {
        score = 0;
        String str2 = read(str);
        byte b = -1;
        switch (str2.hashCode()) {
            case -980110702:
                if (str2.equals("prefix")) {
                    b = 5;
                }
                break;
            case -891422895:
                if (str2.equals("suffix")) {
                    b = 6;
                }
                break;
            case 0:
                if (str2.equals("")) {
                    b = 3;
                }
                break;
            case 2112:
                if (str2.equals("BB")) {
                    b = 1;
                } else if (str2.equals("Aa")) {
                    b = 0;
                }
                break;
            case 38634:
                if (str2.equals("雪")) {
                    b = 4;
                }
                break;
        }
        switch (b) {
            case 0:
            case 1:
                add(1);
                break;
            case 2:
            default:
                add(2);
            case 3:
                add(3);
                break;
            case 4:
                add(4);
                break;
            case 5:
                add(5);
            case 6:
                add(6);
                break;
        }
        return score;
    }

    public static int once(String str) {
        return choose(str);
    }
}
