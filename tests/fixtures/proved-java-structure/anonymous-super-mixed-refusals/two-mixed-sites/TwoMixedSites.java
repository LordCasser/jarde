public class TwoMixedSites {
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

    static Base make1() {
        final String captured = capture("one");
        return new Base(text("a", "x"), number("b", 1)) {
            @Override
            String render() {
                event(captured);
                return super.render();
            }
        };
    }

    static Base make2() {
        final String captured = capture("two");
        return new Base(text("c", "y"), number("d", 2)) {
            @Override
            String render() {
                event(captured);
                return super.render();
            }
        };
    }

    public static void main(String[] args) {
        Base first = make1();
        Base second = make2();
        System.out.println(EVENTS);
        System.out.println(first.render());
        System.out.println(second.render());
        System.out.println(EVENTS);
    }
}

class Base {
    private final String label;
    private final int value;

    Base(String label, int value) {
        this.label = label;
        this.value = value;
        TwoMixedSites.event("base:" + label + ":" + value);
    }

    String render() {
        return label + ":" + value;
    }
}
