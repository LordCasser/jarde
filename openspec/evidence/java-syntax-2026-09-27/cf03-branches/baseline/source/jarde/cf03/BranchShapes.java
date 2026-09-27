// jarde: presentation of `cf03/BranchShapes` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf03;

public class BranchShapes extends java.lang.Object {
    public static int hits;

    public BranchShapes() {
        // @method <init>()V
        // @declaration a constructor of `cf03.BranchShapes`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean matches(java.lang.String arg0, java.lang.String arg1) {
        // @method matches(Ljava/lang/String;Ljava/lang/String;)Z
        // @declaration a static method of `cf03.BranchShapes`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        cf03.BranchShapes.hits = cf03.BranchShapes.hits + 1;
        return arg0.equals((java.lang.Object) arg1);
    }

    public static int chain(java.lang.String arg0) {
        // @method chain(Ljava/lang/String;)I
        // @declaration a static method of `cf03.BranchShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        cf03.BranchShapes.hits = 0;
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
                        cf03.BranchShapes.hits = cf03.BranchShapes.hits + 10;
                    }
                }
            }
        }
        local1 = local1 * 10;
        return java.lang.Math.abs(local1);
    }

    public static boolean nested(boolean arg0, int arg1, int arg2) {
        // jarde: not recovered: the recovery run for `nested(ZII)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nested(ZII)Z
        // @declaration a static method of `cf03.BranchShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 8 9 12 13 14 15 18 19 22 23 24 27 28 29 32 33
        // canonical block at BCI 24 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static int guards(java.lang.String arg0) {
        // @method guards(Ljava/lang/String;)I
        // @declaration a static method of `cf03.BranchShapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 == null) {
            return -1;
        } else {
            if (arg0.length() != 1) {
                return -2;
            } else {
                int local1 = arg0.charAt(0);
                if (local1 == 97) {
                    return 1;
                } else {
                    if (local1 == 98) {
                        return 2;
                    } else {
                        return 0;
                    }
                }
            }
        }
    }
}
