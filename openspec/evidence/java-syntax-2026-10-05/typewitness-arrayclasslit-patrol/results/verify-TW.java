public class TW {
    public TW() {
        // @method <init>()V
        // @declaration a constructor of `TW`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }
    static java.util.List witness() {
        // @method witness()Ljava/util/List;
        // @declaration a static method of `TW`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return java.util.Collections.emptyList();
    }
    static java.lang.Class arrClass() {
        // @method arrClass()Ljava/lang/Class;
        // @declaration a static method of `TW`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return int[].class;
    }
    static java.lang.Class deepArrClass() {
        // @method deepArrClass()Ljava/lang/Class;
        // @declaration a static method of `TW`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String[][].class;
    }
    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `TW`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("" + witness().size() + "/" + arrClass().getName() + "/" + deepArrClass().getName() + "/" + TW$Color.all());
        return;
    }
}
