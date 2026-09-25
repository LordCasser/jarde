package nested;

public final class IdentityRunner {
    private static void run(String label, SimpleOuter checked, SimpleOuter actual) {
        SimpleOuter.trace = "";
        try {
            SimpleOuter.Inner inner = (SimpleOuter.Inner) IdentityUse.make(checked, actual, 3);
            System.out.println(label + ":" + inner.value() + ":" + SimpleOuter.trace);
        } catch (NullPointerException failure) {
            System.out.println(label + ":null:" + SimpleOuter.trace);
        }
    }

    public static void main(String[] args) {
        run("different", new SimpleOuter(4), new SimpleOuter(40));
        run("checked-null", null, new SimpleOuter(40));
        run("actual-null", new SimpleOuter(4), null);
    }
}
