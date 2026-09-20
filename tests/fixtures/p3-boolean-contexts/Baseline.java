// The baseline driver of the boolean-context fixture: the committed original class is run in a
// controlled way, and this is what its own bytecode answers — the values the executed comparison's
// generated side must return for the same calls. `isZero(0)`, `isZero(1)` and `parity` are the
// inputs the two defects were measured on; the rest of the set is the one the comparison calls every
// member with (README.md records the run).
public class Baseline {
    public static void main(String[] args) {
        System.out.println("isZero(0)=" + BooleanContexts.isZero(0));
        System.out.println("isZero(1)=" + BooleanContexts.isZero(1));
        System.out.println("isZero(7)=" + BooleanContexts.isZero(7));
        System.out.println("isZero(-1)=" + BooleanContexts.isZero(-1));
        System.out.println("flag()=" + BooleanContexts.flag());
        System.out.println("parity(0)=" + BooleanContexts.parity(0));
        System.out.println("parity(1)=" + BooleanContexts.parity(1));
        System.out.println("passed(true)=" + BooleanContexts.passed(true));
        System.out.println("passed(false)=" + BooleanContexts.passed(false));
        System.out.println("callFlag()=" + BooleanContexts.callFlag());
        System.out.println("fieldFlag()=" + BooleanContexts.fieldFlag());
        System.out.println("localFromCall()=" + BooleanContexts.localFromCall());
        System.out.println("pick(7, true, true)=" + BooleanContexts.pick(7, true, true));
        System.out.println("pick(0, false, false)=" + BooleanContexts.pick(0, false, false));
        System.out.println("fromLocal(true)=" + BooleanContexts.fromLocal(true));
        System.out.println("fromLocal(false)=" + BooleanContexts.fromLocal(false));
        System.out.println("assignFromCall(true)=" + BooleanContexts.assignFromCall(true));
        System.out.println("assignFromCall(false)=" + BooleanContexts.assignFromCall(false));
        System.out.println("throughLocal(true)=" + BooleanContexts.throughLocal(true));
        System.out.println("throughLocal(false)=" + BooleanContexts.throughLocal(false));
        System.out.println("staticFlagCount()=" + BooleanContexts.staticFlagCount());
        System.out.println("intLocal(7)=" + BooleanContexts.intLocal(7));
        System.out.println("intLocal(0)=" + BooleanContexts.intLocal(0));
        System.out.println("count(true)=" + BooleanContexts.count(true));
        System.out.println("count(false)=" + BooleanContexts.count(false));
        System.out.println("nonzero(0)=" + BooleanContexts.nonzero(0));
        System.out.println("nonzero(1)=" + BooleanContexts.nonzero(1));
        System.out.println("answer()=" + BooleanContexts.answer());
    }
}
