// jarde: presentation of `cf03/ChainOnly` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf03;

public class ChainOnly extends java.lang.Object {
    public static int hits;

    public ChainOnly() {
        // @method <init>()V
        // @declaration a constructor of `cf03.ChainOnly`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean matches(java.lang.String arg0, java.lang.String arg1) {
        // @method matches(Ljava/lang/String;Ljava/lang/String;)Z
        // @declaration a static method of `cf03.ChainOnly`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf03.ChainOnly.hits = cf03.ChainOnly.hits + 1;
        return arg0.equals((java.lang.Object) arg1);
    }

    public static int chain(java.lang.String arg0) {
        // @method chain(Ljava/lang/String;)I
        // @declaration a static method of `cf03.ChainOnly`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        cf03.ChainOnly.hits = 0;
        if (matches(arg0, "a")) {
            local1 = 1;
        } else {
            if (matches(arg0, "b")) {
                local1 = 2;
            } else {
                if (matches(arg0, "3")) {
                    local1 = 3;
                } else {
                    if (matches(arg0, "$")) {
                        local1 = 4;
                    } else {
                        local1 = -1;
                        cf03.ChainOnly.hits = cf03.ChainOnly.hits + 10;
                    }
                }
            }
        }
        local1 = local1 * 10;
        return java.lang.Math.abs(local1);
    }
}
