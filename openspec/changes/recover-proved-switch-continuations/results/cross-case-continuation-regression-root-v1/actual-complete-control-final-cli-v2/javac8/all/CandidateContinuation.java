// jarde: presentation of `CandidateContinuation` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class CandidateContinuation extends java.lang.Object {
    public CandidateContinuation() {
        // @method <init>()V
        // @declaration a constructor of `CandidateContinuation`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int mixedExits(int arg0) {
        // @method mixedExits(I)I
        // @declaration a static method of `CandidateContinuation`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        switch (arg0) {
            case 1:
                local1 = 10;
                break;
            case 2:
                return 20;
        }
        local1 = 30;
        return local1;
    }
}
