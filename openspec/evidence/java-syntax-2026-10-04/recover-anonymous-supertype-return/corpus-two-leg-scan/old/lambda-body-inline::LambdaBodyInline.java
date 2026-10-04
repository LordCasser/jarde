// jarde: presentation of `LambdaBodyInline` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class LambdaBodyInline extends java.lang.Object {
    // jarde: renamed physical lambda helper "lambda$build$0" after proving its single class-wide use
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
        // jarde: lambda companion call renamed at invokedynamic@1
        // @method build(I)LIntAction;
        // @declaration a static method of `LambdaBodyInline`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (int p0) -> LambdaBodyInline.lambda$build$0$jarde(captured, p0);
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
        // @bytecode 0 3 6 7 10 11 14 15 18
        // the carried argument crosses an independent instruction at BCI 19
        // @bytecode 19 21 24 27 30 31 34
        // the carried argument crosses an independent instruction at BCI 19
        // @bytecode 35 37 40 43 48 50 53 56 57 60 62 65 68 71 74 77 80 83 84 87 89 92 95 98 100 103 106 109 111 114 117 120 123 126 127 128 133 134 135 137 140 141 144
        // the carried argument crosses an independent instruction at BCI 145
        // @bytecode 145 148 149 152 154 157 158 161 164 167 170 171 174 175 178
        // the carried argument crosses an independent instruction at BCI 145
        // @bytecode 179 182 183 186 188 191 194 197 200 203 206 211 213 216 219 220 223 225 228 231 234 237 240 243 246 247 250 252 255 256 259 261 264 267 270 272 275 278 281 284 287 288 289 294 295 296 298 301 302 305
        // the carried argument crosses an independent instruction at BCI 306
        // @bytecode 306 309 310 313 315 318 319 322 325 328 331 332 335 336 339
        // the carried argument crosses an independent instruction at BCI 306
        // @bytecode 340 343 344
        // the dependency chain from BCI 344 to final consumer 361 is not bounded
        // @bytecode 340 343 344 349
        // the dependency chain from BCI 349 to final consumer 361 is not bounded
        // @bytecode 352
        // the dependency chain from BCI 352 to final consumer 361 is not bounded
        // @bytecode 340 343 344 349 352 355
        // the dependency chain from BCI 355 to final consumer 361 is not bounded
        // @bytecode 340 343 344 349 352 355 358
        // the dependency chain from BCI 358 to final consumer 361 is not bounded
        // @bytecode 361 358 355 349 344 340 343 352
        // the value at BCI 361 is the entry state of stack depth 0, which no instruction produced
        require(LambdaBodyInline.events.toString().equals((java.lang.Object) "[capture, start:10:2, end:10:2, start:10:3, end:10:3]"), (java.lang.String) new java.lang.StringBuilder().append("second events: ").append((java.lang.Object) LambdaBodyInline.events).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("second result=").append(second).append(" bodyCalls=").append(LambdaBodyInline.bodyCalls).append(" events=").append((java.lang.Object) LambdaBodyInline.events).toString());
        return;
    }

    private static int lambda$build$0$jarde(int captured, int value) {
        // jarde: renamed physical lambda helper "lambda$build$0" to its non-conflicting source name (javac re-synthesizes the physical one beside the lambda expression)
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
