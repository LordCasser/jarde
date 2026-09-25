// jarde: presentation of `LambdaBodyInline` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class LambdaBodyInline extends java.lang.Object {
    // jarde: field Signature projection refused for `eventsLjava/util/List;`: unsupported (field_generic_body_unproved): a same-class Fieldref names this field and descriptor
    static final java.util.List events;

    static int captureCalls;

    static int bodyCalls;

    public LambdaBodyInline() {
        // @method <init>()V
        // @declaration a constructor of `LambdaBodyInline`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int capture() {
        // @method capture()I
        // @declaration a static method of `LambdaBodyInline`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        LambdaBodyInline.captureCalls = LambdaBodyInline.captureCalls + 1;
        LambdaBodyInline.events.add((java.lang.Object) "capture");
        return 10;
    }

    static IntAction build(int captured) {
        // @method build(I)LIntAction;
        // @declaration a static method of `LambdaBodyInline`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (int p0) -> LambdaBodyInline.lambda$build$0(captured, p0);
    }

    static void require(boolean condition, java.lang.String message) {
        // @method require(ZLjava/lang/String;)V
        // @declaration a static method of `LambdaBodyInline`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (!condition) {
            throw new java.lang.AssertionError((java.lang.Object) message);
        } else {
            return;
        }
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `LambdaBodyInline`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        IntAction action;
        int first;
        int second;
        action = build(capture());
        // @bytecode 21
        // the parameter 0 of the invocation at BCI 21 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
        // @bytecode 37
        // the parameter 0 of the invocation at BCI 37 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
        require(LambdaBodyInline.events.toString().equals((java.lang.Object) "[capture]"), (java.lang.String) new java.lang.StringBuilder().append("creation events: ").append((java.lang.Object) LambdaBodyInline.events).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("created captureCalls=").append(LambdaBodyInline.captureCalls).append(" bodyCalls=").append(LambdaBodyInline.bodyCalls).append(" events=").append((java.lang.Object) LambdaBodyInline.events).toString());
        first = action.apply(2);
        // @bytecode 164 161 158 154 157
        // the parameter 0 of the invocation at BCI 164 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
        // @bytecode 200 197 194 188 191
        // the parameter 0 of the invocation at BCI 200 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
        require(LambdaBodyInline.events.toString().equals((java.lang.Object) "[capture, start:10:2, end:10:2]"), (java.lang.String) new java.lang.StringBuilder().append("first events: ").append((java.lang.Object) LambdaBodyInline.events).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("first result=").append(first).append(" bodyCalls=").append(LambdaBodyInline.bodyCalls).append(" events=").append((java.lang.Object) LambdaBodyInline.events).toString());
        second = action.apply(3);
        // @bytecode 325 322 319 315 318
        // the parameter 0 of the invocation at BCI 325 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
        // @bytecode 361 358 355 349 352
        // the parameter 0 of the invocation at BCI 361 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`
        require(LambdaBodyInline.events.toString().equals((java.lang.Object) "[capture, start:10:2, end:10:2, start:10:3, end:10:3]"), (java.lang.String) new java.lang.StringBuilder().append("second events: ").append((java.lang.Object) LambdaBodyInline.events).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("second result=").append(second).append(" bodyCalls=").append(LambdaBodyInline.bodyCalls).append(" events=").append((java.lang.Object) LambdaBodyInline.events).toString());
        return;
    }

    private static int lambda$build$0(int captured, int value) {
        // @method lambda$build$0(II)I
        // @declaration a static method of `LambdaBodyInline`, member flags 0x100a
        // recovered from bytecode; presentation is not claimed to compile
        LambdaBodyInline.events.add((java.lang.Object) ("start:" + captured + ":" + value));
        LambdaBodyInline.bodyCalls = LambdaBodyInline.bodyCalls + 1;
        LambdaBodyInline.events.add((java.lang.Object) ("end:" + captured + ":" + value));
        return captured + value;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `LambdaBodyInline`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        events = new java.util.ArrayList();
    }
}
