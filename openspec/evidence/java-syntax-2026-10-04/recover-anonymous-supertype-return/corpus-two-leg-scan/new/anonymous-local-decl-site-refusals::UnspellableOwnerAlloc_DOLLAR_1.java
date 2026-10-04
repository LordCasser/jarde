// jarde: presentation of `UnspellableOwnerAlloc$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class UnspellableOwnerAlloc$1 extends Base {
    UnspellableOwnerAlloc$1(java.lang.String arg1, int arg2) {
        // @method <init>(Ljava/lang/String;I)V
        // @declaration a constructor of `UnspellableOwnerAlloc$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    java.lang.String render() {
        // @method render()Ljava/lang/String;
        // @declaration an instance method of `UnspellableOwnerAlloc$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 4 7
        // the copy at BCI 3 has no proved local assignment
        UnspellableOwnerAlloc.event("leaf-allocated");
        return super.render();
    }
}
