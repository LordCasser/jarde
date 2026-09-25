// jarde: presentation of `ZFieldStores` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ZFieldStores extends java.lang.Object {
    public boolean instanceFlag;

    public static boolean staticFlag;

    public boolean ordinaryInstanceFlag;

    public static boolean ordinaryStaticFlag;

    public ZFieldStores() {
        // @method <init>()V
        // @declaration a constructor of `ZFieldStores`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void putInstance(int arg1) {
        // @method putInstance(I)V
        // @declaration an instance method of `ZFieldStores`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.instanceFlag = arg1 % 2 != 0;
        return;
    }

    public void putInstanceProduced(int arg1, boolean arg2) {
        // @method putInstanceProduced(IZ)V
        // @declaration an instance method of `ZFieldStores`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.instanceFlag = ZFieldStoreEffects.value(arg1, arg2) % 2 != 0;
        return;
    }

    public static void putStatic(int arg0) {
        // @method putStatic(I)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ZFieldStores.staticFlag = arg0 % 2 != 0;
        return;
    }

    public static void putStaticProduced(int arg0, boolean arg1) {
        // @method putStaticProduced(IZ)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ZFieldStores.staticFlag = ZFieldStoreEffects.value(arg0, arg1) % 2 != 0;
        return;
    }

    public static void putOn(ZFieldStores arg0, int arg1) {
        // @method putOn(LZFieldStores;I)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0.instanceFlag = arg1 % 2 != 0;
        return;
    }

    public static void putProducedOn(ZFieldStores arg0, int arg1, boolean arg2) {
        // @method putProducedOn(LZFieldStores;IZ)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0.instanceFlag = ZFieldStoreEffects.value(arg1, arg2) % 2 != 0;
        return;
    }

    public void putOrdinaryInstance(boolean arg1) {
        // @method putOrdinaryInstance(Z)V
        // @declaration an instance method of `ZFieldStores`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.ordinaryInstanceFlag = arg1;
        return;
    }

    public void putOrdinaryInstanceTrue() {
        // @method putOrdinaryInstanceTrue()V
        // @declaration an instance method of `ZFieldStores`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.ordinaryInstanceFlag = true;
        return;
    }

    public static void putOrdinaryStatic(boolean arg0) {
        // @method putOrdinaryStatic(Z)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ZFieldStores.ordinaryStaticFlag = arg0;
        return;
    }

    public static void putOrdinaryStaticTrue() {
        // @method putOrdinaryStaticTrue()V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ZFieldStores.ordinaryStaticFlag = true;
        return;
    }
}
