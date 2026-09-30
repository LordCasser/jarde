// jarde: presentation of `T2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class T2 extends java.lang.Object {
    public T2() {
        // @method <init>()V
        // @declaration a constructor of `T2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String twoLabels(int arg0) {
        // @method twoLabels(I)Ljava/lang/String;
        // @declaration a static method of `T2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        loop2: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            loop: for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                local4 = 0;
                while (local4 < arg0) {
                    if (local3 > 0) {
                        if (local4 == 1) {
                            continue loop;
                        }
                    }
                    if (local2 > 0) {
                        if (local4 == 2) {
                            continue loop2;
                        }
                    }
                    local1.append(local3).append(local4).append(',');
                    local4 = local4 + 1;
                }
                local1.append('W').append(local3).append(';');
            }
            local1.append('O').append(local2).append('.');
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `T2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) twoLabels(3));
        return;
    }
}
