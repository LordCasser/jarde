public class V17aRunner {
    public static void main(String[] a) throws Exception {
        if (a.length > 0 && a[0].equals("boom")) { V17a.boom = true; }
        run("pureCalls", () -> V17a.pureCalls());
        run("mixedVoidAndCall", () -> V17a.mixedVoidAndCall());
        run("callBeforeReturnInside", () -> V17a.callBeforeReturnInside());
        run("callOnlyReturnInside", () -> V17a.callOnlyReturnInside());
        run("staticCall", () -> V17a.staticCall());
        run("virtualCall", () -> V17a.virtualCall());
        run("interfaceCall", () -> V17a.interfaceCall());
        run("receivedLocal", () -> V17a.receivedLocal());
    }
    interface Call { String go() throws Exception; }
    static void run(String name, Call call) {
        try {
            System.out.println(name + ":" + call.go());
        } catch (Throwable t) {
            System.out.print(name + ":caught:" + t.getMessage());
            for (Throwable s : t.getSuppressed()) { System.out.print("|sup:" + s.getMessage()); }
            System.out.println();
        }
    }
}
