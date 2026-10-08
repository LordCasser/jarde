public class Scratch {
    private final java.util.concurrent.locks.ReentrantLock a =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock b =
            new java.util.concurrent.locks.ReentrantLock();
    private int count = 0;
    private boolean flag = false;
    private IllegalStateException stored = new IllegalStateException("stored");

    void singleIf(boolean fail) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (fail) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void ifElse(boolean fail) {
        this.a.lock();
        this.b.lock();
        try {
            if (fail) {
                this.count++;
            } else {
                this.count--;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void tailThrow() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (this.flag) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        throw new IllegalStateException("after");
    }

    void tailStoredThrow() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (this.flag) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        throw this.stored;
    }

    void tailAssign() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (this.flag) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        this.count++;
    }

    void tailLoop() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (this.flag) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        while (this.flag) {
            this.count--;
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

    void nestedIf(boolean first, boolean second) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (first) {
                if (second) {
                    this.count++;
                }
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void switchBody(int which) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            switch (which) {
                case 0:
                    this.count++;
                    break;
                case 1:
                    this.count--;
                    break;
                default:
                    throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void bodyReturn(boolean fail) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (fail) {
                return;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void trailingIf(boolean after) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (this.flag) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        if (after) {
            this.count--;
        }
    }
}
