// jarde: presentation of `Cf13Exits` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Cf13Exits extends java.lang.Object {
    public Cf13Exits() {
        // @method <init>()V
        // @declaration a constructor of `Cf13Exits`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int noJoinNoContinue(int arg0) {
        // @method noJoinNoContinue(I)I
        // @declaration a static method of `Cf13Exits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            switch (local2 % 3) {
                case 0:
                    local1 = local1 + 1;
                    break;
                case 1:
                    local1 = local1 + 2;
                    break;
                default:
                    local1 = local1 + 3;
                    break;
            }
        }
        return local1;
    }

    public static int twoContinueArms(int arg0) {
        // @method twoContinueArms(I)I
        // @declaration a static method of `Cf13Exits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            switch (local2 % 4) {
                case 0:
                    if (local2 > 4) {
                        continue;
                    } else {
                        local1 = local1 + 1;
                    }
                    break;
                case 1:
                    if (local2 > 5) {
                        continue;
                    } else {
                        local1 = local1 + 2;
                    }
                    break;
                case 2:
                    local1 = local1 + 4;
                    break;
                default:
                    local1 = local1 + 3;
                    break;
            }
            local1 = local1 + 10;
        }
        return local1;
    }

    public static int defaultWholeContinue(int arg0) {
        // @method defaultWholeContinue(I)I
        // @declaration a static method of `Cf13Exits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            switch (local2 % 3) {
                case 0:
                    local1 = local1 + 1;
                    break;
                case 1:
                    local1 = local1 + 2;
                    break;
                default:
                    continue;
            }
            local1 = local1 + 10;
        }
        return local1;
    }

    public static int armBreaksLoop(int arg0) {
        // @method armBreaksLoop(I)I
        // @declaration a static method of `Cf13Exits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        local2 = 0;
        loop: while (local2 < arg0) {
            switch (local2 % 3) {
                case 0:
                    if (local2 > 3) {
                        break loop;
                    } else {
                        local1 = local1 + 1;
                    }
                    break;
                case 1:
                    local1 = local1 + 2;
                    break;
                default:
                    local1 = local1 + 3;
                    break;
            }
            local1 = local1 + 10;
            local2 = local2 + 1;
        }
        return local1;
    }

    public static int whileFormContinue(int arg0) {
        // @method whileFormContinue(I)I
        // @declaration a static method of `Cf13Exits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        local2 = 0;
        while (local2 < arg0) {
            switch (local2 % 3) {
                case 0:
                    local1 = local1 + 1;
                    break;
                case 1:
                    local1 = local1 + 2;
                    break;
                default:
                    if (local2 < 0) {
                        continue;
                    } else {
                        local1 = local1 + 3;
                    }
                    break;
            }
            local1 = local1 + 10;
            local2 = local2 + 1;
        }
        return local1;
    }

    public static int outerLabeledContinue(int arg0) {
        // @method outerLabeledContinue(I)I
        // @declaration a static method of `Cf13Exits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            local3 = 0;
            loop: while (local3 < 3) {
                switch (local3 % 3) {
                    case 0:
                        local1 = local1 + 1;
                        break;
                    default:
                        if (local2 > 1) {
                            break loop;
                        } else {
                            local1 = local1 + 3;
                        }
                        break;
                }
                local1 = local1 + 10;
                local3 = local3 + 1;
            }
        }
        return local1;
    }

    public static int stringSwitchContinue(java.lang.String[] arg0) {
        // @method stringSwitchContinue([Ljava/lang/String;)I
        // @declaration a static method of `Cf13Exits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        java.lang.String[] local2;
        local1 = 0;
        local2 = arg0;
        for (java.lang.String local5 : local2) {
            switch (local5) {
                case "a":
                    local1 = local1 + 1;
                    break;
                case "b":
                    local1 = local1 + 2;
                    break;
                default:
                    if (local5.isEmpty()) {
                        continue;
                    } else {
                        local1 = local1 + 3;
                    }
                    break;
            }
            local1 = local1 + 10;
        }
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Cf13Exits`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(noJoinNoContinue(7));
        java.lang.System.out.println(twoContinueArms(7));
        java.lang.System.out.println(defaultWholeContinue(7));
        java.lang.System.out.println(armBreaksLoop(7));
        java.lang.System.out.println(whileFormContinue(7));
        java.lang.System.out.println(outerLabeledContinue(4));
        java.io.PrintStream saved0 = java.lang.System.out;
        saved0.println(stringSwitchContinue(new java.lang.String[]{"a", "b", "", "c", "a"}));
        return;
    }
}
