// jarde: presentation of `HoistedBoolean` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class HoistedBoolean extends java.lang.Object {
    public HoistedBoolean() {
        // @method <init>()V
        // @declaration a constructor of `HoistedBoolean`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int copied(boolean arg0, int arg1) {
        // @method copied(ZI)I
        // @declaration a static method of `HoistedBoolean`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local3;
        boolean local2 = arg0;
        if (arg1 == 0) {
            local3 = local2;
        } else {
            local3 = arg0;
        }
        if (local3) {
            return 1;
        } else {
            return 0;
        }
    }

    public static int swapped(boolean arg0, int arg1) {
        // @method swapped(ZI)I
        // @declaration a static method of `HoistedBoolean`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local3;
        boolean local2 = arg0;
        if (arg1 == 0) {
            local3 = arg0;
        } else {
            local3 = local2;
        }
        if (local3) {
            return 1;
        } else {
            return 0;
        }
    }

    public static boolean relayed(boolean arg0) {
        // @method relayed(Z)Z
        // @declaration a static method of `HoistedBoolean`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local3;
        boolean local2;
        boolean local1 = arg0;
        if (arg0) {
            local2 = local1;
            local3 = local2;
        } else {
            local2 = local1;
            local3 = local2;
        }
        return local3;
    }

    public static int literalArmed(boolean arg0) {
        // @method literalArmed(Z)I
        // @declaration a static method of `HoistedBoolean`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        if (arg0) {
            local1 = 1;
        } else {
            local1 = 0;
        }
        if (local1 != 0) {
            return 1;
        } else {
            return 0;
        }
    }

    public static boolean fromParameter(boolean arg0, int arg1) {
        // @method fromParameter(ZI)Z
        // @declaration a static method of `HoistedBoolean`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local2;
        if (arg1 == 0) {
            local2 = arg0;
        } else {
            local2 = arg0;
        }
        return local2;
    }

    public static int intLocal(int arg0) {
        // @method intLocal(I)I
        // @declaration a static method of `HoistedBoolean`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        if (arg0 == 0) {
            local1 = 1;
        } else {
            local1 = arg0;
        }
        return local1;
    }

    public static boolean unproven(boolean arg0) {
        // @method unproven(Z)Z
        // @declaration a static method of `HoistedBoolean`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        if (arg0) {
            local1 = 1;
        } else {
            local1 = 0;
        }
        return local1 % 2 != 0;
    }

    public static int conflicted(boolean arg0, int arg1) {
        // @method conflicted(ZI)I
        // @declaration a static method of `HoistedBoolean`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        if (arg1 == 0) {
            local2 = 1;
        } else {
            // @bytecode 10
            // the value at BCI 9 is presented as `boolean` and the write at BCI 10 stores into `local2`, which this run decided holds `int`, and no conversion this layer's evidence states connects the two: a widening primitive conversion (JLS 5.1.2) is the only one a position performs for itself, so the region is refused rather than published with the value the position would convert differently or with text `javac` refuses
        }
        if (local2 != 0) {
            return 1;
        } else {
            return 0;
        }
    }
}
