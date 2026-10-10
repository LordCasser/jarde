// jarde: presentation of `LocalSourceTypesBoundaries` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class LocalSourceTypesBoundaries extends java.lang.Object {
    private static final java.lang.String CHAR_INPUT = "./?A";

    private static char MUTABLE_CHAR_FIELD;

    private LocalSourceTypesBoundaries() {
        // @method <init>()V
        // @declaration a constructor of `LocalSourceTypesBoundaries`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String charCallAndLiteralWrites(int arg0, boolean arg1, boolean arg2) {
        // @method charCallAndLiteralWrites(IZZ)Ljava/lang/String;
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        char local3;
        java.lang.StringBuilder local4;
        java.lang.String local5;
        local3 = "./?A".charAt(arg0);
        if (arg1) {
            local3 = '\u0000';
        }
        if (arg2) {
            local3 = '\uffff';
        }
        local4 = new java.lang.StringBuilder();
        local4.append(local3);
        switch (local3) {
            case '\u0000':
                local5 = "zero";
                break;
            case '.':
                local5 = "dot";
                break;
            case '/':
                local5 = "slash";
                break;
            case '?':
                local5 = "question";
                break;
            default:
                local5 = "other";
                break;
        }
        return local5 + ":" + (int) local3 + ":" + local4.length();
    }

    public static java.lang.String charFieldSeed() {
        // @method charFieldSeed()Ljava/lang/String;
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        char local0 = LocalSourceTypesBoundaries.MUTABLE_CHAR_FIELD;
        java.lang.StringBuilder local1 = new java.lang.StringBuilder();
        local1.append(local0);
        return "" + (int) local0 + ":" + local1.length();
    }

    public static int charI2cSeed(int arg0) {
        // @method charI2cSeed(I)I
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        char local1 = (char) arg0;
        java.lang.StringBuilder local2 = new java.lang.StringBuilder();
        local2.append(local1);
        return local1 + local2.length();
    }

    public static java.lang.String charEntryParameterSeed(char arg0, boolean arg1) {
        // @method charEntryParameterSeed(CZ)Ljava/lang/String;
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        char local2;
        local2 = arg0;
        if (arg1) {
            local2 = '\uffff';
        }
        java.lang.StringBuilder local3 = new java.lang.StringBuilder();
        local3.append(local2);
        return "" + (int) local2 + ":" + local3.length();
    }

    public static int intWithOutOfRangeWrites(int arg0, int arg1) {
        // @method intWithOutOfRangeWrites(II)I
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        local2 = "./?A".charAt(0);
        if (arg0 == 0) {
            local2 = -1;
        } else if (arg0 == 1) {
            local2 = 65536;
    } else {
            local2 = arg1;
    }
        return local2;
    }

    public static int intWithArithmeticWrite(boolean arg0) {
        // @method intWithArithmeticWrite(Z)I
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = "./?A".charAt(0);
        if (arg0) {
            local1 = local1 + 1;
        } else {
            local1 = 65535;
        }
        return local1;
    }

    public static int intWithUnknownCopyMerge(boolean arg0, int arg1) {
        // @method intWithUnknownCopyMerge(ZI)I
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local3;
        int local2 = arg1;
        local3 = "./?A".charAt(0);
        if (arg0) {
            local3 = local2;
        } else {
            local3 = 65;
        }
        return local3;
    }

    public static void exactStringWritesAfterNull(int arg0) {
        // @method exactStringWritesAfterNull(I)V
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1;
        local1 = null;
        switch (arg0) {
            case 1:
                local1 = "one";
                break;
            case 2:
                local1 = new java.lang.String("two");
                break;
            case 3:
                local1 = "three";
                break;
        }
        java.lang.System.out.println(local1);
        return;
    }

    public static void mixedReferenceWrites(boolean arg0) {
        // @method mixedReferenceWrites(Z)V
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Object local1;
        local1 = null;
        if (arg0) {
            local1 = "text";
        } else {
            local1 = new java.lang.StringBuilder("builder");
        }
        java.lang.System.out.println((java.lang.Object) local1);
        return;
    }

    public static void allNullWrites(boolean arg0) {
        // @method allNullWrites(Z)V
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Object local1;
        local1 = null;
        if (arg0) {
            local1 = null;
        } else {
            local1 = null;
        }
        java.lang.System.out.println((java.lang.Object) local1);
        return;
    }

    private static java.lang.Object opaqueCopy(java.lang.Object arg0) {
        // @method opaqueCopy(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    public static void unknownReferenceCopy(boolean arg0, java.lang.Object arg1) {
        // @method unknownReferenceCopy(ZLjava/lang/Object;)V
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Object local2;
        local2 = null;
        if (arg0) {
            local2 = opaqueCopy(arg1);
        } else {
            local2 = "known";
        }
        java.lang.System.out.println((java.lang.Object) local2);
        return;
    }

    public static java.lang.String possibleSlotReuse(boolean arg0) {
        // @method possibleSlotReuse(Z)Ljava/lang/String;
        // @declaration a static method of `LocalSourceTypesBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local1;
        // @bytecode 10
        // local 2 is treated as one source variable, but BCI 10 writes `int` and BCI 17 writes `Object`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
        // @bytecode 15 12 11
        // the parameter 0 of the invocation at BCI 12 is declared `int` has no type fact for the primitive argument
        // @bytecode 17
        // local 2 is treated as one source variable, but BCI 10 writes `int` and BCI 17 writes `Object`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
        if (arg0) {
            // @bytecode 24
            // local 2 is treated as one source variable, but BCI 10 writes `int` and BCI 17 writes `Object`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
        } else {
            // @bytecode 37
            // local 2 is treated as one source variable, but BCI 10 writes `int` and BCI 17 writes `Object`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local
        }
        // @bytecode 38 41 42 45 46 49 51 54 55 58 61
        // the statement at BCI 61 reads `local2`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `LocalSourceTypesBoundaries`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        LocalSourceTypesBoundaries.MUTABLE_CHAR_FIELD = 'F';
    }
}
