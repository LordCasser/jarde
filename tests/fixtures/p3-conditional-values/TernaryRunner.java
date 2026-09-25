public class TernaryRunner {
    private interface Action {
        Object run();
    }

    private static void value(String name, Action action) {
        try {
            Object result = action.run();
            System.out.println(name + "=" + result + ":trace=" + TernaryValues.trace());
        } catch (Throwable error) {
            System.out.println(name + "=throws:" + error.getClass().getName() + ":" +
                error.getMessage() + ":trace=" + TernaryValues.trace());
        }
    }

    private static void run(String name, boolean condition, Action action) {
        TernaryValues.reset(false, false);
        value(name + ":" + condition, action);
    }

    public static void main(String[] args) {
        run("returned", true, () -> TernaryValues.returned(true));
        run("returned", false, () -> TernaryValues.returned(false));
        run("assigned", true, () -> TernaryValues.assigned(true));
        run("assigned", false, () -> TernaryValues.assigned(false));
        run("arithmetic", true, () -> TernaryValues.arithmetic(true));
        run("arithmetic", false, () -> TernaryValues.arithmetic(false));
        run("call", true, () -> TernaryValues.callArgument(true));
        run("call", false, () -> TernaryValues.callArgument(false));
        run("reference", true, () -> TernaryValues.reference(true));
        run("reference", false, () -> TernaryValues.reference(false));
        run("overload", true, () -> TernaryValues.overloadChoice(true));
        run("overload", false, () -> TernaryValues.overloadChoice(false));

        TernaryValues.reset(true, false);
        value("throwing:true:failA", () -> TernaryValues.throwing(true));
        TernaryValues.reset(false, true);
        value("throwing:false:failB", () -> TernaryValues.throwing(false));
        TernaryValues.reset(true, true);
        value("throwing:true:onlyA", () -> TernaryValues.throwing(true));
        value("throwing:false:onlyB", () -> TernaryValues.throwing(false));
    }
}
