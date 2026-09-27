// jarde: presentation of `dt29/InterfaceCast` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class InterfaceCast extends java.lang.Object {
    public InterfaceCast() {
        // @method <init>()V
        // @declaration a constructor of `dt29.InterfaceCast`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Runnable asRunnable(java.io.Closeable arg0) {
        // @method asRunnable(Ljava/io/Closeable;)Ljava/lang/Runnable;
        // @declaration a static method of `dt29.InterfaceCast`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (java.lang.Runnable) arg0;
    }

    public static java.lang.String choose(java.io.Closeable arg0) {
        // @method choose(Ljava/io/Closeable;)Ljava/lang/String;
        // @declaration a static method of `dt29.InterfaceCast`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "closeable";
    }

    public static java.lang.String choose(java.lang.Runnable arg0) {
        // @method choose(Ljava/lang/Runnable;)Ljava/lang/String;
        // @declaration a static method of `dt29.InterfaceCast`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "runnable";
    }

    public static java.lang.String chooseRunnable(java.io.Closeable arg0) {
        // @method chooseRunnable(Ljava/io/Closeable;)Ljava/lang/String;
        // @declaration a static method of `dt29.InterfaceCast`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return choose((java.lang.Runnable) arg0);
    }
}
