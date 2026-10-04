public class IC extends java.lang.Object {
    public IC() {
        super();
        return;
    }

    static int viaStatic() {
        return IC$Outer$Inner.stat + IC$Outer$Inner.statM();
    }

    static int viaInst() {
        IC$Outer$Inner local0 = new IC$Outer$Inner();
        return local0.inst + local0.instM();
    }

    static int viaIface(IC$Op arg0) {
        return arg0.apply(3);
    }

    static int lambda() {
        return viaIface((IC$Op) ((int arg0) -> arg0 * 7));
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println("" + viaStatic() + "/" + viaInst() + "/" + lambda());
        return;
    }
}

interface IC$Op {
    public abstract int apply(int arg1);
}

class IC$Outer extends java.lang.Object {
    IC$Outer() {
        super();
        return;
    }
}

class IC$Outer$Inner extends java.lang.Object {
    static int stat;

    int inst;

    IC$Outer$Inner() {
        super();
        this.inst = 6;
        return;
    }

    static int statM() {
        return 50;
    }

    int instM() {
        return 60;
    }

    static {
        IC$Outer$Inner.stat = 5;
    }
}
