public class SpecialRunner {
    private interface Call {
        int run();
    }

    private static void print(String name, Call call) {
        try {
            System.out.println(name + "=" + call.run());
        } catch (Throwable error) {
            System.out.println(name + "=" + error.getClass().getName());
        }
    }

    public static void main(String[] args) {
        final SpecialProbe special = new SpecialProbe();
        print("value", new Call() {
            public int run() {
                return special.value();
            }
        });
        print("defaultCall", new Call() {
            public int run() {
                return special.defaultCall();
            }
        });
        print("own", new Call() {
            public int run() {
                return special.callOwnPrivate(5);
            }
        });
        print("other", new Call() {
            public int run() {
                return special.callOtherPrivate(new SpecialProbe(3), 5);
            }
        });
        print("otherNull", new Call() {
            public int run() {
                return special.callOtherPrivate(null, 5);
            }
        });
        print("superSideEffect", new Call() {
            public int run() {
                return special.superWithSideEffect();
            }
        });
        print("sideEffectCount", new Call() {
            public int run() {
                return BaseProbe.sideEffectCount();
            }
        });
        print("superThrowingArgument", new Call() {
            public int run() {
                return special.superWithThrowingArgument();
            }
        });
        print("superThrowingParent", new Call() {
            public int run() {
                return special.superThrowing();
            }
        });
    }
}
