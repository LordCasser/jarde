public class EN extends java.lang.Object {
    public EN() {
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.Object) EN$Plain.A).append("/").append(EN$Plain.values().length).append("/").append(EN$WithBody.values().length).toString());
        EN$WithBody.X.extra();
        EN$Impl.R1.run();
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(EN$WithCtor.BIG.size).append("/").append((java.lang.Object) EN$WithCtor.values()[1]).toString());
        return;
    }
}

enum EN$Plain {
    A,
    B,
    C;
}

enum EN$WithBody {
    X {
        void extra() {
            java.lang.System.out.println("X-extra");
            return;
        }
    },
    Y;

    void extra() {
        java.lang.System.out.println("default");
        return;
    }
}

enum EN$Impl implements java.lang.Runnable {
    R1,
    R2;

    public void run() {
        java.lang.System.out.println("run:" + this.name());
        return;
    }
}

enum EN$WithCtor {
    BIG(10),
    SMALL(1);

    final int size;

    private EN$WithCtor(int arg0) {
        this.size = arg0;
    }
}
