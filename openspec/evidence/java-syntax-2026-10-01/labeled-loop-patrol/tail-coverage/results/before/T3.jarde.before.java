// jarde: presentation of `T3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class T3 extends java.lang.Object {
    public T3() {
        // @method <init>()V
        // @declaration a constructor of `T3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String mixedTail(int arg0) {
        // @method mixedTail(I)Ljava/lang/String;
        // @declaration a static method of `T3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        jarde_loop_10: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                if (local2 == 0) {
                    if (local3 == 1) {
                        continue jarde_loop_10;
                    }
                }
            }
            touch(local1, local2);
            local1.append('E').append(local2).append('.');
            // @bytecode 60
            // the instruction at BCI 60 is not part of the provable subset
        }
        return local1.toString();
    }

    static void touch(java.lang.StringBuilder arg0, int arg1) {
        // @method touch(Ljava/lang/StringBuilder;I)V
        // @declaration a static method of `T3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.append('<').append(arg1).append('>');
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `T3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) mixedTail(3));
        return;
    }
}
