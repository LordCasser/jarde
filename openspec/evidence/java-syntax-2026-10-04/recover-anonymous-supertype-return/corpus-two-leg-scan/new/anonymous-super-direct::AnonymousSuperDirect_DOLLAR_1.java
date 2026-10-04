// jarde: presentation of `AnonymousSuperDirect$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class AnonymousSuperDirect$1 extends Base {
    AnonymousSuperDirect$1(int arg1, int arg2) {
        // @method <init>(II)V
        // @declaration a constructor of `AnonymousSuperDirect$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1, arg2);
        return;
    }

    int sum() {
        // @method sum()I
        // @declaration an instance method of `AnonymousSuperDirect$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return super.sum() + 1;
    }
}
