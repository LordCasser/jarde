// jarde: presentation of `OuterLocalBoundary$1Local` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class OuterLocalBoundary$1Local extends java.lang.Object {
    final OuterLocalBoundary val$other;

    final OuterLocalBoundary this$0;

    // jarde: generic Signature projection refused for `<init>(LOuterLocalBoundary;LOuterLocalBoundary;)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    OuterLocalBoundary$1Local(OuterLocalBoundary this$0, OuterLocalBoundary arg2) {
        // @method <init>(LOuterLocalBoundary;LOuterLocalBoundary;)V
        // @declaration a constructor of `OuterLocalBoundary$1Local`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$other = arg2;
        this.this$0 = this$0;
        super();
        return;
    }

    java.lang.String read() {
        // @method read()Ljava/lang/String;
        // @declaration an instance method of `OuterLocalBoundary$1Local`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(OuterLocalBoundary.access$000(this.val$other)).append(":").append(OuterLocalBoundary.access$000(this.this$0)).append(":").append(OuterLocalBoundary.access$101(this.this$0)).toString();
    }
}
