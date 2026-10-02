// jarde: presentation of `DJLoops` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DJLoops extends java.lang.Object {
    public DJLoops() {
        // @method <init>()V
        // @declaration a constructor of `DJLoops`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String dblContLabels(int arg0) {
        // jarde: not recovered: the recovery run for `dblContLabels(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method dblContLabels(I)Ljava/lang/String;
        // @declaration a static method of `DJLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 8 9 10 11 12 15 16 17 18 19 22 23 25 27 28 31 33 34 37 40 41 42 45 48 49 50 53 54 57 59 62 64 67 68 71 74 77 80 83 86 87 90
        // canonical block at BCI 68 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static java.lang.String dblBreakTargets(int arg0) {
        // @method dblBreakTargets(I)Ljava/lang/String;
        // @declaration a static method of `DJLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        loop: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                local4 = 0;
                while (local4 < arg0) {
                    if (local4 == 1) {
                        break;
                    } else if (local2 == 2) {
                        break loop;
    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                        local4 = local4 + 1;
    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String brkContTwoLevels(int arg0) {
        // @method brkContTwoLevels(I)Ljava/lang/String;
        // @declaration a static method of `DJLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            loop: for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                local4 = 0;
                while (local4 < arg0) {
                    if (local4 == 1) {
                        break;
                    } else if (local3 == 2) {
                        break loop;
    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                        local4 = local4 + 1;
    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String contSelfOnly(int arg0) {
        // @method contSelfOnly(I)Ljava/lang/String;
        // @declaration a static method of `DJLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                    if (local4 == 1) {
                    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String contMidLabelOnly(int arg0) {
        // @method contMidLabelOnly(I)Ljava/lang/String;
        // @declaration a static method of `DJLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                local4 = 0;
                while (local4 < arg0) {
                    if (local3 == 2) {
                        break;
                    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                        local4 = local4 + 1;
                    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String brkMidLabelOnly(int arg0) {
        // @method brkMidLabelOnly(I)Ljava/lang/String;
        // @declaration a static method of `DJLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            loop: for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                local4 = 0;
                while (local4 < arg0) {
                    if (local3 == 2) {
                        break loop;
                    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                        local4 = local4 + 1;
                    }
                }
            }
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `DJLoops`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) dblContLabels(3));
        java.lang.System.out.println((java.lang.String) dblBreakTargets(3));
        java.lang.System.out.println((java.lang.String) brkContTwoLevels(3));
        java.lang.System.out.println((java.lang.String) contSelfOnly(3));
        java.lang.System.out.println((java.lang.String) contMidLabelOnly(3));
        java.lang.System.out.println((java.lang.String) brkMidLabelOnly(3));
        return;
    }
}
