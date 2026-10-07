/// The behavior driver of the order leg, compiled by the test with each leg's own compiler and run
/// against both the fixture's own class and the recovered text's recompiled class.
///
/// It exercises every completion the two guards cover, and every line states what the `finally`
/// must do:
///
/// * **normal** — the two acquisitions, one body statement, and the releases in the **reverse**
///   order of the acquisitions (`b` before `a`);
/// * **throwing body** — an exception raised *inside* the protected range: it propagates with its
///   own message, and the same two releases still run, in the same order;
/// * **interrupted acquisition** — `lockInterruptibly` throws before the protected range begins:
///   the exception propagates and the log holds **no** release (the bytecode's own row excludes
///   the call, so the `finally` never runs);
/// * **interruptible completion** — the acquisition returns, the body runs, and the release runs.
///
/// The last line is the anchor's own recorded behavior: the patrol's `ML.main` calls the two
/// guarded methods and prints the counter, which answers `2` — the value the recovered text's own
/// two calls answer too.
public final class MLOrderDriver {
    private MLOrderDriver() {}

    public static void main(String[] args) throws Exception {
        MLOrder normal = new MLOrder();
        Order.reset();
        normal.nestedLocks();
        System.out.println("normal count=" + normal.count() + " log=" + Order.log().trim());

        MLOrder failing = new MLOrder();
        Order.reset();
        failing.arm(true);
        try {
            failing.nestedLocksThrowing();
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

        MLOrder interrupted = new MLOrder();
        Order.reset();
        Order.interruptNext = true;
        try {
            interrupted.interruptibly();
            System.out.println("unexpected=no interruption");
        } catch (InterruptedException interruption) {
            System.out.println(
                    "interrupted="
                            + interruption.getMessage()
                            + " count="
                            + interrupted.count()
                            + " log="
                            + Order.log().trim());
        }

        MLOrder again = new MLOrder();
        Order.reset();
        again.interruptibly();
        System.out.println("interruptible count=" + again.count() + " log=" + Order.log().trim());

        MLOrder anchor = new MLOrder();
        Order.reset();
        anchor.nestedLocks();
        try {
            anchor.interruptibly();
        } catch (InterruptedException interruption) {
            System.out.println("unexpected=" + interruption.getMessage());
        }
        System.out.println("anchor=" + anchor.count());
    }
}
