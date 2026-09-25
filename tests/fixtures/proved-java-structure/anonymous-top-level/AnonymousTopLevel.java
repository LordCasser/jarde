interface Renderer {
    String render();
}

abstract class Base implements Renderer {
    private final long seed;

    Base(long seed) {
        this.seed = seed;
        AnonymousTopLevel.baseCalls++;
        AnonymousTopLevel.event("base(" + seed + ")");
    }

    long seed() {
        return seed;
    }
}

public final class AnonymousTopLevel {
    private static final StringBuilder EVENTS = new StringBuilder();
    private static int captureCalls;
    private static int chooseCalls;
    static int baseCalls;
    static int renderCalls;

    private static String captureLocal() {
        captureCalls++;
        event("capture");
        return "captured";
    }

    private static long choose() {
        chooseCalls++;
        event("choose");
        return 23L;
    }

    static void event(String value) {
        if (EVENTS.length() > 0) {
            EVENTS.append(',');
        }
        EVENTS.append(value);
    }

    static Renderer create() {
        final String captured = captureLocal();
        return new Base(choose()) {
            @Override
            public String render() {
                renderCalls++;
                event("render");
                return seed() + ":" + captured;
            }
        };
    }

    public static void main(String[] args) {
        Renderer renderer = create();
        System.out.println("value=" + renderer.render());
        System.out.println("events=" + EVENTS);
        System.out.println("counts=" + captureCalls + "," + chooseCalls + "," + baseCalls + "," + renderCalls);
    }
}
