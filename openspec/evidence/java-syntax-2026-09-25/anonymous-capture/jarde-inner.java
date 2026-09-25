// jarde: presentation of `AnonymousProbe$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class AnonymousProbe$1 extends java.lang.Object implements AnonymousProbe$Action {
    final int val$captured;

    final AnonymousProbe this$0;

    // jarde: generic Signature projection refused for `<init>(LAnonymousProbe;I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    AnonymousProbe$1(AnonymousProbe arg1, int arg2) {
        // @method <init>(LAnonymousProbe;I)V
        // @declaration a constructor of `AnonymousProbe$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$captured = arg2;
        this.this$0 = arg1;
        super();
        return;
    }

    public int run(int arg1) {
        // @method run(I)I
        // @declaration an instance method of `AnonymousProbe$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.val$captured + AnonymousProbe.access$000(this.this$0) + arg1;
    }
}
