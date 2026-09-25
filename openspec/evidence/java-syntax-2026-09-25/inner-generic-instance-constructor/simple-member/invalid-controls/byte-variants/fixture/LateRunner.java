package nested;

public final class LateRunner {
    public static void main(String[] args) {
        for (SimpleOuter outer : new SimpleOuter[] { new SimpleOuter(4), null }) {
            SimpleOuter.trace = "";
            try {
                SimpleOuter.Inner inner = (SimpleOuter.Inner) LateUse.make(outer, 3);
                System.out.println(inner.value() + ":" + SimpleOuter.trace);
            } catch (NullPointerException failure) {
                System.out.println("null:" + SimpleOuter.trace);
            }
        }
    }
}
