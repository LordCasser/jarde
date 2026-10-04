public class TwoDeclSites {
    private static final StringBuilder EVENTS = new StringBuilder();

    static void event(String event) {
        if (EVENTS.length() != 0) {
            EVENTS.append('|');
        }
        EVENTS.append(event);
    }

    private static String text(String name, String value) {
        event("arg:" + name);
        return value;
    }

    private static int number(String name, int value) {
        event("arg:" + name);
        return value;
    }

    public static void main(String[] args) {
        Base first = new Base(text("a", "x"), number("b", 1)) {
            @Override
            String render() {
                event("first");
                return super.render() + ":first";
            }
        };
        Base second = new Base(text("c", "y"), number("d", 2)) {
            @Override
            String render() {
                event("second");
                return super.render() + ":second";
            }
        };
        System.out.println(first.render());
        System.out.println(second.render());
        System.out.println(EVENTS);
    }
}
