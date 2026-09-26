package defpackage;

/* JADX INFO: loaded from: fixture.jar:AnonymousSuperArgs.class */
public class AnonymousSuperArgs {
    private static final StringBuilder EVENTS = new StringBuilder();

    static void event(String str) {
        if (EVENTS.length() != 0) {
            EVENTS.append('|');
        }
        EVENTS.append(str);
    }

    private static String text(String str, String str2) {
        event("arg:" + str);
        return str2;
    }

    private static int number(String str, int i) {
        event("arg:" + str);
        return i;
    }

    public static void main(String[] strArr) {
        final String strText = text("capture", "captured");
        Base base = new Base(text("super-label", "explicit"), number("super-value", 17)) { // from class: AnonymousSuperArgs.1
            @Override // defpackage.Base
            String render() {
                AnonymousSuperArgs.event("body:" + strText);
                return super.render() + ":" + strText;
            }
        };
        System.out.println(EVENTS);
        System.out.println(base.render());
    }
}
