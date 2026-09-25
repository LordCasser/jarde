// jarde: presentation of `DuplicateTarget` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@Tags(value = {@Tag(value = "one"), @Tag(value = "two")})

class DuplicateTarget extends java.lang.Object {
    DuplicateTarget() {
        // @method <init>()V
        // @declaration a constructor of `DuplicateTarget`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }
}
