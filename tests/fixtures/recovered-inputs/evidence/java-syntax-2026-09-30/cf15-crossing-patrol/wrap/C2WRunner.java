public class C2WRunner {
    public static void main(String[] args) {
        run(0);
        System.out.println("=== modes ===");
        run(1);
    }

    static void run(int mode) {
        try { C2W.alias(mode); } catch (RuntimeException e) {
            System.out.println("alias:" + e.getMessage() + "/" + (e.getCause() instanceof IllegalStateException));
        }
        if (mode == 1) System.out.println("alias:" + C2W.alias(mode));
        try { C2W.iaeWrap(mode); } catch (RuntimeException e) {
            System.out.println("iaeWrap:" + e.getMessage() + "/" + (e.getCause() instanceof IllegalArgumentException));
        }
        if (mode == 1) System.out.println("iaeWrap:" + C2W.iaeWrap(mode));
        try { C2W.nested(mode); } catch (RuntimeException e) {
            RuntimeException mid = (RuntimeException) e.getCause();
            System.out.println("nested:" + e.getMessage() + "/" + mid.getMessage()
                + "/" + (mid.getCause() instanceof IllegalStateException));
        }
        if (mode == 1) System.out.println("nested:" + C2W.nested(mode));
        try { System.out.println("forward:" + C2W.forward(mode)); } catch (RuntimeException e) {
            System.out.println("forward:threw:" + e.getMessage());
        }
        try { System.out.println("multi:" + C2W.multi(mode)); } catch (RuntimeException e) {
            System.out.println("multi:threw:" + e.getMessage());
        }
        try { C2W.causeOnly(mode); } catch (RuntimeException e) {
            System.out.println("causeOnly:" + (e.getCause() instanceof IllegalStateException) + ":" + e.getMessage());
        }
        if (mode == 1) System.out.println("causeOnly:" + C2W.causeOnly(mode));
        try { System.out.println("overloadNarrow:" + C2W.overloadNarrow(mode)); } catch (RuntimeException e) {
            System.out.println("overloadNarrow:threw:" + e.getMessage());
        }
        try { System.out.println("objectTarget:" + C2W.objectTarget(mode)); } catch (RuntimeException e) {
            System.out.println("objectTarget:threw:" + e.getMessage());
        }
    }
}
