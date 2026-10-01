// jarde: presentation of `CWN` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class CWN extends java.lang.Object {
    public CWN() {
        // @method <init>()V
        // @declaration a constructor of `CWN`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `take(Ljava/util/List;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String take(java.util.List arg0) {
        // @method take(Ljava/util/List;)Ljava/lang/String;
        // @declaration a static method of `CWN`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (java.lang.String) arg0.get(0);
    }

    // jarde: generic Signature projection refused for `mapValue(Ljava/util/Map;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String mapValue(java.util.Map arg0) {
        // @method mapValue(Ljava/util/Map;)Ljava/lang/String;
        // @declaration a static method of `CWN`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return (java.lang.String) arg0.get((java.lang.Object) "k");
    }

    // jarde: generic Signature projection refused for `setSize(Ljava/util/Set;)I`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static int setSize(java.util.Set arg0) {
        // @method setSize(Ljava/util/Set;)I
        // @declaration a static method of `CWN`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.size();
    }

    // jarde: generic Signature projection refused for `describe(Ljava/lang/Comparable;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String describe(java.lang.Comparable arg0) {
        // @method describe(Ljava/lang/Comparable;)Ljava/lang/String;
        // @declaration a static method of `CWN`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `CWN`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 31 0 28 25 12 22
        // the parameter 0 of the invocation at BCI 22 is declared `java.util.List` presents `CWN$MyList` but the invocation requires `java.util.List` and this layer has no safe reference conversion evidence
        CWN$MySubList local1 = new CWN$MySubList();
        local1.add((java.lang.Object) "sub");
        // @bytecode 74 49 64
        // the parameter 0 of the invocation at BCI 65 is declared `java.util.List` presents `CWN$MySubList` but the invocation requires `java.util.List` and this layer has no safe reference conversion evidence
        java.util.concurrent.ConcurrentHashMap local2 = new java.util.concurrent.ConcurrentHashMap();
        local2.put((java.lang.Object) "k", (java.lang.Object) "concurrent");
        // @bytecode 119 94 109
        // the parameter 0 of the invocation at BCI 110 is declared `java.util.Map` presents `java.util.concurrent.ConcurrentHashMap` but the invocation requires `java.util.Map` and this layer has no safe reference conversion evidence
        // @bytecode 128 125 122
        // the parameter 0 of the invocation at BCI 125 is declared `java.lang.Enum` presents `CWN$Kind` but the invocation requires `java.lang.Enum` and this layer has no safe reference conversion evidence
        // @bytecode 154 129 144
        // the parameter 0 of the invocation at BCI 145 is declared `java.util.Set` presents `java.util.EnumSet` but the invocation requires `java.util.Set` and this layer has no safe reference conversion evidence
        java.util.IdentityHashMap local4 = new java.util.IdentityHashMap();
        local4.put((java.lang.Object) "k", (java.lang.Object) "identity");
        // @bytecode 202 176 191
        // the parameter 0 of the invocation at BCI 193 is declared `java.util.Map` presents `java.util.IdentityHashMap` but the invocation requires `java.util.Map` and this layer has no safe reference conversion evidence
        // @bytecode 240 205 237 234 217 231 228
        // the parameter 0 of the invocation at BCI 228 is declared `java.lang.Comparable` presents `java.util.Date` but the invocation requires `java.lang.Comparable` and this layer has no safe reference conversion evidence
        return;
    }
}
