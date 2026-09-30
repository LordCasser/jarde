// jarde: presentation of `W1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class W1 extends java.lang.Object {
    public W1() {
        // @method <init>()V
        // @declaration a constructor of `W1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int mix(int arg0) {
        // @method mix(I)I
        // @declaration a static method of `W1`, member flags 0x0009
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
                    if (local2 > 4) {
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
        // @declaration a static method of `W1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(mix(7));
        java.lang.System.out.println(mix(3));
        return;
    }
}
