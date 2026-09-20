// The baseline driver of the integer-comparison fixture: the committed original class is run in a
// controlled way, and this is what its own bytecode answers — the values the executed comparison's
// generated side must return for the same calls. The comparisons' input set is `0`, `1`, `2` and
// `-1`, which reaches both arms of `1 == n`, `n == 1`, `0 < n`, `n > 0`, `1 < n` and `n != 0`;
// the boolean shapes at the end are the predecessor change's controls (`README.md` records the run).
public class Baseline {
    public static void main(String[] args) {
        System.out.println("oneFirst(0)=" + IntComparisons.oneFirst(0));
        System.out.println("oneFirst(1)=" + IntComparisons.oneFirst(1));
        System.out.println("oneFirst(2)=" + IntComparisons.oneFirst(2));
        System.out.println("oneFirst(-1)=" + IntComparisons.oneFirst(-1));
        System.out.println("zeroFirst(0)=" + IntComparisons.zeroFirst(0));
        System.out.println("zeroFirst(1)=" + IntComparisons.zeroFirst(1));
        System.out.println("zeroFirst(2)=" + IntComparisons.zeroFirst(2));
        System.out.println("zeroFirst(-1)=" + IntComparisons.zeroFirst(-1));
        System.out.println("oneLess(0)=" + IntComparisons.oneLess(0));
        System.out.println("oneLess(1)=" + IntComparisons.oneLess(1));
        System.out.println("oneLess(2)=" + IntComparisons.oneLess(2));
        System.out.println("oneLess(-1)=" + IntComparisons.oneLess(-1));
        System.out.println("oneLast(0)=" + IntComparisons.oneLast(0));
        System.out.println("oneLast(1)=" + IntComparisons.oneLast(1));
        System.out.println("oneLast(2)=" + IntComparisons.oneLast(2));
        System.out.println("oneLast(-1)=" + IntComparisons.oneLast(-1));
        System.out.println("zeroLast(0)=" + IntComparisons.zeroLast(0));
        System.out.println("zeroLast(1)=" + IntComparisons.zeroLast(1));
        System.out.println("zeroLast(2)=" + IntComparisons.zeroLast(2));
        System.out.println("zeroLast(-1)=" + IntComparisons.zeroLast(-1));
        System.out.println("nonzero(0)=" + IntComparisons.nonzero(0));
        System.out.println("nonzero(1)=" + IntComparisons.nonzero(1));
        System.out.println("nonzero(2)=" + IntComparisons.nonzero(2));
        System.out.println("nonzero(-1)=" + IntComparisons.nonzero(-1));
        System.out.println("isZero(0)=" + IntComparisons.isZero(0));
        System.out.println("isZero(1)=" + IntComparisons.isZero(1));
        System.out.println("isZero(7)=" + IntComparisons.isZero(7));
        System.out.println("isZero(-1)=" + IntComparisons.isZero(-1));
        System.out.println("count(true)=" + IntComparisons.count(true));
        System.out.println("count(false)=" + IntComparisons.count(false));
        System.out.println("throughLocal(true)=" + IntComparisons.throughLocal(true));
        System.out.println("throughLocal(false)=" + IntComparisons.throughLocal(false));
    }
}
