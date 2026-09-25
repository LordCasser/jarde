// jarde: presentation of `StaticEarlyReturn` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StaticEarlyReturn extends java.lang.Object {
    static int value;

    public StaticEarlyReturn() {
        // @method <init>()V
        // @declaration a constructor of `StaticEarlyReturn`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean early() {
        // @method early()Z
        // @declaration a static method of `StaticEarlyReturn`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Boolean.getBoolean("jarde.static.early");
    }

    public static int get() {
        // @method get()I
        // @declaration a static method of `StaticEarlyReturn`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return StaticEarlyReturn.value;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `StaticEarlyReturn`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (early()) {
            StaticEarlyReturn.value = 1;
            return;
        } else {
            StaticEarlyReturn.value = 7;
            return;
        }
    }
}
