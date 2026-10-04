// jarde: presentation of `ThreeLevel` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ThreeLevel extends java.lang.Object {
    public ThreeLevel() {
        // @method <init>()V
        // @declaration a constructor of `ThreeLevel`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int threeLevel(int arg0) {
        // @method threeLevel(I)I
        // @declaration a static method of `ThreeLevel`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            if (local2 % 2 == 0) {
                continue;
            }
            local3 = 0;
            int local4;
            while (local3 < local2) {
                for (local4 = 0; local4 < local3; local4 = local4 + 1) {
                    local1 = local1 + local4;
                }
                local1 = local1 + local3;
                local3 = local3 + 1;
            }
        }
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ThreeLevel`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(threeLevel(6));
        return;
    }
}
