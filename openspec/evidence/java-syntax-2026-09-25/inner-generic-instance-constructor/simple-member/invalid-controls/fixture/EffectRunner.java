package nested;

public final class EffectRunner {
    public static void main(String[] args) {
        for (SimpleOuter outer : new SimpleOuter[] { new SimpleOuter(4), null }) {
            run("qualified", outer, false);
            run("prepared", outer, true);
        }
    }

    private static void run(String label, SimpleOuter outer, boolean prepared) {
        SimpleOuter.trace = "";
        try {
            Object result = prepared
                    ? EffectOrder.preparedBeforeCheck(outer, 3)
                    : EffectOrder.qualified(outer, 3);
            System.out.println(label + ":" + ((SimpleOuter.Inner) result).value() + ":" + SimpleOuter.trace);
        } catch (NullPointerException failure) {
            System.out.println(label + ":null:" + SimpleOuter.trace);
        }
    }
}
