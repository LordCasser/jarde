/// A `$`-named parent whose other criteria are unmet: `NB$Box` itself has a non-`Object`
/// superclass, so the one proved single-parameter parent definition does not hold and the
/// header stays refused.
public class NestedExtends extends NB$Box<java.lang.String> {
    public static void main(String[] args) {
        System.out.println("nested");
    }
}
