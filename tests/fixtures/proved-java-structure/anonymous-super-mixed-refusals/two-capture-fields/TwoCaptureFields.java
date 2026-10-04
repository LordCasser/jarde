public class TwoCaptureFields {
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

    private static String capture(String name) {
        event("capture:" + name);
        return name;
    }

    static Base make() {
        final String first = capture("first");
        final String second = capture("second");
        return new Base(text("super-label", "explicit"), number("super-value", 17)) {
            @Override
            String render() {
                event(first);
                event(second);
                return super.render();
            }
        };
    }

    public static void main(String[] args) {
        Base instance = make();
        System.out.println(EVENTS);
        System.out.println(instance.render());
        System.out.println(EVENTS);
    }
}

class Base {
    private final String label;
    private final int value;

    Base(String label, int value) {
        this.label = label;
        this.value = value;
        TwoCaptureFields.event("base:" + label + ":" + value);
    }

    String render() {
        return label + ":" + value;
    }
}
