package negative;

public final class NegativeRunner {
    public static void main(String[] args) {
        for (NegativeOuter outer : new NegativeOuter[] { new NegativeOuter(), null }) {
            NegativeOuter.trace = "";
            try {
                NegativeUse.nestedEffects(outer, 7);
                System.out.println("ok:" + NegativeOuter.trace);
            } catch (NullPointerException ex) {
                System.out.println("null:" + NegativeOuter.trace);
            }
        }
        NegativeOuter.trace = "";
        try {
            NegativeUse.preEffect(null, 7);
        } catch (NullPointerException ex) {
            System.out.println("pre-null:" + NegativeOuter.trace);
        }
    }
}
