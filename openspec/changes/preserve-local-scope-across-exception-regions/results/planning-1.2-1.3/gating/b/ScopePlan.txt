// jarde: presentation of `ScopePlan` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ScopePlan extends java.lang.Object {
    private ScopePlan() {
        // @method <init>()V
        // @declaration a constructor of `ScopePlan`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int catchOnly(boolean arg0) {
        // @method catchOnly(Z)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (arg0) {
                throw null;
            } else {
                return 1;
            }
        } catch (java.lang.NullPointerException local1) {
            int local2 = 2;
            return local2;
        }
    }

    static int assignedAcrossTry(boolean arg0) {
        // @method assignedAcrossTry(Z)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        try {
            if (arg0) {
                throw null;
            } else {
                local1 = 3;
            }
        } catch (java.lang.NullPointerException local2) {
            local1 = 4;
        }
        return local1;
    }

    static int assignedAcrossIf(boolean arg0) {
        // @method assignedAcrossIf(Z)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        if (arg0) {
            local1 = 5;
        } else {
            local1 = 6;
        }
        return local1;
    }

    static int nestedHandlerOnly(java.lang.Runnable arg0) {
        // @method nestedHandlerOnly(Ljava/lang/Runnable;)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            try {
                arg0.run();
            } catch (java.lang.IllegalArgumentException local1) {
                int local2 = 8;
                return local2;
            }
        } catch (java.lang.RuntimeException local1) {
            return 9;
        }
        return 0;
    }

    static int nestedAcross(boolean arg0) {
        // @method nestedAcross(Z)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        try {
            try {
                if (arg0) {
                    throw null;
                } else {
                    local1 = 10;
                }
            } catch (java.lang.NullPointerException local2) {
                local1 = 11;
            }
        } catch (java.lang.RuntimeException local2) {
            local1 = 12;
        }
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ScopePlan`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print("" + catchOnly(false) + "," + catchOnly(true) + ",");
        java.lang.System.out.print("" + assignedAcrossTry(false) + "," + assignedAcrossTry(true) + ",");
        java.lang.System.out.print("" + assignedAcrossIf(false) + "," + assignedAcrossIf(true) + ",");
        java.lang.System.out.print((java.lang.String) new java.lang.StringBuilder().append(nestedHandlerOnly((java.lang.Runnable) null)).append(",").toString());
        java.lang.System.out.print("" + nestedAcross(false) + "," + nestedAcross(true) + "\n");
        return;
    }
}
