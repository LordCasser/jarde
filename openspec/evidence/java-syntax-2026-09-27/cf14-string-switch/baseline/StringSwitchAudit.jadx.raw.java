package defpackage;

/* JADX INFO: loaded from: input.jar:StringSwitchAudit.class */
public class StringSwitchAudit {
    static int calls;

    static String read(String str) {
        calls++;
        return str;
    }

    public static int choose(String str) {
        switch (read(str)) {
            case "frewhyh":
                return 1;
            case "phgafkp":
                return 2;
            case "test":
            case "test2":
                return 3;
            case "other":
                return 4;
            default:
                return 0;
        }
    }
}
