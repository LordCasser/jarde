public class LocalDeclInterfaceHold {
    private static final StringBuilder EVENTS = new StringBuilder();

    static void event(String event) {
        if (EVENTS.length() != 0) {
            EVENTS.append('|');
        }
        EVENTS.append(event);
    }

    private static String label(String name) {
        event("label:" + name);
        return name;
    }

    public static void main(String[] args) {
        Runnable handler = new Runnable() {
            @Override
            public void run() {
                event("run:" + label("inside"));
            }
        };
        event("after-allocation");
        handler.run();
        System.out.println(EVENTS);
    }
}
