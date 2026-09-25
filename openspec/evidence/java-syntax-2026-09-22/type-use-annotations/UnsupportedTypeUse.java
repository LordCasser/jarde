class TypeUsePeer {}

class UnsupportedTypeUse {
    @TypeMark("single name") TypeUsePeer peer;
    java.lang.String @TypeMark("array dimension") [] array;
    java.util.List<@TypeMark("generic argument") String> values;
}
