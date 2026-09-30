// jarde: presentation of `T1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class T1 extends java.lang.Object {
    public T1() {
        // @method <init>()V
        // @declaration a constructor of `T1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String breakTail(int arg0) {
        // @method breakTail(I)Ljava/lang/String;
        // @declaration a static method of `T1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        jarde_loop_10: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            local3 = 0;
            while (local3 < arg0) {
                if (local2 == 0) {
                    if (local3 == 1) {
                        continue jarde_loop_10;
                    }
                }
                if (local2 == 2) {
                    if (local3 == 0) {
                        break jarde_loop_10;
                    }
                }
                local1.append(local2).append(local3).append(',');
                local3 = local3 + 1;
            }
            local1.append('T').append(local2).append(';');
            // @bytecode 82
            // the instruction at BCI 82 is not part of the provable subset
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `T1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) breakTail(4));
        return;
    }
}
