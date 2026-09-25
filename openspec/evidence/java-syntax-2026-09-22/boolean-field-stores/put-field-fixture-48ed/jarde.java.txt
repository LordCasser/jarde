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
        // @bytecode 2
        // the field written at BCI 2 is declared `boolean`, and this layer has no evidence that the value it reads at BCI 2 is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the field's own type rejects
        return;
    }

    public void putInstanceProduced(int arg1, boolean arg2) {
        // @method putInstanceProduced(IZ)V
        // @declaration an instance method of `ZFieldStores`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 6 3
        // the field written at BCI 6 is declared `boolean`, and this layer has no evidence that the value it reads at BCI 6 is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the field's own type rejects
        return;
    }

    public static void putStatic(int arg0) {
        // @method putStatic(I)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the field written at BCI 1 is declared `boolean`, and this layer has no evidence that the value it reads at BCI 1 is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the field's own type rejects
        return;
    }

    public static void putStaticProduced(int arg0, boolean arg1) {
        // @method putStaticProduced(IZ)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 5 2
        // the field written at BCI 5 is declared `boolean`, and this layer has no evidence that the value it reads at BCI 5 is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the field's own type rejects
        return;
    }

    public static void putOn(ZFieldStores arg0, int arg1) {
        // @method putOn(LZFieldStores;I)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the field written at BCI 2 is declared `boolean`, and this layer has no evidence that the value it reads at BCI 2 is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the field's own type rejects
        return;
    }

    public static void putProducedOn(ZFieldStores arg0, int arg1, boolean arg2) {
        // @method putProducedOn(LZFieldStores;IZ)V
        // @declaration a static method of `ZFieldStores`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 6 3
        // the field written at BCI 6 is declared `boolean`, and this layer has no evidence that the value it reads at BCI 6 is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the field's own type rejects
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
