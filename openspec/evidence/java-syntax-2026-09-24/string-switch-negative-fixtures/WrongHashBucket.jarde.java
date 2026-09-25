// jarde: presentation of `WrongHashBucket` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class WrongHashBucket extends java.lang.Object {
    static int calls;

    public WrongHashBucket() {
        // @method <init>()V
        // @declaration a constructor of `WrongHashBucket`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String read(java.lang.String arg0) {
        // @method read(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `WrongHashBucket`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        WrongHashBucket.calls = WrongHashBucket.calls + 1;
        return arg0;
    }

    public static int choose(java.lang.String arg0) {
        // @method choose(Ljava/lang/String;)I
        // @declaration a static method of `WrongHashBucket`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local3;
        java.lang.String local1 = read(arg0);
        int local2 = local1.hashCode();
        local3 = -1;
        switch (local2) {
            case 2112:
                if (local1.equals((java.lang.Object) "abc")) {
                    local3 = 1;
                }
                break;
            case 96354:
                if (local1.equals((java.lang.Object) "Aa")) {
                    local3 = 0;
                }
                break;
        }
        switch (local3) {
            case 0:
                return 10;
            case 1:
                return 20;
            default:
                return 40;
        }
    }
}
