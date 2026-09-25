package nested;

/* JADX INFO: loaded from: full-target.jar:nested/EffectRunner.class */
public final class EffectRunner {
    public static void main(String[] strArr) {
        for (SimpleOuter simpleOuter : new SimpleOuter[]{new SimpleOuter(4), null}) {
            run("qualified", simpleOuter, false);
            run("prepared", simpleOuter, true);
        }
    }

    private static void run(String str, SimpleOuter simpleOuter, boolean z) {
        SimpleOuter.trace = "";
        try {
            System.out.println(str + ":" + ((SimpleOuter.Inner) (z ? EffectOrder.preparedBeforeCheck(simpleOuter, 3) : EffectOrder.qualified(simpleOuter, 3))).value() + ":" + SimpleOuter.trace);
        } catch (NullPointerException e) {
            System.out.println(str + ":null:" + SimpleOuter.trace);
        }
    }
}
