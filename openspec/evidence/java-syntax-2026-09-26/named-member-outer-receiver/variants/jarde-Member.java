// jarde: presentation of `OuterReceiverCases$Member` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class OuterReceiverCases$Member extends ReceiverMemberBase {
    final OuterReceiverCases this$0;

    OuterReceiverCases$Member(OuterReceiverCases this$0) {
        // @method <init>(LOuterReceiverCases;)V
        // @declaration a constructor of `OuterReceiverCases$Member`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.this$0 = this$0;
        super();
        return;
    }

    java.lang.String compare(OuterReceiverCases other) {
        // @method compare(LOuterReceiverCases;)Ljava/lang/String;
        // @declaration an instance method of `OuterReceiverCases$Member`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(OuterReceiverCases.access$000(other)).append(":").append(OuterReceiverCases.access$000(this.this$0)).append(":").append(OuterReceiverCases.access$101(this.this$0)).append(":").append(this.value()).toString();
    }
}
