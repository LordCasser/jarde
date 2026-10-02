// jarde: presentation of `NM` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NM extends java.lang.Object {
    static java.lang.StringBuilder log = new java.lang.StringBuilder();

    public NM() {
        // @method <init>()V
        // @declaration a constructor of `NM`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int v1(int arg0) {
        // @method v1(I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (NM.class) {
            int local2;
            int local4;
            local2 = 0;
            synchronized (NM.log) {
                for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                    local2 = local2 + local4;
                }
            }
            return local2;
        }
    }

    public static int v2(int arg0) {
        // @method v2(I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (NM.class) {
            int local2;
            int local3;
            local2 = 0;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                synchronized (NM.log) {
                    local2 = local2 + local3;
                }
            }
            return local2;
        }
    }

    public static int v3(int arg0) {
        // @method v3(I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (NM.class) {
            int local2;
            int local4;
            local2 = 0;
            synchronized (NM.log) {
                for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                    local2 = local2 + local4;
                }
            }
            return local2;
        }
    }

    public static int n3(int arg0) {
        // jarde: not recovered: the recovery run for `n3(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method n3(I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 34 42 47 54 58
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static int sr(java.lang.Object arg0, int arg1) {
        // @method sr(Ljava/lang/Object;I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (arg0) {
            int local3;
            int local5;
            local3 = 0;
            synchronized (arg0) {
                for (local5 = 0; local5 < arg1; local5 = local5 + 1) {
                    local3 = local3 + local5;
                }
            }
            return local3;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(v1(5));
        java.lang.System.out.println(v2(4));
        java.lang.System.out.println(v3(5));
        java.lang.System.out.println(n3(5));
        java.lang.System.out.println(sr(new java.lang.Object(), 5));
        return;
    }
}
