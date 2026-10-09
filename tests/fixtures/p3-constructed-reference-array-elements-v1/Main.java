public class Main {
    public static void event(String kind, String value) {
        System.out.print(kind);
        System.out.print(':');
        System.out.println(value);
    }

    public static String mark(String value) {
        event("mark", value);
        return value;
    }

    public static CharSequence[] sequence() {
        return new CharSequence[]{
            new StringBuilder(mark("sequence-first")),
            new StringBuffer(mark("sequence-second"))
        };
    }

    public static java.util.Collection<?>[] collections() {
        return new java.util.Collection<?>[]{
            new java.util.ArrayList<String>(java.util.Arrays.asList(mark("collection-first"))),
            new java.util.HashSet<String>(java.util.Arrays.asList(mark("collection-second")))
        };
    }

    public static Throwable[] failures() {
        return new Throwable[]{
            new IllegalStateException(mark("throwable-first")),
            new IllegalArgumentException(mark("throwable-second"))
        };
    }

    public static Base[] ownDirect() {
        return new Base[]{
            new DirectA(mark("direct-first")),
            new DirectB(mark("direct-second"))
        };
    }

    public static Base[] ownTwoHop() {
        return new Base[]{
            new TwoHop(mark("two-hop-first")),
            new DirectB(mark("two-hop-second"))
        };
    }

    public static LocalInterface[] ownInterface() {
        return new LocalInterface[]{
            new DirectA(mark("interface-first")),
            new TwoHop(mark("interface-second"))
        };
    }

    private static void observe(Object[] values) {
        Object first = values[0];
        if (first == null) {
            System.out.println("null");
        } else {
            System.out.println(first.getClass().getName());
        }
        Object second = values[1];
        if (second == null) {
            System.out.println("null");
        } else {
            System.out.println(second.getClass().getName());
        }
    }

    public static void main(String[] args) {
        observe(sequence());
        observe(collections());
        observe(failures());
        observe(ownDirect());
        observe(ownTwoHop());
        observe(ownInterface());
    }
}
