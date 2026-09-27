// jarde: presentation of `OuterReceiverCases` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class OuterReceiverCases extends ReceiverBase {
    private final int state;

    OuterReceiverCases(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `OuterReceiverCases`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.state = arg1;
        return;
    }

    int value() {
        // @method value()I
        // @declaration an instance method of `OuterReceiverCases`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return 2;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `OuterReceiverCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        OuterReceiverCases local1 = new OuterReceiverCases(10);
        OuterReceiverCases local2 = new OuterReceiverCases(20);
        java.lang.System.out.println((java.lang.String) local1.new Member().compare(local2));
        return;
    }

    static int access$000(OuterReceiverCases arg0) {
        // @method access$000(LOuterReceiverCases;)I
        // @declaration a static method of `OuterReceiverCases`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.state;
    }

    class Member extends ReceiverMemberBase {
        Member() {
            super();
            return;
        }

        java.lang.String compare(OuterReceiverCases arg1) {
            // @method compare(LOuterReceiverCases;)Ljava/lang/String;
            // @declaration an instance method of `OuterReceiverCases$Member`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return new java.lang.StringBuilder().append(OuterReceiverCases.access$000(arg1)).append(":").append(OuterReceiverCases.access$000(OuterReceiverCases.this)).append(":").append(OuterReceiverCases.super.value()).append(":").append(this.value()).toString();
        }
    }
}
