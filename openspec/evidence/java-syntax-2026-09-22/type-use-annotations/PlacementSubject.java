public class PlacementSubject {
    @PlaceMark("field-scalar") int scalarField;
    java.lang.@PlaceMark("field-qualified") String qualifiedField;

    @PlaceMark("method-scalar") String scalarMethod(
            @PlaceMark("parameter-scalar") int scalarParameter,
            java.lang.@PlaceMark("parameter-qualified") String qualifiedParameter) {
        return qualifiedParameter + scalarParameter;
    }

    java.lang.@PlaceMark("method-qualified") String qualifiedMethod() {
        return "qualified";
    }
}
