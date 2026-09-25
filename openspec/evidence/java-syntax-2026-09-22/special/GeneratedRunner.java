public class GeneratedRunner {
    private interface Call {
        int run();
    }

    private static void check(String name, Call call) {
        try {
            System.out.println(name + "=" + call.run());
        } catch (Throwable error) {
            System.out.println(name + "=" + error.getClass().getName());
        }
    }

    public static void main(String[] args) {
        final SpecialProbe special = new SpecialProbe();
        check("value", new Call() {
            public int run() {
                return special.value();
            }
        });
        check("defaultCall", new Call() {
            public int run() {
                return special.defaultCall();
            }
        });
        check("own", new Call() {
            public int run() {
                return special.callOwnPrivate(5);
            }
        });
        check("other", new Call() {
            public int run() {
                return special.callOtherPrivate(new SpecialProbe(), 5);
            }
        });
    }
}
