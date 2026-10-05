// jarde: presentation of `WA` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class WA extends java.lang.Object {
    private boolean flag;

    private int count;

    private long big;

    private double d;

    private java.lang.String ref;

    private byte by;

    private short sh;

    private char ch;

    private float fl;

    public WA() {
        // @method <init>()V
        // @declaration a constructor of `WA`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `WA`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        WA local1 = new WA();
        WA.S local2 = local1.new S();
        local2.sB(true);
        local2.sI(1);
        local2.sL(2L);
        local2.sD(0x1.8000000000000p1d);
        local2.sS("x");
        local2.sBy((byte) 4);
        local2.sSh((short) 5);
        local2.sC('c');
        local2.sF(0x1.800000p2f);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("").append(local1.flag).append(local1.count).append(local1.big).append(local1.d).append(local1.ref).append((int) local1.by).append((int) local1.sh).append(local1.ch).append(local1.fl).toString());
        return;
    }

    static boolean access$002(WA arg0, boolean arg1) {
        // @method access$002(LWA;Z)Z
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.flag = arg1;
        return arg1;
    }

    static int access$102(WA arg0, int arg1) {
        // @method access$102(LWA;I)I
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.count = arg1;
        return arg1;
    }

    static long access$202(WA arg0, long arg1) {
        // @method access$202(LWA;J)J
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.big = arg1;
        return arg1;
    }

    static double access$302(WA arg0, double arg1) {
        // @method access$302(LWA;D)D
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.d = arg1;
        return arg1;
    }

    static java.lang.String access$402(WA arg0, java.lang.String arg1) {
        // @method access$402(LWA;Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.ref = arg1;
        return arg1;
    }

    static byte access$502(WA arg0, byte arg1) {
        // @method access$502(LWA;B)B
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.by = arg1;
        return arg1;
    }

    static short access$602(WA arg0, short arg1) {
        // @method access$602(LWA;S)S
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.sh = arg1;
        return arg1;
    }

    static char access$702(WA arg0, char arg1) {
        // @method access$702(LWA;C)C
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.ch = arg1;
        return arg1;
    }

    static float access$802(WA arg0, float arg1) {
        // @method access$802(LWA;F)F
        // @declaration a static method of `WA`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.fl = arg1;
        return arg1;
    }

    class S extends java.lang.Object {
        S() {
            super();
            return;
        }

        void sB(boolean arg1) {
            // @method sB(Z)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$002(WA.this, arg1);
            return;
        }

        void sI(int arg1) {
            // @method sI(I)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$102(WA.this, arg1);
            return;
        }

        void sL(long arg1) {
            // @method sL(J)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$202(WA.this, arg1);
            // @bytecode 8
            // the instruction at BCI 8 is not part of the provable subset
            return;
        }

        void sD(double arg1) {
            // @method sD(D)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$302(WA.this, arg1);
            // @bytecode 8
            // the instruction at BCI 8 is not part of the provable subset
            return;
        }

        void sS(java.lang.String arg1) {
            // @method sS(Ljava/lang/String;)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$402(WA.this, arg1);
            return;
        }

        void sBy(byte arg1) {
            // @method sBy(B)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$502(WA.this, arg1);
            return;
        }

        void sSh(short arg1) {
            // @method sSh(S)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$602(WA.this, arg1);
            return;
        }

        void sC(char arg1) {
            // @method sC(C)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$702(WA.this, arg1);
            return;
        }

        void sF(float arg1) {
            // @method sF(F)V
            // @declaration an instance method of `WA$S`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            WA.access$802(WA.this, arg1);
            return;
        }
    }
}
