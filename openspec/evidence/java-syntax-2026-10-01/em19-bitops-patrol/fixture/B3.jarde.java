// jarde: presentation of `B3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class B3 extends java.lang.Object {
    public B3() {
        // @method <init>()V
        // @declaration a constructor of `B3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static boolean twoBits(int arg0) {
        // @method twoBits(I)Z
        // @declaration a static method of `B3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (arg0 & 1) != 0 && (arg0 & 2) != 0;
    }

    public static boolean mixed(int arg0) {
        // @method mixed(I)Z
        // @declaration a static method of `B3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (arg0 & 1) != 0 && arg0 > 5;
    }

    public static boolean twoCmp(int arg0) {
        // @method twoCmp(I)Z
        // @declaration a static method of `B3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 > 1 && arg0 < 9;
    }

    public static boolean threeBits(int arg0) {
        // @method threeBits(I)Z
        // @declaration a static method of `B3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (arg0 & 1) != 0 || (arg0 & 2) != 0 && (arg0 & 4) != 0;
    }

    public static java.lang.String use(int arg0) {
        // @method use(I)Ljava/lang/String;
        // @declaration a static method of `B3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "" + twoBits(arg0) + ":" + mixed(arg0) + ":" + twoCmp(arg0) + ":" + threeBits(arg0);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `B3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) use(27));
        java.lang.System.out.println((java.lang.String) use(1));
        return;
    }
}
