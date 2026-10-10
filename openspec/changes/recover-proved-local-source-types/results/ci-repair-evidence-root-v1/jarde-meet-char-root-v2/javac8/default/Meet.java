// jarde: presentation of `Meet` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Meet extends java.lang.Object {
    public Meet() {
        // @method <init>()V
        // @declaration a constructor of `Meet`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int at(java.lang.String arg0, int arg1) {
        // @method at(Ljava/lang/String;I)I
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.charAt(arg1);
    }

    static int viaStore(char arg0) {
        // @method viaStore(C)I
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        char local1 = arg0;
        return local1;
    }

    static int pass(int arg0) {
        // @method pass(I)I
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    static int fieldArg(java.lang.String arg0) {
        // @method fieldArg(Ljava/lang/String;)I
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return pass((int) arg0.charAt(0));
    }

    static long viaStoreLong(int arg0) {
        // @method viaStoreLong(I)J
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        long local1 = (long) arg0;
        return local1;
    }

    static byte trunc(int arg0) {
        // @method trunc(I)B
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1 = (byte) arg0;
        return (byte) local1;
    }

    static char grade(int arg0) {
        // @method grade(I)C
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        switch (arg0) {
            case 80:
                return 'B';
            case 90:
            case 95:
                return 'A';
            default:
                return 'C';
        }
    }

    static int stat() {
        // @method stat()I
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return 7;
    }

    static int viaRef(Meet arg0) {
        // @method viaRef(LMeet;)I
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.stat();
    }

    static void unchecked(java.util.List arg0) {
        // @method unchecked(Ljava/util/List;)V
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        arg0.add((java.lang.Object) "x");
        return;
    }

    static void pop2Control() {
        // @method pop2Control()V
        // @declaration a static method of `Meet`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.nanoTime();
        // @bytecode 3
        // the instruction at BCI 3 is not part of the provable subset
        return;
    }
}
