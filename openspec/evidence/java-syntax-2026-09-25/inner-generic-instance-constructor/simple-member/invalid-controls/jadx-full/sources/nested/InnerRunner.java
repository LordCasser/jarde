package nested;

/* JADX INFO: loaded from: full-target.jar:nested/InnerRunner.class */
public final class InnerRunner {
    public static void main(String[] strArr) {
        for (SimpleOuter simpleOuter : new SimpleOuter[]{new SimpleOuter(4), null}) {
            SimpleOuter.trace = "";
            try {
                System.out.println(((SimpleOuter.Inner) UseInner.make(simpleOuter, SimpleOuter.mark("I", 3))).value() + ":" + SimpleOuter.trace);
            } catch (NullPointerException e) {
                System.out.println("null:" + SimpleOuter.trace);
            }
        }
    }
}
