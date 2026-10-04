public class NestedAnonAlloc {
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
        Base instance = new Base(text("a", "x"), number("b", 1)) {
            @Override
            String render() {
                Runnable helper = new Runnable() {
                    @Override
                    public void run() {
                        event("helper-ran");
                    }
                };
                helper.run();
                return super.render();
            }
        };
        System.out.println(instance.render());
        System.out.println(EVENTS);
    }
}
