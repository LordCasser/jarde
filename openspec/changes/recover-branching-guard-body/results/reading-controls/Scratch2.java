public class Scratch2 {
    private final java.util.concurrent.locks.ReentrantLock a =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock b =
            new java.util.concurrent.locks.ReentrantLock();
    private int count = 0;
    private boolean flag = false;
    private IllegalStateException stored = new IllegalStateException("stored");

    // The separate-block controls: the same completions with a straight (non-branching) body, so
    // the release copy stays in the protected call's block and the return is a block of its own.
    void plainTailStoredThrow() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        throw this.stored;
    }

    void plainTailThrow() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        throw new IllegalStateException("after");
    }

    void plainTailAssign() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        this.count++;
    }

    void plainTailLoop() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        while (this.flag) {
            this.count--;
        }
    }

    void plainSwitchBody(int which) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            switch (which) {
                case 0:
                    this.count++;
                    break;
                default:
                    this.count--;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void bothArmsThrow(boolean first) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (first) {
                throw new IllegalStateException("first");
            } else {
                throw new IllegalStateException("second");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void ladder(int which) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (which == 0) {
                this.count++;
            } else if (which == 1) {
                this.count--;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void bodyLoop() {
        this.a.lock();
        this.b.lock();
        try {
            while (this.flag) {
                this.count++;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void bodyBranchLoop(boolean first) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (first) {
                while (this.flag) {
                    this.count++;
                }
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }
}
