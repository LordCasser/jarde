public final class AnonymousCaptureCases {
    private static final StringBuilder EVENTS = new StringBuilder();
    private static int captureCalls;
    private static int chooseCalls;
    private static int baseCalls;
    private static int renderCalls;

    interface Renderer {
        String render();
    }

    static abstract class Base implements Renderer {
        private final long seed;

        Base(long seed) {
            this.seed = seed;
            baseCalls++;
            event("base(" + seed + ")");
        }

        long seed() {
            return seed;
        }
    }

    private static String captureLocal() {
        captureCalls++;
        event("capture");
        return "captured";
    }

    private static long choose() {
        chooseCalls++;
        event("choose");
        return 7L;
    }

    private static Renderer baseArgumentAndCapture() {
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

    static final class Outer {
        private final int state;

        Outer(int state) {
            this.state = state;
        }

        Renderer captureOther(Outer other) {
            return new Renderer() {
                @Override
                public String render() {
                    return other.state + ":" + Outer.this.state;
                }
            };
        }
    }

    private static void event(String value) {
        if (EVENTS.length() > 0) {
            EVENTS.append(',');
        }
        EVENTS.append(value);
    }

    public static void main(String[] args) {
        Renderer baseCase = baseArgumentAndCapture();
        System.out.println("base=" + baseCase.render());
        System.out.println("events=" + EVENTS);
        System.out.println("counts=" + captureCalls + "," + chooseCalls + "," + baseCalls + "," + renderCalls);

        Renderer receiverCase = new Outer(10).captureOther(new Outer(20));
        System.out.println("receiver=" + receiverCase.render());
    }
}
