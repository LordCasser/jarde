/// The order-and-exception leg of the branching guard body: the anchor's shapes over this
/// fixture's own recording lock, so the `finally`'s release order and its behavior on an exception
/// raised **inside** the protected range are observable in the driver's log.
///
/// `singleIf` is the anchor's shape verbatim (`if (this.fail) throw …;` — the driver arms it), and
/// `ifElse`/`twoIfs` are the branch variants the fixture pins: the driver runs both arms of the
/// `if`/`else` and all four combinations of the two branches, so a presentation that moved an arm,
/// dropped one, or re-ran the release would answer different lines than the class answers.
public final class BGOrder {
    private final Order a = new Order("a");
    private final Order b = new Order("b");
    private int count = 0;
    private boolean fail = false;

    void singleIf() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (this.fail) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void ifElse(boolean take) {
        this.a.lock();
        this.b.lock();
        try {
            if (take) {
                this.count++;
            } else {
                this.count--;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void twoIfs(boolean first, boolean second) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (first) {
                this.count++;
            }
            if (second) {
                this.count--;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    int count() {
        return this.count;
    }

    void arm(boolean armed) {
        this.fail = armed;
    }
}
