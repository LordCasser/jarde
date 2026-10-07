// The patrol's `B6` source with the registered `chained` boundary left out, so that the three
// shapes this change recovers — `argless`, `withArg` and the consumed control — can be replayed as
// one whole class: the frozen `B6.class` itself keeps all four shapes, and its `chained` refusal is
// exactly what keeps that class's text incomplete by design.
//
// `SB.main` prints what `B6.main` prints (`9`): the constructions themselves have no side effect,
// and the discarded ones must still run.
public class SB {
    int n;

    SB() { n = 42; }

    SB(int v) { n = v; }

    static void argless() { new SB(); }

    static void withArg() { new SB(7); }

    static void consumed() { int r = new SB(3).n; }

    public static void main(String[] args) {
        argless();
        withArg();
        consumed();
        System.out.println(new SB(9).n);
    }
}
