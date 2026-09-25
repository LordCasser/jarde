package nested;

/* JADX INFO: loaded from: wrong-identity.jar:nested/IdentityRunner.class */
public final class IdentityRunner {
    private static void run(String str, SimpleOuter simpleOuter, SimpleOuter simpleOuter2) {
        SimpleOuter.trace = "";
        try {
            System.out.println(str + ":" + ((SimpleOuter.Inner) IdentityUse.make(simpleOuter, simpleOuter2, 3)).value() + ":" + SimpleOuter.trace);
        } catch (NullPointerException e) {
            System.out.println(str + ":null:" + SimpleOuter.trace);
        }
    }

    public static void main(String[] strArr) {
        run("different", new SimpleOuter(4), new SimpleOuter(40));
        run("checked-null", null, new SimpleOuter(40));
        run("actual-null", new SimpleOuter(4), null);
    }
}
