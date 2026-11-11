public class C2WNRunner {
    public static void main(String[] args) {
        run(0);
        System.out.println("=== modes ===");
        run(1);
    }

    static void run(int mode) {
        try { C2WN.userWrap(mode); } catch (RuntimeException e) {
            System.out.println("userWrap:" + e.getMessage() + "/" + (e.getCause() instanceof C2WN.MyFailure));
        }
        if (mode == 1) System.out.println("userWrap:" + C2WN.userWrap(mode));
        try { System.out.println("userForward:" + C2WN.userForward(mode)); } catch (RuntimeException e) {
            System.out.println("userForward:threw:" + e.getMessage());
        }
        try { C2WN.ioWrap(mode); } catch (RuntimeException e) {
            System.out.println("ioWrap:" + e.getMessage() + "/" + (e.getCause() instanceof java.io.IOException));
        }
        if (mode == 1) System.out.println("ioWrap:" + C2WN.ioWrap(mode));
        try { C2WN.causeOnlyUser(mode); } catch (RuntimeException e) {
            System.out.println("causeOnlyUser:" + (e.getCause() instanceof C2WN.MyFailure));
        }
        if (mode == 1) System.out.println("causeOnlyUser:" + C2WN.causeOnlyUser(mode));
        try {
            System.out.println("arrayWrap:" + C2WN.arrayWrap(new IllegalStateException[] { new IllegalStateException("a") }, mode));
        } catch (RuntimeException e) {
            System.out.println("arrayWrap:threw:" + e.getMessage());
        }
        try {
            System.out.println("userArrayWrap:" + C2WN.userArrayWrap(new C2WN.MyFailure[] { new C2WN.MyFailure("c") }, mode));
        } catch (RuntimeException e) {
            System.out.println("userArrayWrap:threw:" + e.getMessage());
        }
        try { System.out.println("overloadNarrow:" + C2WN.overloadNarrow(mode)); } catch (RuntimeException e) {
            System.out.println("overloadNarrow:threw:" + e.getMessage());
        }
        try { System.out.println("rawMix:" + C2WN.rawMix(mode)); } catch (RuntimeException e) {
            System.out.println("rawMix:threw:" + e.getMessage());
        }
    }
}
