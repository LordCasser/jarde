package demo;

public enum C2 implements demo.IOps {
    public static final demo.C2 TIMES;

    public static final demo.C2 DIVIDE;

    private final java.lang.String t;

    private static final demo.C2[] $VALUES;

    public static demo.C2[] values() {
        return (demo.C2[]) demo.C2.$VALUES.clone();
    }

    public static demo.C2 valueOf(java.lang.String arg0) {
        return (demo.C2) java.lang.Enum.valueOf(demo.C2.class, arg0);
    }

    private C2(java.lang.String arg1, int arg2, java.lang.String arg3) {
        super(arg1, arg2);
        this.t = arg3;
        return;
    }

    public java.lang.String getT() {
        return this.t;
    }

    private static demo.C2[] $values() {
        return new demo.C2[]{demo.C2.TIMES, demo.C2.DIVIDE};
    }

    C2(java.lang.String arg1, int arg2, java.lang.String arg3, demo.C2$1 arg4) {
        this(arg1, arg2, arg3);
        return;
    }

    static {
        demo.C2.TIMES = new demo.C2$1("TIMES", 0, "*");
        demo.C2.DIVIDE = new demo.C2$2("DIVIDE", 1, "/");
        demo.C2.$VALUES = $values();
    }
}
