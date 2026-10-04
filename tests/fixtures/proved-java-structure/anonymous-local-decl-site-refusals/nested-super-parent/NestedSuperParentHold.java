class ParentCarrier {
    static class Holder {
        String render() {
            return "holder";
        }
    }
}

public class NestedSuperParentHold {
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
        ParentCarrier.Holder instance = new ParentCarrier.Holder() {
            @Override
            String render() {
                event("anon-render");
                return super.render() + ":anon";
            }
        };
        System.out.println(instance.render());
        System.out.println(EVENTS);
    }
}
