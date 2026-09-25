// jarde: presentation of `PlacementSubject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class PlacementSubject extends java.lang.Object {
    @PlaceMark(value = "field-scalar")
    // jarde: type annotation refused: target=0x13 info=[] path=[]: same annotation type is present on the declaration and type target
    int scalarField;

    java.lang.@PlaceMark(value = "field-qualified") String qualifiedField;

    public PlacementSubject() {
        // @method <init>()V
        // @declaration a constructor of `PlacementSubject`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    @PlaceMark(value = "method-scalar")
    java.lang.String scalarMethod(@PlaceMark(value = "parameter-scalar") int arg1, java.lang.@PlaceMark(value = "parameter-qualified") String arg2) {
        // jarde: type annotation refused: target=0x14 info=[] path=[]: same annotation type is present on the declaration and type target
        // jarde: type annotation refused: target=0x16 info=[0] path=[]: same annotation type is present on the declaration and type target
        // @method scalarMethod(ILjava/lang/String;)Ljava/lang/String;
        // @declaration an instance method of `PlacementSubject`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return arg2 + arg1;
    }

    java.lang.@PlaceMark(value = "method-qualified") String qualifiedMethod() {
        // @method qualifiedMethod()Ljava/lang/String;
        // @declaration an instance method of `PlacementSubject`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return "qualified";
    }
}
