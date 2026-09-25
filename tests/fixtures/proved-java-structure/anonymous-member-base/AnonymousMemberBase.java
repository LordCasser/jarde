public final class AnonymousMemberBase {
    private static final StringBuilder EVENTS = new StringBuilder();

    static class Outer {
        class Base {
            final int value;

            Base(int value) {
                this.value = value;
                event("base(" + value + ")");
            }

            int render() {
                return value;
            }
        }
    }

    private static void event(String value) {
        if (EVENTS.length() > 0) {
            EVENTS.append(',');
        }
        EVENTS.append(value);
    }

    private static Outer outer() {
        event("outer");
        return new Outer();
    }

    private static Outer nullOuter() {
        event("nullOuter");
        return null;
    }

    private static int sideEffect() {
        event("argument");
        return 7;
    }

    private static Outer.Base normal() {
        Outer outer = outer();
        return outer.new Base(sideEffect()) {
            @Override
            int render() {
                event("anonymous");
                return value + 1;
            }
        };
    }

    private static Outer.Base nullPath() {
        Outer outer = nullOuter();
        return outer.new Base(sideEffect()) {
            @Override
            int render() {
                event("anonymous-null");
                return value + 1;
            }
        };
    }

    public static void main(String[] args) {
        Outer.Base value = normal();
        System.out.println("normal=" + value.render() + ";events=" + EVENTS);
        EVENTS.setLength(0);
        try {
            nullPath();
            throw new AssertionError("expected null receiver failure");
        } catch (NullPointerException expected) {
            System.out.println("null=" + EVENTS);
        }
    }
}
