// jarde: presentation of `LK` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class LK extends java.lang.Object {
    private final java.util.concurrent.locks.ReentrantLock lock;

    private final java.util.concurrent.locks.Condition notFull;

    private int count;

    public LK() {
        // @method <init>()V
        // @declaration a constructor of `LK`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.lock = new java.util.concurrent.locks.ReentrantLock();
        this.notFull = this.lock.newCondition();
        this.count = 0;
        return;
    }

    void put() throws java.lang.InterruptedException {
        // @method put()V
        // @declaration an instance method of `LK`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.lock.lock();
        try {
            while (this.count >= 2) {
                this.notFull.await();
            }
            this.count += 1;
            this.notFull.signalAll();
        } finally {
            this.lock.unlock();
        }
        return;
    }

    int take() throws java.lang.InterruptedException {
        // @method take()I
        // @declaration an instance method of `LK`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.lock.lock();
        try {
            while (this.count <= 0) {
                this.notFull.await();
            }
            this.count -= 1;
            this.notFull.signalAll();
            int local1 = this.count;
            return local1;
        } finally {
            this.lock.unlock();
        }
    }

    boolean tryLockQuick() {
        // @method tryLockQuick()Z
        // @declaration an instance method of `LK`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        if (this.lock.tryLock()) {
            try {
                this.count = this.count + 10;
                int local1 = 1;
                return local1 % 2 != 0;
            } finally {
                this.lock.unlock();
            }
        } else {
            return false;
        }
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `LK`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        LK local1 = new LK();
        local1.put();
        local1.put();
        java.lang.System.out.println("" + local1.take() + "/" + local1.take() + "/" + local1.tryLockQuick());
        return;
    }
}
