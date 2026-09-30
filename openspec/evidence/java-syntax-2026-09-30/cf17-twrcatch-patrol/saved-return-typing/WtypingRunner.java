public class WtypingRunner {
    public static void main(String[] a) throws Exception {
        if (a.length > 0 && a[0].equals("boom")) { Wtyping.boom = true; }
        run("stringLiteral", () -> Wtyping.stringLiteral());
        run("intLiteral", () -> Integer.toString(Wtyping.intLiteral()));
        run("constructorValue", () -> Wtyping.constructorValue().toString());
        run("callReturn", () -> Wtyping.callReturn());
        run("nullValue", () -> String.valueOf(Wtyping.nullValue()));
        run("classLiteral", () -> Wtyping.classLiteral().toString());
    }
    interface Call { String go() throws Exception; }
    static void run(String name, Call call) {
        try {
            System.out.println(name + ":" + call.go());
        } catch (Throwable t) {
            System.out.print(name + ":" + t.getMessage());
            for (Throwable s : t.getSuppressed()) { System.out.print("|sup:" + s.getMessage()); }
            System.out.println();
        }
    }
}
