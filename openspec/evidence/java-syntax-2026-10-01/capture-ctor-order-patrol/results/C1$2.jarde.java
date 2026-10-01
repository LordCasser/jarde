// jarde: presentation of `C1$2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class C1$2 extends java.lang.Object implements C1$Op {
    final int val$step;

    // jarde: generic Signature projection refused for `<init>(I)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    C1$2(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `C1$2`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$step = arg1;
        super();
        return;
    }

    public int apply(int arg1) {
        // @method apply(I)I
        // @declaration an instance method of `C1$2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 + this.val$step;
    }
}
