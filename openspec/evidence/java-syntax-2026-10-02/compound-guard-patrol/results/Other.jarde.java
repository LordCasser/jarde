// jarde: presentation of `Other` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class Other extends java.lang.Object {
    static int first;

    Other() {
        // @method <init>()V
        // @declaration a constructor of `Other`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int second() {
        // @method second()I
        // @declaration a static method of `Other`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        int local1;
        local0 = 0;
        local1 = 0;
        while (local1 < Other.first) {
            local0 = local0 + local1;
            local1 = local1 + 1;
        }
        return local0;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `Other`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        Other.first = 5;
    }
}
