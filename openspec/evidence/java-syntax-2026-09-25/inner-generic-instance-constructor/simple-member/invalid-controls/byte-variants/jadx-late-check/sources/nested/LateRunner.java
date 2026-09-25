package nested;

/* JADX INFO: loaded from: late-check.jar:nested/LateRunner.class */
public final class LateRunner {
    public static void main(String[] strArr) {
        for (SimpleOuter simpleOuter : new SimpleOuter[]{new SimpleOuter(4), null}) {
            SimpleOuter.trace = "";
            try {
                System.out.println(((SimpleOuter.Inner) LateUse.make(simpleOuter, 3)).value() + ":" + SimpleOuter.trace);
            } catch (NullPointerException e) {
                System.out.println("null:" + SimpleOuter.trace);
            }
        }
    }
}
