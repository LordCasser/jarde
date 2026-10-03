// jarde: presentation of `MV2$Inner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class MV2$Inner extends java.lang.Object {
    int tag;

    final MV2 this$0;

    MV2$Inner(MV2 arg1, int arg2) {
        // @method <init>(LMV2;I)V
        // @declaration a constructor of `MV2$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.this$0 = arg1;
        this.tag = arg2;
        return;
    }

    int total() {
        // @method total()I
        // @declaration an instance method of `MV2$Inner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.tag * 2;
    }
}
