package dt13;

public final class Runner {
    public static void main(String[] args) {
        NestedShape.Major major = NestedShape.Major.FIRST;
        NestedShape.Major.Minor minor = NestedShape.Major.Minor.LEFT;
        System.out.println("nested=" + major + ":" + minor + ":" + NestedShape.observe());
        System.out.println("interface=" + InterfaceShape.FIRST + ":" + ((Object) InterfaceShape.FIRST instanceof Marker));
        System.out.println("plain=" + PlainShape.FIRST + ":" + ((Object) PlainShape.FIRST instanceof Marker));
        System.out.println("class=" + ((Object) new PlainImpl() instanceof Marker));
    }
}
