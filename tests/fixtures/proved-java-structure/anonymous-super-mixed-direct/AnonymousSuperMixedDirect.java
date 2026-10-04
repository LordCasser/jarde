public class AnonymousSuperMixedDirect {
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

    private static String capture() {
        event("capture");
        return "captured";
    }

    static Base make() {
        final String captured = capture();
        return new Base(text("super-label", "explicit"), number("super-value", 17)) {
            @Override
            String render() {
                event(captured);
                return super.render();
            }
        };
    }

    public static void main(String[] args) {
        Base instance = make();
        System.out.println(EVENTS);
        String rendered = instance.render();
        System.out.println(rendered);
        System.out.println(EVENTS);
    }
}

class Base {
    private final String label;
    private final int value;

    Base(String label, int value) {
        this.label = label;
        this.value = value;
        AnonymousSuperMixedDirect.event("base:" + label + ":" + value);
    }

    Base(int value, String label) {
        this.label = label;
        this.value = value;
        AnonymousSuperMixedDirect.event("base-int:" + label + ":" + value);
    }

    String render() {
        return label + ":" + value;
    }
}
