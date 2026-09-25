package nested;

public final class InnerRunner {
    public static void main(String[] args) {
        for (SimpleOuter outer : new SimpleOuter[] { new SimpleOuter(4), null }) {
            SimpleOuter.trace = "";
            try {
                Object value = UseInner.make(outer, SimpleOuter.mark("I", 3));
                System.out.println(((SimpleOuter.Inner) value).value() + ":" + SimpleOuter.trace);
            } catch (NullPointerException failure) {
                System.out.println("null:" + SimpleOuter.trace);
            }
        }
    }
}
