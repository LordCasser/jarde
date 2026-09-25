// jarde: presentation of `BoundaryProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BoundaryProbe extends java.lang.Object {
    public BoundaryProbe() {
        // @method <init>()V
        // @declaration a constructor of `BoundaryProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static boolean third(boolean arg0, boolean arg1) {
        // @method third(ZZ)Z
        // @declaration a static method of `BoundaryProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0) {
            return true;
        } else {
            if (arg1) {
                return false;
            } else {
                return true;
            }
        }
    }

    public static boolean backedge(int arg0) {
        // @method backedge(I)Z
        // @declaration a static method of `BoundaryProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        while (arg0 > 0) {
            if (arg0 == 1) {
            } else {
                arg0 = arg0 - 1;
            }
        }
        return false;
        // @bytecode 9 10
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [9]
    }

    public static boolean exception(java.lang.String arg0) {
        // @method exception(Ljava/lang/String;)Z
        // @declaration a static method of `BoundaryProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (arg0.equals((java.lang.Object) "x")) {
                return true;
            } else {
                return false;
            }
        } catch (java.lang.RuntimeException local1) {
            return false;
        }
    }
}
