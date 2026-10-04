// jarde: presentation of `G` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class G extends java.lang.Object {
    public G() {
        // @method <init>()V
        // @declaration a constructor of `G`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    G$In explicit(G arg1) {
        // @method explicit(LG;)LG$In;
        // @declaration an instance method of `G`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        arg1.getClass();
        return arg1.new In();
    }

    G$In plain(G arg1) {
        // @method plain(LG;)LG$In;
        // @declaration an instance method of `G`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return arg1.new In();
    }

    class In extends java.lang.Object {
        In() {
            super();
            return;
        }

        int t() {
            // @method t()I
            // @declaration an instance method of `G$In`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return 1;
        }
    }
}
