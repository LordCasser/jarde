
/* JADX INFO: loaded from: input.jar:NestedStringSwitchAudit.class */
public class NestedStringSwitchAudit {
    public static int choose(String str) {
        switch (str) {
            case "a":
                return 1;
            default:
                switch (str) {
                    case "b":
                        return 2;
                    case "c":
                        return 3;
                    default:
                        return 4;
                }
        }
    }
}
