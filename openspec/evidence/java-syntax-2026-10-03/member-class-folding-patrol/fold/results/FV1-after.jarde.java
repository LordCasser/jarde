// jarde: presentation of `VIface` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class VIface extends java.lang.Object {
    VIface() {
        // @method <init>()V
        // @declaration a constructor of `VIface`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] a) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `VIface`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new Tagged().tag());
        return;
    }

    static class Tagged extends java.lang.Object implements Mark {
        Tagged() {
            // @method <init>()V
            // @declaration a constructor of `VIface$Tagged`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int tag() {
            // @method tag()I
            // @declaration an instance method of `VIface$Tagged`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return 5;
        }
    }

    static interface Mark {
        public static final int TAG = 5;
    }
}
