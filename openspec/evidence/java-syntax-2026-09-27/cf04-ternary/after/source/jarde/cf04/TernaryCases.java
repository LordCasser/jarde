// jarde: presentation of `cf04/TernaryCases` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf04;

public class TernaryCases extends java.lang.Object {
    public static int calls;

    private final int value;

    public TernaryCases(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `cf04.TernaryCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1 == null ? 0 : arg2);
        return;
    }

    public TernaryCases(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `cf04.TernaryCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.value = arg1;
        return;
    }

    public TernaryCases(java.lang.String arg1, int arg2, boolean arg3) {
        // @method <init>(Ljava/lang/String;IZ)V
        // @declaration a constructor of `cf04.TernaryCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this(arg2 == 1 ? arg1 : "", arg2 == 0 ? "" : arg1);
        return;
    }

    public TernaryCases(java.lang.String arg1, java.lang.String arg2) {
        // @method <init>(Ljava/lang/String;Ljava/lang/String;)V
        // @declaration a constructor of `cf04.TernaryCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.value = arg1.length() * 10 + arg2.length();
        return;
    }

    public int value() {
        // @method value()I
        // @declaration an instance method of `cf04.TernaryCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value;
    }

    public static int positive(int arg0) {
        // @method positive(I)I
        // @declaration a static method of `cf04.TernaryCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 > 0 ? arg0 : (arg0 + 2) * 3;
    }

    public static boolean choose(boolean arg0, boolean arg1, boolean arg2) {
        // @method choose(ZZZ)Z
        // @declaration a static method of `cf04.TernaryCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? arg1 : arg2;
    }

    public static int nested(boolean arg0, boolean arg1, boolean arg2) {
        // @method nested(ZZZ)I
        // @declaration a static method of `cf04.TernaryCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return !arg0 ? arg2 ? 1 : 2 : arg1 ? 1 : 2;
    }

    private static int arm(int arg0) {
        // @method arm(I)I
        // @declaration a static method of `cf04.TernaryCases`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf04.TernaryCases.calls = cf04.TernaryCases.calls + 1;
        return arg0;
    }

    public static int effect(boolean arg0) {
        // @method effect(Z)I
        // @declaration a static method of `cf04.TernaryCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? arm(1) : arm(2);
    }
}
