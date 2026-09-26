public class VarargsCallsRunner {
    public static void main(String[] args) {
        VarargsCalls.effects = 0;
        VarargsCalls.order = "";
        System.out.println("ordered=" + VarargsCalls.ordered()
                + "/" + VarargsCalls.effects + "/" + VarargsCalls.order);

        System.out.println("one=" + VarargsCalls.oneString());
        System.out.println("objects=" + VarargsCalls.objectValues());

        VarargsCalls.effects = 0;
        VarargsCalls.order = "";
        System.out.println("explicit=" + VarargsCalls.explicitVarargsArray()
                + "/" + VarargsCalls.effects + "/" + VarargsCalls.order);

        VarargsCalls.effects = 0;
        VarargsCalls.order = "";
        System.out.println("plain=" + VarargsCalls.ordinaryArray()
                + "/" + VarargsCalls.effects + "/" + VarargsCalls.order);

        System.out.println("held=" + VarargsCalls.heldArray());
        System.out.println("overload=" + VarargsCalls.overloadedArray());
        try {
            System.out.println("null=" + VarargsCalls.nullElement());
        } catch (RuntimeException failure) {
            System.out.println("null-exception=" + failure.getClass().getName());
        }
        System.out.println("array=" + VarargsCalls.arrayElement());

        VarargsCalls.effects = 0;
        VarargsCalls.order = "";
        try {
            VarargsCalls.exceptionOrder();
            throw new AssertionError("expected IllegalStateException");
        } catch (IllegalStateException expected) {
            System.out.println("exception=" + expected.getMessage()
                    + "/" + VarargsCalls.effects + "/" + VarargsCalls.order);
        }
    }
}
