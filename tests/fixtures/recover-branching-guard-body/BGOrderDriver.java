/// The behavior driver of the order leg, compiled by the test with each leg's own compiler and run
/// against both the fixture's own class and the recovered text's recompiled class.
///
/// Every line states what the `finally` must do for a **branching** protected body:
///
/// * **normal** — the branch's fall-through path: the body statement runs and the releases run in
///   the **reverse** order of the acquisitions (`b` before `a`);
/// * **throwing body** — an exception raised *inside* the protected range: it propagates with its
///   own message and the same two releases still run, in the same order — which is the arm the
///   branch takes when the driver arms it;
/// * **both arms** — the `if`/`else` and the two-branch shapes, run over every combination: each
///   arm's statement runs exactly once, and the release runs once per call.
public final class BGOrderDriver {
    private BGOrderDriver() {}

    public static void main(String[] args) throws Exception {
        BGOrder normal = new BGOrder();
        Order.reset();
        normal.singleIf();
        System.out.println("normal count=" + normal.count() + " log=" + Order.log().trim());

        BGOrder failing = new BGOrder();
        Order.reset();
        failing.arm(true);
        try {
            failing.singleIf();
            System.out.println("unexpected=no exception");
        } catch (IllegalStateException failure) {
            System.out.println(
                    "caught="
                            + failure.getMessage()
                            + " count="
                            + failing.count()
                            + " log="
                            + Order.log().trim());
        }

        BGOrder taken = new BGOrder();
        Order.reset();
        taken.ifElse(true);
        System.out.println("ifElse-taken count=" + taken.count() + " log=" + Order.log().trim());

        BGOrder notTaken = new BGOrder();
        Order.reset();
        notTaken.ifElse(false);
        System.out.println(
                "ifElse-not-taken count=" + notTaken.count() + " log=" + Order.log().trim());

        for (int first = 0; first < 2; first++) {
            for (int second = 0; second < 2; second++) {
                BGOrder both = new BGOrder();
                Order.reset();
                both.twoIfs(first == 1, second == 1);
                System.out.println(
                        "twoIfs-"
                                + first
                                + second
                                + " count="
                                + both.count()
                                + " log="
                                + Order.log().trim());
            }
        }
    }
}
